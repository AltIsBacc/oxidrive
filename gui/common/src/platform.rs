use std::{path::PathBuf, sync::OnceLock};

use anyhow::Result;

use crate::prefs;

#[derive(Debug)]
pub enum DirectoryType {
    Config,
    Cache,
    Data,
}

#[derive(Debug)]
pub enum EventType {
    SaveState,
    Pause,
    Resume,
    Suspend,
}

pub trait Platform: Send + Sync + 'static {
    fn on_event(&self, event: EventType) {
        handle_default_event(&event);
    }

    fn get_dir(&self, dir_type: DirectoryType) -> Result<PathBuf>;
    fn get_asset(&self, path: &str) -> Result<Vec<u8>>;

    fn platform_name(&self) -> &str;
    fn is_mobile(&self) -> bool { false }
}

static PLATFORM: OnceLock<Box<dyn Platform>> = OnceLock::new();

pub fn register(platform: impl Platform) {
    PLATFORM.get_or_init(|| Box::new(platform));
}

pub fn get() -> &'static dyn Platform {
    PLATFORM.get()
        .expect("Platform not registered!")
        .as_ref()
}

pub fn try_get() -> Option<&'static dyn Platform> {
    PLATFORM.get().map(|p| p.as_ref())
}

pub fn dispatch_event(event: EventType) {
    get().on_event(event);
}

pub fn try_dispatch_event(event: EventType) -> bool {
    if let Some(platform) = try_get() {
        platform.on_event(event);
        return true;
    }

    false
}

pub fn handle_default_event(event: &EventType) {
    match event {
        EventType::SaveState | EventType::Suspend => {
            if let Err(e) = prefs::save() {
                log::error!("Failed to save config: {:?}", e);
            }
        },
        EventType::Resume => {
            oxidrive_core::with_dsp(|dsp| dsp.engine.play());
        },
        EventType::Pause => {
            oxidrive_core::with_dsp(|dsp| dsp.engine.pause());
        }
        _ => {}
    }
}

