use anyhow::Context;

use crate::{chain::{PedalChain, PedalChainHandle}, engine::{AudioCallback, AudioDeviceInfo, AudioEngine}};

pub mod engine;
pub mod chain;
pub mod commands;
pub mod node;

pub struct OxidriveDSP {
    pub audio_engine: AudioEngine,
    pub pedals: PedalChainHandle,
}

impl OxidriveDSP {
    pub fn new(input: AudioDeviceInfo, output: AudioDeviceInfo) -> Result<Self> {
        let (chain, handle) = PedalChain::new();
        let audio_engine = AudioEngine::new(input, output)?;

        audio_engine.register_callback(chain);

        Ok(Self {
            audio_engine,
            pedals: handle,
        })
    }

    pub fn with_defaults() -> Result<Self> {
        let (chain, handle) = PedalChain::new();
        let audio_engine = AudioEngine::with_defaults()?;

        Ok(Self {
            audio_engine,
            pedals: handle,
        })
    }

    pub fn init(&mut self) -> Result<()> {
        self.audio_engine.build_streams()?;
        self.audio_engine.play()?;

        Ok(())
    }
}

