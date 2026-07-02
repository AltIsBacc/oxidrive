use rtrb::{Consumer, Producer, RingBuffer};

use crate::{engine::AudioCallback, pedal::{PedalNode, commands::PedalChainCommand::{self, *}}};

pub struct PedalChain {
    pub nodes: Vec<Box<dyn PedalNode>>,
    commands: Consumer<PedalChainCommand>,
}

impl PedalChain {
    pub fn new() -> (Self, PedalController) {
        let (producer, consumer) = RingBuffer::new(64);
        let chain  = Self { nodes: Vec::new(), commands: consumer };
        let handle = PedalController { commands: producer };
        (chain, handle)
    }

    fn handle_command(&mut self, cmd: PedalChainCommand) {
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
    fn prepare(&mut self, sample_rate: u32, buffer_size: usize) {
        for node in &mut self.nodes {
            log::info!("pedalnode::prepare for {}", node.name());
            node.prepare(sample_rate, buffer_size);
        }
    }

    fn process_frame(&mut self, data: &mut [f32]) {
        while let Ok(cmd) = self.commands.pop() {
            self.handle_command(cmd);
        }

        for node in &mut self.nodes {
            if !node.bypass() { node.process(data); }
        }
    }
}

pub struct PedalController {
    commands: Producer<PedalChainCommand>,
}

macro_rules! chain_cmd {
    // struct variant: fn name(fields...) => Variant { fields... }
    ($name:ident ( $($arg:ident : $ty:ty),* ) => $variant:ident) => {
        pub fn $name(&mut self, $($arg: $ty),*) {
            self.push($variant { $($arg),* });
        }
    };
    // unit variant: fn name() => Variant
    ($name:ident () => $variant:ident) => {
        pub fn $name(&mut self) {
            self.push($variant);
        }
    };
}

impl PedalController {
    fn push(&mut self, cmd: PedalChainCommand) {
        self.commands.push(cmd).ok();
    }

    chain_cmd!(add_pedal(index: usize, pedal: Box<dyn PedalNode>) => AddPedal);
    chain_cmd!(remove_pedal(index: usize)                         => RemovePedal);
    chain_cmd!(swap_pedals(a: usize, b: usize)                    => SwapPedals);
    chain_cmd!(move_pedal(from: usize, to: usize)                 => MovePedal);
    chain_cmd!(set_bypass(index: usize, bypass: bool)             => SetBypass);
    chain_cmd!(toggle_bypass(index: usize)                        => ToggleBypass);
    chain_cmd!(bypass_all()                                       => BypassAll);
    chain_cmd!(unbypass_all()                                     => UnbypassAll);
    chain_cmd!(load_preset(pedals: Vec<Box<dyn PedalNode>>)       => LoadPreset);
    chain_cmd!(clear()                                            => ClearChain);
    chain_cmd!(set_param(index: usize, param: usize, value: f32)  => SetParam);

    pub fn set_param_typed<P: Into<usize>>(&mut self, index: usize, param: P, value: f32) {
        self.push(PedalChainCommand::SetParam { index, param: param.into(), value });
    }
}

