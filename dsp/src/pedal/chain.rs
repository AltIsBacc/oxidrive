use anyhow::Result;

use crate::{engine::{AudioCallback, buffer::AudioBuffer, streams::ResolvedStreamConfig}, pedal::{BoxedPedal, commands::{ChainCommand, ChainUpdate}}, util::bi_channel::{self, BiDirectionalChannel}};

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
    nodes: Vec<BoxedPedal>,
    channel: BiDirectionalChannel<ChainUpdate, ChainCommand>,
    input_config: Option<ResolvedStreamConfig>,

    is_ready: bool,
}

impl PedalChain {
    pub fn new() -> (Self, PedalController) {
        let (
            ui_side,
            audio_side
        ) = bi_channel::create_bi_channel(256);

        (Self {
            nodes: Vec::with_capacity(10),
            channel: audio_side,
            input_config: None,
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
                self.nodes.push(pedal);
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
        
        for node in &mut self.nodes {
            if !node.should_process() { node.process(data); }
        }
    }
}

