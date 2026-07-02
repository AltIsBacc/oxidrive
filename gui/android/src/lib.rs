use oxidrive_gui_common::platform::EventType;
use slint::android;

mod android_types;
mod android_jni_ctx;
mod platform;

fn handle_events(event: &android::android_activity::PollEvent) {
    use android::android_activity::{PollEvent, MainEvent};

    match event {
        PollEvent::Main(MainEvent::GainedFocus) => oxidrive_gui_common::platform::dispatch_event(
            EventType::Resume
        ),
        PollEvent::Main(MainEvent::LostFocus) |
        PollEvent::Main(MainEvent::Pause) => oxidrive_gui_common::platform::dispatch_event(
            EventType::Pause
        ),
        PollEvent::Main(MainEvent::SaveState { saver: _, .. }) => oxidrive_gui_common::platform::dispatch_event(
            EventType::SaveState
        ),
        PollEvent::Main(MainEvent::Destroy) => oxidrive_gui_common::platform::dispatch_event(
            EventType::Suspend
        ),
        _ => { }
    }
}

#[unsafe(no_mangle)]
fn android_main(app: slint::android::AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default()
            .with_tag("Oxidrive-Android")
            .with_max_level(
                log::LevelFilter::Debug
            )
    );

    oxidrive_gui_common::platform::register(
        platform::AndroidPlatform::new(
            app.clone()
        )
    );

    android::init_with_event_listener(
        app, handle_events
    ).expect("Failed to init slint!");

    oxidrive_gui_common::run().expect("Fatal error!");
}

