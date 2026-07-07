use anyhow::Result;
use oxidrive_gui_common::platform::EventType;

mod platform;

fn main() -> Result<()> {
    oxidrive_gui_common::platform::register(platform::WindowsPlatform);
    oxidrive_gui_common::platform::dispatch_event(
        EventType::Resume
    );

    oxidrive_gui_common::run()
}

