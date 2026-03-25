use std::fs;
use std::path::PathBuf;

use anyhow::Context;
use directories::BaseDirs;
use serde::{Deserialize, Serialize};

use crate::error::Result;

const CONFIG_FILE: &str = "config.toml";
const CATALOG_FILE: &str = "themes.toml";

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub current_theme: Option<String>,
}

pub struct Paths {
    pub home: PathBuf,
    pub root: PathBuf,
    pub config: PathBuf,
    pub catalog: PathBuf,
}

impl Paths {
    pub fn new() -> Result<Self> {
        let home = BaseDirs::new()
            .context("could not determine the user home directory")?
            .home_dir()
            .to_path_buf();

        let root = home.join(".config/hue");

        Ok(Self {
            home,
            config: root.join(CONFIG_FILE),
            catalog: root.join(CATALOG_FILE),
            root,
        })
    }
}

pub fn init_config(paths: &Paths) -> Result<()> {
    if paths.config.exists() {
        return Ok(());
    }

    fs::create_dir_all(&paths.root)?;

    let config = Config::default();
    save_config(paths, &config)?;

    Ok(())
}

pub fn load_config(paths: &Paths) -> Result<Config> {
    init_config(paths)?;

    let config_str = fs::read_to_string(&paths.config)?;
    let config_toml = toml::from_str(&config_str)?;

    Ok(config_toml)
}

pub fn save_config(paths: &Paths, config: &Config) -> Result<()> {
    let config_str = toml::to_string_pretty(config)?;

    fs::write(&paths.config, config_str)?;

    Ok(())
}
