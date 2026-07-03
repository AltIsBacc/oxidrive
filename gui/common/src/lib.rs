use anyhow::Result;
use oxidrive_core::{oxidrive_dsp::pedal::{PedalNode, TypedPedalNode}, pedals::{amp::{AmpNode, AmpParam}, waveshaper::{WaveshaperNode, WaveshaperParam}}, util::ir::load_ir};
use slint::{ComponentHandle, Global};

pub use oxidrive_core;

pub mod platform;
pub mod utils;
pub mod window;
pub mod prefs;
pub mod locale;

pub fn run() -> Result<()> {
    let platform = platform::try_get()
        .ok_or_else(|| anyhow::anyhow!("No platform registered!"))?;

    let window = window::WindowWrapper::from(
        window::MainWindow::new().map_err(Into::<anyhow::Error>::into)?
    );

    prefs::init();
    locale::load(&window);

    window.with_window(|w| {
        w.window().on_close_requested(|| {
            platform.on_event(platform::EventType::SaveState);
            slint::CloseRequestResponse::HideWindow
        });

        window::MaterialWindowAdapter::get(&w).set_disable_hover(true);
    });

    oxidrive_core::with_dsp(|dsp| {
        let ir = platform.get_asset("cabir.wav")
            .map_err(|e| log::warn!("Failed to load cabinet IR: {e}"))
            .ok()
            .and_then(|b| load_ir(b)
                .map_err(|e| log::warn!("Failed to decode cabinet IR: {e}"))
                .ok()
            )
            .unwrap_or_default();

        /* let mut amp = AmpNode::new(ir);

        amp.set_param(AmpParam::InputGain.into(),      1.2);  // slight boost into the stage
        amp.set_param(AmpParam::Drive.into(),          0.9); // crunch, not full distortion
        amp.set_param(AmpParam::Asymmetric.into(),     1.0);  // tube-like asymmetric clip
        amp.set_param(AmpParam::Bass.into(),          -2.0);  // tighten the low end
        amp.set_param(AmpParam::Mid.into(),            3.0);  // mid push, classic British honk
        amp.set_param(AmpParam::Treble.into(),         1.5);  // just enough air
        amp.set_param(AmpParam::OutputLevel.into(),    0.8);  // pull back so it's not clipping output
        amp.set_param(AmpParam::CabinetEnabled.into(), 1.0); */


        let mut waveshaper = WaveshaperNode::new();

        waveshaper.set_param(WaveshaperParam::InputGain, 1.5f);

        dsp.pedals.add_pedal(Box::new(waveshaper));
    });

    window.run()?;

    Ok(())
}

