use clap::Parser;

pub mod catalog;
pub mod cli;
pub mod config;
pub mod error;
pub mod target;
pub mod ui;

use catalog::{init_catalog, load_catalog};
use cli::{Cli, Command, SetArgs};
use config::{Paths, init_config, load_config, save_config};
use error::{Error, Result};
use target::set_theme;
use ui::render_theme_table;

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    let paths = Paths::new()?;

    init_config(&paths)?;
    init_catalog(&paths)?;

    match cli.command {
        Command::List => list_themes(&paths),
        Command::Current => show_current_theme(&paths),
        Command::Set(args) => handle_set_theme(&paths, args),
    }
}

fn list_themes(paths: &Paths) -> Result<()> {
    let config = load_config(paths)?;
    let catalog = load_catalog(paths)?;

    println!(
        "{}",
        render_theme_table(config.current_theme.as_deref(), &catalog)
    );

    Ok(())
}

fn show_current_theme(paths: &Paths) -> Result<()> {
    let config = load_config(paths)?;
    let current = config
        .current_theme
        .as_ref()
        .ok_or(Error::MissingCurrentTheme)?;

    println!("{current}");

    Ok(())
}

fn handle_set_theme(paths: &Paths, args: SetArgs) -> Result<()> {
    let SetArgs { theme, dry_run } = args;
    let mut config = load_config(paths)?;

    set_theme(paths, &theme, dry_run)?;

    if !dry_run {
        config.current_theme = Some(theme);
        save_config(paths, &config)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::target::{patch_ghostty, patch_zed};
    use jsonc_parser::ParseOptions;
    use jsonc_parser::cst::{CstObject, CstRootNode};

    fn parse_root_object(contents: &str) -> CstObject {
        CstRootNode::parse(contents, &ParseOptions::default())
            .unwrap()
            .value()
            .unwrap()
            .as_object()
            .unwrap()
    }

    fn object_string_prop(obj: &CstObject, name: &str) -> String {
        obj.get(name)
            .and_then(|prop| prop.value())
            .and_then(|value| value.as_string_lit())
            .and_then(|value| value.decoded_value().ok())
            .unwrap()
    }

    #[test]
    fn ghostty_theme_is_replaced() {
        let input = "theme = Old Theme\nfont-size = 16\n";
        let output = patch_ghostty(input, "New Theme");

        assert!(output.contains("theme = New Theme"));
        assert!(output.contains("font-size = 16"));
    }

    #[test]
    fn ghostty_theme_is_appended_if_missing() {
        let input = "font-size = 16\n";
        let output = patch_ghostty(input, "New Theme");

        assert!(output.ends_with("theme = New Theme\n"));
    }

    #[test]
    fn zed_theme_object_is_replaced() {
        let input = r#"{
                        "theme": {
                            "dark": "Old",
                            "light": "Old",
                            "mode": "system"
                        },
                        "vim_mode": true,
                    }"#;
        let output = patch_zed(input, "New").unwrap();
        let root = parse_root_object(&output);
        let theme = root
            .get("theme")
            .and_then(|prop| prop.value())
            .and_then(|value| value.as_object())
            .unwrap();

        assert_eq!(object_string_prop(&theme, "dark"), "New");
        assert_eq!(object_string_prop(&theme, "light"), "New");
        assert_eq!(object_string_prop(&theme, "mode"), "dark");
        assert!(root.get("vim_mode").is_some());
    }

    #[test]
    fn zed_theme_object_is_inserted_if_missing() {
        let input = r#"{
                        "vim_mode": false,
                    }"#;
        let output = patch_zed(input, "New").unwrap();
        let root = parse_root_object(&output);
        let theme = root
            .get("theme")
            .and_then(|prop| prop.value())
            .and_then(|value| value.as_object())
            .unwrap();

        assert_eq!(object_string_prop(&theme, "dark"), "New");
        assert_eq!(object_string_prop(&theme, "light"), "New");
        assert_eq!(object_string_prop(&theme, "mode"), "dark");
        assert!(root.get("vim_mode").is_some());
    }
}
