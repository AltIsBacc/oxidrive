use std::{io::{Read, Write}, sync::{OnceLock, RwLock}};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{platform, utils};

const CONFIG_NAME: &str = "_config";

pub static CONFIG_INSTANCE: OnceLock<RwLock<Config>> = OnceLock::new();

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub language: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            language: "en-us".into()
        }
    }
}

pub fn init() {
    let _ = CONFIG_INSTANCE.get_or_init(|| {
        let config = (|| -> Result<Config> {
            let config_dir = platform::get().get_dir(platform::DirectoryType::Config)?;
            let config_file_path = config_dir.join(format!("{}.json", CONFIG_NAME));

            let mut config_file = utils::io::open_file(config_file_path);
            let mut contents = String::new();
            config_file.read_to_string(&mut contents)?;

            Ok(serde_json::from_str::<Config>(&contents)?)
        })().unwrap_or_default();

        RwLock::new(config)
    });
}

pub fn with_config<F, R>(f: F) -> R 
where
    F: FnOnce(&Config) -> R,
{
    let lock = CONFIG_INSTANCE.get()
        .expect("Config must be initialized before calling with_config!");
    
    let config = lock.read().expect("Config lock was poisoned!");
    f(&config)
}

pub fn save() -> anyhow::Result<()> {
    let lock = CONFIG_INSTANCE.get()
        .ok_or_else(|| anyhow::anyhow!("Config not initialized!"))?;

    let config = lock.read()
        .map_err(|_| anyhow::anyhow!("Lock poisoned"))?;

    let config_dir = crate::platform::get()
        .get_dir(crate::platform::DirectoryType::Config)?;

    let config_file_path = config_dir.join(format!("{CONFIG_NAME}.json"));

    let json_string = serde_json::to_string_pretty(&*config)?;

    let mut config_file = std::fs::File::create(config_file_path)?;
    config_file.write_all(json_string.as_bytes())?;

    Ok(())
}

