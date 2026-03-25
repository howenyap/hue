use std::collections::BTreeMap;
use std::fs;

use serde::{Deserialize, Serialize};

use crate::config::Paths;
use crate::error::Result;

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ThemeMapping {
    #[serde(default)]
    pub ghostty: Option<String>,
    #[serde(default)]
    pub zed: Option<String>,
}

pub type ThemeCatalog = BTreeMap<String, ThemeMapping>;

const THEME_CATALOG: &str = include_str!("../themes.toml");

pub fn init_catalog(paths: &Paths) -> Result<()> {
    fs::create_dir_all(&paths.root)?;

    if !paths.catalog.exists() {
        fs::write(&paths.catalog, THEME_CATALOG)?;
    }

    Ok(())
}

pub fn load_catalog(paths: &Paths) -> Result<ThemeCatalog> {
    init_catalog(paths)?;

    let contents = fs::read_to_string(&paths.catalog)?;

    Ok(toml::from_str(&contents)?)
}
