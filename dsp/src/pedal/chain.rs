use anyhow::Result;

use crate::{
    engine::{AudioCallback, buffer::AudioBuffer, streams::ResolvedStreamConfig},
    pedal::{
        BoxedPedal,
        commands::{ChainCommand, ChainUpdate},
        graph::{NodeId, PedalGraph},
    },
    util::bi_channel::{self, BiDirectionalChannel},
};

pub struct PedalController(BiDirectionalChannel<ChainCommand, ChainUpdate>);

impl PedalController {
    pub fn send_command(&mut self, cmd: ChainCommand) -> Result<()> {
        self.0.send(cmd)
    }

    pub fn pop_update(&mut self) -> Option<ChainUpdate> {
        self.0.recv().ok()
    }
}

pub struct PedalChain {
    graph: PedalGraph,
    channel: BiDirectionalChannel<ChainUpdate, ChainCommand>,
    input_config: Option<ResolvedStreamConfig>,

    /// Scratch buffer a node's summed inputs are accumulated into before
    /// `PedalNode::process` runs on it in-place. Reused across nodes/frames
    /// to avoid allocating on the audio thread.
    mix_scratch: Vec<f32>,

    is_ready: bool,
}

impl PedalChain {
    pub fn new() -> (Self, PedalController) {
        let (
            ui_side,
            audio_side
        ) = bi_channel::create_bi_channel(256);

        (Self {
            graph: PedalGraph::new(),
            channel: audio_side,
            input_config: None,
            mix_scratch: Vec::new(),
            is_ready: false
        }, PedalController(ui_side))
    }

    fn handle_command(&mut self, command: ChainCommand) {
        match command {
            ChainCommand::AddPedal(mut pedal) => {
                // SAFETY: input_config is not null at this point
                pedal.prepare(unsafe {
                    self.input_config.as_ref().unwrap_unchecked()
                });
                self.graph.add_node(pedal);
            }
        }
    }

    /// Fills `mix_scratch` with the summed output of `inputs`, or with
    /// `external` (the raw incoming audio) if the node has no upstream
    /// pedals feeding it, i.e. it sits at the head of a chain.
    ///
    /// Merging is a plain sum: every upstream pedal already applied its own
    /// `mix`/`output_gain` while producing its output buffer, so re-applying
    /// a weight here would double-count it. A pedal that wants to control
    /// how much it contributes to a downstream merge does so via its own
    /// `output_gain`/`mix`, same as it would in a linear chain.
    ///
    /// Takes `graph`/`mix_scratch` as separate borrows (rather than `&mut
    /// self`) so callers can still hold a live borrow into `graph` from a
    /// different field at the same time.
    fn gather_inputs(graph: &PedalGraph, mix_scratch: &mut Vec<f32>, inputs: &[NodeId], external: &[f32]) {
        mix_scratch.clear();
        mix_scratch.resize(external.len(), 0.0);

        if inputs.is_empty() {
            mix_scratch.copy_from_slice(external);
            return;
        }

        for input_id in inputs {
            if let Some(node) = graph.get(*input_id) {
                for (dst, src) in mix_scratch.iter_mut().zip(node.output.iter()) {
                    *dst += *src;
                }
            }
        }
    }
}

impl AudioCallback<f32> for PedalChain {
    fn prepare(&mut self, input_config: ResolvedStreamConfig) {
        self.input_config = Some(input_config);

        self.is_ready = true;
        _ = self.channel.send(ChainUpdate::PedalReady);
    }

    fn process_frame(&mut self, data: &mut AudioBuffer<'_, f32>) {
        if !self.is_ready { return; }

        while let Ok(cmd) = self.channel.recv() {
            self.handle_command(cmd);
            _ = self.channel.send(ChainUpdate::CommandAcknowledged);
        }

        let channels = data.channels();
        let frames = data.frames();
        self.graph.resize_buffers(channels, frames);

        let interleaved = data.interleaved();

        // Process every node in topological order: nodes with no inputs pull
        // straight from the dry input signal, nodes with one or more inputs
        // get those upstream outputs summed together first (this is where
        // branching/merging happens), then the pedal runs in-place on that
        // buffer and the result is cached as this node's output for whatever
        // is downstream of it.
        let order = self.graph.order().to_vec();
        for id in &order {
            let inputs = match self.graph.get(*id) {
                Some(node) => node.inputs.clone(),
                None => continue,
            };

            Self::gather_inputs(&self.graph, &mut self.mix_scratch, &inputs, interleaved);

            if let Some(node) = self.graph.get_mut(*id) {
                {
                    let mut buf = AudioBuffer::wrap(&mut self.mix_scratch, channels);
                    if node.pedal.should_process() {
                        node.pedal.process(&mut buf);
                    }
                }
                node.output.copy_from_slice(&self.mix_scratch);
            }
        }

        // Final mix: sum every sink (a node nothing else consumes) into the
        // real output buffer. With no nodes at all, or all nodes bypassed,
        // this correctly falls through to silence rather than passing the
        // dry signal through, matching "you built an empty/disconnected
        // graph" instead of "everything is a no-op".
        let sinks = self.graph.sinks().to_vec();
        if sinks.is_empty() {
            return;
        }

        for sample in interleaved.iter_mut() {
            *sample = 0.0;
        }

        for id in &sinks {
            if let Some(node) = self.graph.get(*id) {
                for (dst, src) in interleaved.iter_mut().zip(node.output.iter()) {
                    *dst += *src;
                }
            }
        }
    }
}

