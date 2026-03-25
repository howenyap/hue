use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use jsonc_parser::ParseOptions;
use jsonc_parser::cst::{CstInputValue, CstRootNode};

use crate::catalog::ThemeMapping;
use crate::catalog::load_catalog;
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::ui::diff;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    Ghostty,
    Helix,
    Zed,
}

impl Target {
    pub const ALL: [Self; 3] = [Self::Ghostty, Self::Helix, Self::Zed];

    pub fn catalog(&self) -> &'static str {
        match self {
            Self::Ghostty => include_str!("../themes/ghostty.toml"),
            Self::Helix => include_str!("../themes/helix.toml"),
            Self::Zed => include_str!("../themes/zed.toml"),
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Ghostty => "ghostty",
            Self::Helix => "helix",
            Self::Zed => "zed",
        }
    }

    pub fn mapped_theme<'a>(&self, mapping: &'a ThemeMapping) -> Option<&'a str> {
        match self {
            Self::Ghostty => mapping.ghostty.as_deref(),
            Self::Helix => mapping.helix.as_deref(),
            Self::Zed => mapping.zed.as_deref(),
        }
    }

    pub fn config_path(&self, paths: &Paths) -> PathBuf {
        match self {
            Self::Ghostty => paths.home.join(".config/ghostty/config"),
            Self::Helix => paths.home.join(".config/helix/config.toml"),
            Self::Zed => paths.home.join(".config/zed/settings.json"),
        }
    }

    pub fn apply_theme(&self, contents: &str, theme: &str) -> Result<String> {
        match self {
            Self::Ghostty => Ok(patch_ghostty(contents, theme)),
            Self::Helix => Ok(patch_helix(contents, theme)),
            Self::Zed => patch_zed(contents, theme),
        }
    }

    pub fn apply_mapping(
        &self,
        mapping: &ThemeMapping,
        paths: &Paths,
        dry_run: bool,
    ) -> Result<bool> {
        let Some(theme) = self.mapped_theme(mapping) else {
            return Ok(false);
        };

        let path = self.config_path(paths);

        if !path.exists() {
            return Err(Error::MissingTarget {
                app: *self,
                path: path.clone(),
            }
            .into());
        }

        let original = fs::read_to_string(&path)?;
        let updated = self.apply_theme(&original, theme)?;
        let changed = write_target(&path, *self, &original, &updated, dry_run)?;

        if changed && !dry_run {
            match self {
                // SIGUSR2 to reload ghostty config.
                // Refer to https://github.com/ghostty-org/ghostty/issues/7747.
                Self::Ghostty => {
                    let _ = Command::new("killall").args(["-USR2", "ghostty"]).status();
                }
                // SIGUSR1 to reload helix config
                // Refer to https://github.com/helix-editor/helix/issues/2158#issuecomment-1910775800
                Self::Helix => {
                    let _ = Command::new("pkill").args(["-USR1", "-x", "hx"]).status();
                }
                // Zed hot reloads
                Self::Zed => {}
            }
        }

        Ok(true)
    }
}

impl std::fmt::Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

pub fn set_theme(paths: &Paths, theme_name: &str, dry_run: bool) -> Result<()> {
    let catalog = load_catalog(paths)?;
    let mapping = catalog
        .get(theme_name)
        .ok_or_else(|| Error::UnknownTheme(theme_name.to_string()))?;

    Target::ALL
        .into_iter()
        .filter_map(|target| target.apply_mapping(mapping, paths, dry_run).err())
        .for_each(|error| eprintln!("warning: {error}"));

    Ok(())
}

pub fn write_target(
    path: &Path,
    target: Target,
    original: &str,
    updated: &str,
    dry_run: bool,
) -> Result<bool> {
    let changed = original != updated;

    if dry_run {
        if changed {
            println!("{}", diff(original, updated, path));
        } else {
            println!("{target}: no changes {}\n", path.display());
        }

        return Ok(changed);
    }

    fs::write(path, updated)?;

    Ok(changed)
}

pub fn patch_ghostty(contents: &str, theme: &str) -> String {
    let mut lines: Vec<String> = contents.lines().map(str::to_owned).collect();

    if let Some(line) = lines.iter_mut().find(|line| {
        let trimmed = line.trim();
        let is_comment = trimmed.starts_with('#');
        let is_theme_line = trimmed
            .split_once('=')
            .map(|(key, _)| key.trim() == "theme")
            .unwrap_or(false);

        !is_comment && is_theme_line
    }) {
        *line = format!("theme = {theme}");
    } else {
        lines.push(format!("theme = {theme}"));
    }

    format!("{}\n", lines.join("\n"))
}

pub fn patch_helix(contents: &str, theme: &str) -> String {
    let mut lines: Vec<String> = contents.lines().map(str::to_owned).collect();

    if let Some(line) = lines.iter_mut().find(|line| {
        let trimmed = line.trim();
        let is_comment = trimmed.starts_with('#');
        let is_theme_line = trimmed
            .split_once('=')
            .map(|(key, _)| key.trim() == "theme")
            .unwrap_or(false);

        !is_comment && is_theme_line
    }) {
        *line = format!("theme = \"{theme}\"");
    } else {
        lines.push(format!("theme = \"{theme}\""));
    }

    format!("{}\n", lines.join("\n"))
}

pub fn patch_zed(contents: &str, theme: &str) -> Result<String> {
    let root = CstRootNode::parse(contents, &ParseOptions::default()).map_err(|error| {
        Error::UnsupportedLayout {
            app: Target::Zed,
            reason: format!("invalid JSONC: {error}"),
        }
    })?;

    let obj = root
        .object_value()
        .ok_or_else(|| Error::UnsupportedLayout {
            app: Target::Zed,
            reason: "settings root is not an object".to_string(),
        })?;

    let theme_value = CstInputValue::Object(vec![
        ("dark".to_string(), CstInputValue::String(theme.to_string())),
        (
            "light".to_string(),
            CstInputValue::String(theme.to_string()),
        ),
        (
            "mode".to_string(),
            CstInputValue::String("dark".to_string()),
        ),
    ]);

    if let Some(prop) = obj.get("theme") {
        prop.set_value(theme_value);
    } else {
        obj.append("theme", theme_value);
    }

    Ok(root.to_string())
}
