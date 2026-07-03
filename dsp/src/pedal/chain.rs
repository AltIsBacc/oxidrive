use anyhow::Result;
use rtrb::{Consumer, Producer, RingBuffer};

use crate::{engine::{AudioCallback, buffer::AudioBuffer}, pedal::{BoxedPedal, PedalNode}};

type ChainCommand = Box<dyn FnOnce(&mut Vec<BoxedPedal>) + Send>;

pub struct PedalController(Producer<ChainCommand>);

impl PedalController {
    pub fn new(commands: Producer<ChainCommand>) -> Self {
        Self(commands)
    }

    pub fn queue_command(&mut self, cmd: ChainCommand) -> Result<()> {
       self.0.push(cmd).map_err(|_| anyhow::anyhow!("chain command queue full!"))
    }

    pub fn add_pedal(&mut self, node: BoxedPedal) -> Result<()> {
        self.queue_command(Box::new(move |nodes| nodes.push(node)))
    }
}

pub struct PedalChain {
    nodes: Vec<BoxedPedal>,
    commands: Consumer<ChainCommand>,
}

impl PedalChain {
    pub fn new() -> (Self, PedalController) {
        let (producer, consumer) = RingBuffer::new(
            64
        );

        (Self {
            nodes: Vec::with_capacity(10),
            commands: consumer,
        }, PedalController::new(producer))
    }

    fn consume_commands(&mut self) {
        while let Ok(cmd) = self.commands.pop() { 
            cmd(&mut self.nodes);
        }
    }
}

impl AudioCallback<f32> for PedalChain {
    fn prepare(&mut self, sample_rate: u32, buffer_size: usize) {
        for node in &mut self.nodes {
            log::info!("pedalnode::prepare for {}", node.name());
            node.prepare(sample_rate, buffer_size);
        }
    }

    fn process_frame(&mut self, data: &mut AudioBuffer<'_, f32>) {
        self.consume_commands();
        
        for node in &mut self.nodes {
            if !node.bypass() { node.process(data); }
        }
    }
}

