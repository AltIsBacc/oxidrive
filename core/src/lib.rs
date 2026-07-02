use std::sync::{LazyLock, Mutex};

use anyhow::Context;

pub use oxidrive_dsp;
use oxidrive_dsp::OxidriveDSP;

pub mod pedals;
pub mod util;

static DSP: LazyLock<Mutex<OxidriveDSP>> = LazyLock::new(|| {
    Mutex::new(OxidriveDSP::with_defaults()
        .context("failed to create dsp")
        .unwrap()
    )
});

pub fn with_dsp<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut OxidriveDSP) -> R,
{
    if let Ok(mut guard) = DSP.lock() {
        Some(f(&mut *guard))
    } else {
        None
    }
}

