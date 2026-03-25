use clap::Parser;

pub mod catalog;
pub mod cli;
pub mod config;
pub mod error;
pub mod target;
pub mod ui;

use catalog::load_catalog;
use cli::{Cli, Command, SetArgs};
use config::{Paths, load_config, reset_config, save_config};
use error::{Error, Result};
use target::set_theme;
use ui::render_theme_table;

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    let paths = Paths::new()?;

    match cli.command {
        Command::Config => show_config_dir(&paths),
        Command::List => list_themes(&paths),
        Command::Current => show_current_theme(&paths),
        Command::Reset => handle_reset(&paths),
        Command::Set(args) => handle_set_theme(&paths, args),
    }
}

fn show_config_dir(paths: &Paths) -> Result<()> {
    println!("{}", paths.hue_root.display());

    Ok(())
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

fn handle_reset(paths: &Paths) -> Result<()> {
    reset_config(paths)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::catalog::merge_catalogs;
    use crate::target::{Target, patch_ghostty, patch_helix, patch_zed};
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
    fn helix_theme_is_replaced() {
        let input = r#"
                        theme = "old"
                        [editor]
                        line-number = "relative"
                    "#;
        let output = patch_helix(input, "tokyonight");

        assert!(output.contains(r#"theme = "tokyonight""#));
        assert!(output.contains(r#"line-number = "relative""#));
    }

    #[test]
    fn helix_theme_is_appended_if_missing() {
        let input = r#"
                        [editor]
                        true-color = true
                    "#;
        let output = patch_helix(input, "tokyonight");

        assert!(output.ends_with("theme = \"tokyonight\"\n"));
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

    #[test]
    fn catalog_sources_are_merged_by_logical_name() {
        let catalog = merge_catalogs(&[
            (
                Target::Ghostty,
                r#"
                [tokyo-night]
                name = "TokyoNight"
                "#
                .to_string(),
            ),
            (
                Target::Helix,
                r#"
                [tokyo-night]
                name = "tokyonight"

                [rose-pine]
                name = "rose_pine"
                "#
                .to_string(),
            ),
        ])
        .unwrap();

        let tokyo_night = catalog.get("tokyo-night").unwrap();
        assert_eq!(tokyo_night.ghostty.as_deref(), Some("TokyoNight"));
        assert_eq!(tokyo_night.helix.as_deref(), Some("tokyonight"));
        assert_eq!(tokyo_night.zed.as_deref(), None);

        let rose_pine = catalog.get("rose-pine").unwrap();
        assert_eq!(rose_pine.helix.as_deref(), Some("rose_pine"));
        assert_eq!(rose_pine.ghostty.as_deref(), None);
        assert_eq!(rose_pine.zed.as_deref(), None);
    }
}
