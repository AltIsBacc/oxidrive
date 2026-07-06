use anyhow::{Context, Result};

pub use cpal;
use cpal::{Device, HostId, traits::HostTrait};

use crate::{engine::AudioEngine, pedal::chain::{PedalChain, PedalController}};

pub mod engine;
pub mod pedal;
pub mod util;
pub mod traits;

pub struct OxidriveDSP {
    pub engine: AudioEngine,
    pub pedals: PedalController,
}

impl OxidriveDSP {
    pub fn new(input: Device, output: Device) -> Result<Self> {
        let (chain, handle) = PedalChain::new();

        log::info!("Using '{}' for input.", input);
        log::info!("Using '{}' for output.", output);

        let mut engine = AudioEngine::new(input, output)?;

        engine.build_streams(chain)?;

        Ok(Self {
            engine,
            pedals: handle,
        })
    }

    pub fn with_defaults() -> Result<Self> {
        let host = cpal::default_host();

        let input_device = host.default_input_device()
            .context("no default input device")?;
        let output_device = host.default_output_device()
            .context("no default output device")?;

        Self::new(input_device, output_device)
    }

    pub fn with_host(host_id: HostId) -> Result<Self> {
        let host = cpal::host_from_id(host_id)
            .context("failed to find host")?;

        let input_device = host.default_input_device()
            .context("no default input device")?;
        let output_device = host.default_output_device()
            .context("no default output device")?;

        Self::new(input_device, output_device)
    }
}

