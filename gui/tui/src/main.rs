use anyhow::{Context, Result};
use oxidrive_core::{oxidrive_dsp::pedal::{PedalNodeExt, commands::ChainCommand}, pedals::waveshaper::{WaveshaperNode, WaveshaperParam}};

fn main() -> Result<()> {
    loglet::info!("Hello!");

    for host in cpal::available_hosts() {
        loglet::info!("{:?}", host);
    }

    let _ = oxidrive_core::with_dsp(|dsp| {
        let mut waveshaper = WaveshaperNode::new();
        waveshaper.set_param(WaveshaperParam::Asymmetric, 1.0);
        waveshaper.set_param(WaveshaperParam::Drive, 5.0);

        _ = dsp.pedals.send_command(
            ChainCommand::AddPedal(Box::new(waveshaper))
        );
    });

    let _ = oxidrive_core::with_dsp(|d| d.engine.play()).context("failed to play engine")?;

    Ok(())
}

