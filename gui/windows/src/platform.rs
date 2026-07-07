use std::path::PathBuf;

use anyhow::{Context, Result};
use oxidrive_gui_common::platform::{DirectoryType, Platform};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../common/assets/"]
struct Assets;

pub struct WindowsPlatform;

impl Platform for WindowsPlatform {
    fn get_dir(&self, dir_type: DirectoryType) -> Result<PathBuf> {
        match dir_type {
            DirectoryType::Config => sysdirs::config_dir().context("no config dir"),
            DirectoryType::Cache => sysdirs::cache_dir().context("no cache dir"),
            DirectoryType::Data => sysdirs::data_dir().context("no data dir"),
        }
    }

    fn get_asset(&self, path: &str) -> Result<Vec<u8>> {
        Assets::get(path)
            .map(|f| f.data.into_owned())
            .with_context(|| format!("embedded asset not found: {path}"))
    }

    fn platform_name(&self) -> &str { "windows" }
}

