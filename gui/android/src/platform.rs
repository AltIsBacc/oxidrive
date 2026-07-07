use std::{ffi::CString, io::Read, path::PathBuf};

use slint::android::AndroidApp;
use anyhow::Result;
use oxidrive_gui_common::platform::{DirectoryType, Platform};

pub struct AndroidPlatform {
    pub app: AndroidApp
}

impl Platform for AndroidPlatform {
    fn get_dir(&self, dir_type: DirectoryType) -> Result<PathBuf> {
        match dir_type {
            DirectoryType::Config => sysdirs::config_dir()
                .ok_or_else(|| anyhow::anyhow!("Failed to get config dir")),
            DirectoryType::Cache => sysdirs::cache_dir()
                .ok_or_else(|| anyhow::anyhow!("Failed to get cache dir")),
            DirectoryType::Data => sysdirs::data_dir()
                .ok_or_else(|| anyhow::anyhow!("Failed to get data dir")),
        }
    }

    fn get_asset(&self, path: &str) -> Result<Vec<u8>> {
        let manager = self.app.asset_manager();
        let cpath = CString::new(path)?;

        let mut asset = manager.open(&cpath)
            .ok_or_else(|| anyhow::anyhow!("asset not found: {path}"))?;

        let mut buf = Vec::new();
        Read::read_to_end(&mut asset, &mut buf)?;

        Ok(buf)
    }

    fn platform_name(&self) -> &str { "android" }
    fn is_mobile(&self) -> bool { true }
}


