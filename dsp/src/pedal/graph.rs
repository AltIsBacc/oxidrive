use std::collections::VecDeque;

use crate::pedal::BoxedPedal;

/// Stable handle to a node in the pedal graph. Indexes into `PedalGraph::slots`.
/// Ids are never reused for a live node while other nodes may still reference
/// them as an input; removing a node clears incoming edges pointing at it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct NodeId(pub u32);

pub struct GraphNode {
    pub pedal: BoxedPedal,
    /// Upstream nodes feeding this node's input, in no particular order.
    /// Multiple entries here is what makes merging ("branching in") work:
    /// every listed input's last output buffer is summed before processing.
    pub inputs: Vec<NodeId>,
    /// Scratch buffer holding this node's most recently processed output,
    /// interleaved, sized to the stream's buffer size * channels.
    pub output: Vec<f32>,
}

/// A slot-map style storage so NodeIds stay valid across removals: removing a
/// node just leaves a `None` hole instead of shifting every other id.
pub struct PedalGraph {
    slots: Vec<Option<GraphNode>>,
    next_id: u32,

    /// Nodes considered part of the final mix (i.e. the graph's outputs).
    /// A node with no outgoing edges is implicitly a sink; this list is
    /// recomputed whenever topology changes.
    sinks: Vec<NodeId>,

    /// Cached topological processing order, recomputed on any topology edit
    /// (add/remove node, connect/disconnect). Never touched on the audio
    /// thread mid-frame, only between frames when a command mutates topology.
    order: Vec<NodeId>,

    channels: u16,
    frames: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub enum GraphError {
    UnknownNode(NodeId),
    WouldCycle,
}

impl PedalGraph {
    pub fn new() -> Self {
        Self {
            slots: Vec::with_capacity(16),
            next_id: 0,
            sinks: Vec::new(),
            order: Vec::new(),
            channels: 0,
            frames: 0,
        }
    }

    /// Must be called (or re-called) before processing, whenever the stream's
    /// channel count/buffer size is known/changes, so every node's scratch
    /// buffer is sized correctly.
    pub fn resize_buffers(&mut self, channels: u16, frames: usize) {
        self.channels = channels;
        self.frames = frames;

        let len = channels as usize * frames;
        for slot in self.slots.iter_mut().flatten() {
            slot.output.clear();
            slot.output.resize(len, 0.0);
        }
    }

    pub fn add_node(&mut self, pedal: BoxedPedal) -> NodeId {
        let id = NodeId(self.next_id);
        self.next_id += 1;

        let len = self.channels as usize * self.frames;

        let node = GraphNode {
            pedal,
            inputs: Vec::new(),
            output: vec![0.0; len],
        };

        self.slots.push(Some(node));
        // Slots are appended in order and never reordered, so the id can be
        // used directly as an index as long as we started fresh; to keep
        // this true after removals we index by scanning position instead.
        debug_assert_eq!(self.slots.len() as u32 - 1, id.0);

        self.recompute_topology();
        id
    }

    pub fn remove_node(&mut self, id: NodeId) {
        if let Some(slot) = self.slots.get_mut(id.0 as usize) {
            *slot = None;
        }

        // Drop any edges that referenced the removed node as an input.
        for slot in self.slots.iter_mut().flatten() {
            slot.inputs.retain(|input| *input != id);
        }

        self.recompute_topology();
    }

    /// Connects `from -> to`, i.e. `from`'s output becomes one of `to`'s
    /// (possibly several) inputs. Rejects the edge if it would create a
    /// cycle, since the graph is processed in a single topological pass with
    /// no per-node delay compensation.
    pub fn connect(&mut self, from: NodeId, to: NodeId) -> Result<(), GraphError> {
        self.get(from).ok_or(GraphError::UnknownNode(from))?;
        self.get(to).ok_or(GraphError::UnknownNode(to))?;

        if from == to || self.creates_cycle(from, to) {
            return Err(GraphError::WouldCycle);
        }

        let node = self.get_mut(to).ok_or(GraphError::UnknownNode(to))?;
        if !node.inputs.contains(&from) {
            node.inputs.push(from);
        }

        self.recompute_topology();
        Ok(())
    }

    pub fn disconnect(&mut self, from: NodeId, to: NodeId) {
        if let Some(node) = self.get_mut(to) {
            node.inputs.retain(|input| *input != from);
        }

        self.recompute_topology();
    }

    pub fn get(&self, id: NodeId) -> Option<&GraphNode> {
        self.slots.get(id.0 as usize)?.as_ref()
    }

    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut GraphNode> {
        self.slots.get_mut(id.0 as usize)?.as_mut()
    }

    pub fn iter(&self) -> impl Iterator<Item = (NodeId, &GraphNode)> {
        self.slots.iter().enumerate().filter_map(|(i, s)| {
            s.as_ref().map(|n| (NodeId(i as u32), n))
        })
    }

    /// Would adding edge `from -> to` create a cycle? True if `to` can
    /// already reach `from` via existing edges.
    fn creates_cycle(&self, from: NodeId, to: NodeId) -> bool {
        let mut visited = vec![false; self.slots.len()];
        let mut queue = VecDeque::new();
        queue.push_back(to);

        while let Some(current) = queue.pop_front() {
            if current == from {
                return true;
            }

            let idx = current.0 as usize;
            if idx >= visited.len() || visited[idx] {
                continue;
            }
            visited[idx] = true;

            // We're walking "downstream" (who does `current` feed into), so
            // gather every node whose inputs include `current`.
            for (other_id, other) in self.iter() {
                if other.inputs.contains(&current) {
                    queue.push_back(other_id);
                }
            }
        }

        false
    }

    /// Kahn's algorithm over the current edge set. Any node not reachable
    /// (e.g. a cycle that slipped through `connect`'s guard, or dangling
    /// state mid-edit) is simply appended at the end, since a broken graph
    /// should still process deterministically rather than panic.
    fn recompute_topology(&mut self) {
        let mut remaining: Vec<(NodeId, usize)> = self
            .iter()
            .map(|(id, node)| (id, node.inputs.len()))
            .collect();

        let mut order = Vec::with_capacity(remaining.len());

        loop {
            let ready: Vec<NodeId> = remaining
                .iter()
                .filter(|(_, deg)| *deg == 0)
                .map(|(id, _)| *id)
                .collect();

            if ready.is_empty() {
                break;
            }

            for id in &ready {
                order.push(*id);
            }
            remaining.retain(|(rid, _)| !ready.contains(rid));

            for (id, deg) in remaining.iter_mut() {
                if let Some(node) = self.get(*id) {
                    *deg = node.inputs.iter().filter(|input| !order.contains(input)).count();
                }
            }
        }

        // Anything left over is part of a cycle; append in remaining order
        // so processing still terminates rather than dropping nodes.
        for (id, _) in remaining {
            order.push(id);
        }

        self.order = order;
        self.recompute_sinks();
    }

    fn recompute_sinks(&mut self) {
        let ids: Vec<NodeId> = self.iter().map(|(id, _)| id).collect();

        self.sinks.clear();
        for id in ids {
            let has_outgoing = self.iter().any(|(_, node)| node.inputs.contains(&id));
            if !has_outgoing {
                self.sinks.push(id);
            }
        }
    }

    pub fn order(&self) -> &[NodeId] {
        &self.order
    }

    pub fn sinks(&self) -> &[NodeId] {
        &self.sinks
    }
}
