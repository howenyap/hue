use std::collections::BTreeMap;
use std::fs;

use serde::{Deserialize, Serialize};

use crate::config::Paths;
use crate::error::Result;
use crate::target::Target;

// logic types
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ThemeMapping {
    #[serde(default)]
    pub ghostty: Option<String>,
    #[serde(default)]
    pub helix: Option<String>,
    #[serde(default)]
    pub zed: Option<String>,
}

pub type ThemeCatalog = BTreeMap<String, ThemeMapping>;

// serialisation types
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
struct TargetThemeDefinition {
    name: String,
}

type TargetThemeCatalog = BTreeMap<String, TargetThemeDefinition>;

pub fn init_catalog(paths: &Paths) -> Result<()> {
    fs::create_dir_all(&paths.themes_dir)?;

    for target in Target::ALL {
        let file = format!("{}.toml", target.label());
        let path = paths.themes_dir.join(file);

        if !path.exists() {
            fs::write(path, target.catalog())?;
        }
    }

    Ok(())
}

pub fn load_catalog(paths: &Paths) -> Result<ThemeCatalog> {
    init_catalog(paths)?;

    let contents = Target::ALL
        .into_iter()
        .map(|target| {
            let file = format!("{}.toml", target.label());
            let path = paths.themes_dir.join(file);

            fs::read_to_string(path).map(|contents| (target, contents))
        })
        .collect::<std::result::Result<Vec<_>, _>>()?;

    merge_catalogs(&contents)
}

pub fn merge_catalogs(sources: &[(Target, String)]) -> Result<ThemeCatalog> {
    let mut catalog = ThemeCatalog::new();

    for (target, contents) in sources {
        let target_catalog: TargetThemeCatalog = toml::from_str(contents)?;

        for (logical_name, definition) in target_catalog {
            let mapping = catalog.entry(logical_name).or_default();

            match target {
                Target::Ghostty => mapping.ghostty = Some(definition.name),
                Target::Helix => mapping.helix = Some(definition.name),
                Target::Zed => mapping.zed = Some(definition.name),
            }
        }
    }

    Ok(catalog)
}
