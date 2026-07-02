use anyhow::{Context, Result};

fn main() -> Result<()> {
    loglet::info!("Hello!");

    for host in cpal::available_hosts() {
        loglet::info!("{:?}", host);
    }

    let _ = oxidrive_core::with_dsp(|dsp| dsp.engine.play())
        .context("failed to play dsp")?;

    Ok(())
}

