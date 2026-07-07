use anyhow::Result;

mod platform;

fn main() -> Result<()> {
    oxidrive_gui_common::platform::register(platform::WindowsPlatform);
    oxidrive_gui_common::run()
}

