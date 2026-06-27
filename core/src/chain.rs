use rtrb::{Consumer, Producer, RingBuffer};

use crate::{command::ChainCommand, engine::AudioCallback, node::AudioNode};

pub struct PedalChainHandle {
    commands: Consumer<ChainCommand>,
}

macro_rules! chain_cmd {
    // struct variant: fn name(fields...) => Variant { fields... }
    ($name:ident ( $($arg:ident : $ty:ty),* ) => $variant:ident) => {
        pub fn $name(&mut self, $($arg: $ty),*) {
            self.push(ChainCommand::$variant { $($arg),* });
        }
    };
    // unit variant: fn name() => Variant
    ($name:ident () => $variant:ident) => {
        pub fn $name(&mut self) {
            self.push(ChainCommand::$variant);
        }
    };
}

impl PedalChainHandle {
    fn push(&mut self, cmd: ChainCommand) {
        self.commands.push(cmd).ok();
    }

    chain_cmd!(add_pedal(index: usize, pedal: Box<dyn AudioNode>) => AddPedal);
    chain_cmd!(remove_pedal(index: usize)                         => RemovePedal);
    chain_cmd!(swap_pedals(a: usize, b: usize)                    => SwapPedals);
    chain_cmd!(move_pedal(from: usize, to: usize)                 => MovePedal);
    chain_cmd!(set_bypass(index: usize, bypass: bool)             => SetBypass);
    chain_cmd!(toggle_bypass(index: usize)                        => ToggleBypass);
    chain_cmd!(bypass_all()                                       => BypassAll);
    chain_cmd!(unbypass_all()                                     => UnbypassAll);
    chain_cmd!(load_preset(pedals: Vec<Box<dyn AudioNode>>)       => LoadPreset);
    chain_cmd!(clear()                                            => ClearChain);

    pub fn set_param(&mut self, index: usize, param: usize, value: f32) {
        self.push(ChainCommand::SetParam { index, param, value });
    }

    pub fn set_param_typed<P: Into<usize>>(&mut self, index: usize, param: P, value: f32) {
        self.push(ChainCommand::SetParam { index, param: param.into(), value });
    }
}

pub struct PedalChain {
    pub nodes: Vec<Box<dyn AudioNode>>,
    commands: Consumer<ChainCommand>,
}

impl PedalChain {
    pub fn new() -> (Self, PedalChainHandle) {
        let (producer, consumer) = RingBuffer::new(64);
        let chain  = Self { nodes: Vec::new(), commands: consumer };
        let handle = PedalChainHandle { commands: producer };
        (chain, handle)
    }

    fn handle_command(&mut self, cmd: ChainCommand) {
        use crate::commands::ChainCommand::*;
        match cmd {
            AddPedal { index, pedal } => self.nodes.insert(index, pedal),
            RemovePedal { index } => { if index < self.nodes.len() { self.nodes.remove(index); } }
            SwapPedals { a, b } => { if a < self.nodes.len() && b < self.nodes.len() { self.nodes.swap(a, b); } }
            MovePedal { from, to } => {
                if from < self.nodes.len() && to <= self.nodes.len() {
                    let pedal = self.nodes.remove(from);
                    self.nodes.insert(to, pedal);
                }
            }
            SetBypass { index, bypass } => {
                if let Some(node) = self.nodes.get_mut(index) { node.set_bypass(bypass); }
            }
            ToggleBypass { index } => {
                if let Some(node) = self.nodes.get_mut(index) { node.set_bypass(!node.bypass()); }
            }
            BypassAll => self.nodes.iter_mut().for_each(|n| n.set_bypass(true)),
            UnbypassAll  => self.nodes.iter_mut().for_each(|n| n.set_bypass(false)),
            SetParam { index, param, value } => {
                if let Some(node) = self.nodes.get_mut(index) { node.set_param(param, value); }
            }
            LoadPreset { pedals } => self.nodes = pedals,
            ClearChain => self.nodes.clear(),
        }
    }
}

impl AudioCallback for PedalChain {
    fn process_frame(&mut self, data: &mut [f32]) {
        while let Ok(cmd) = self.commands.pop() {
            self.handle_command(cmd);
        }
        for node in &mut self.nodes {
            if !node.bypass() { node.process(data); }
        }
    }
}

