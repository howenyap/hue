use std::fs;
use std::path::PathBuf;

use anyhow::Context;
use directories::BaseDirs;
use serde::{Deserialize, Serialize};

use crate::error::Result;

const CONFIG_FILE: &str = "config.toml";
const THEMES_DIR: &str = "themes";

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub current_theme: Option<String>,
}

pub struct Paths {
    pub home: PathBuf,
    pub hue_root: PathBuf,
    pub config_file: PathBuf,
    pub themes_dir: PathBuf,
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
            config_file: root.join(CONFIG_FILE),
            themes_dir: root.join(THEMES_DIR),
            hue_root: root,
        })
    }
}

pub fn init_config(paths: &Paths) -> Result<()> {
    if paths.config_file.exists() {
        return Ok(());
    }

    fs::create_dir_all(&paths.hue_root)?;

    let config = Config::default();
    save_config(paths, &config)?;

    Ok(())
}

pub fn load_config(paths: &Paths) -> Result<Config> {
    init_config(paths)?;

    let config_str = fs::read_to_string(&paths.config_file)?;
    let config_toml = toml::from_str(&config_str)?;

    Ok(config_toml)
}

pub fn save_config(paths: &Paths, config: &Config) -> Result<()> {
    let config_str = toml::to_string_pretty(config)?;

    fs::write(&paths.config_file, config_str)?;

    Ok(())
}

pub fn reset_config(paths: &Paths) -> Result<()> {
    if paths.hue_root.exists() {
        fs::remove_dir_all(&paths.hue_root)?;
    }

    Ok(())
}
