# Hue
Hue is a CLI tool which synchronises themes across targets, check `themes.toml` for available mappings!

You can also override the themes file locally for any other themes you have.

Currently, Hue supports the following targets:
- [Ghostty](https://ghostty.org/)
- [Zed](https://zed.dev/)

Please create an issue or PR if you'd like to add support for other targets.

# Installation
## Cargo
`cargo install hue-cli`

## Brew
`brew tap howenyap/tap`

`brew install hue`

# Usage
- `hue current` prints out the current set theme
- `hue list` prints out available themes
- `hue set <theme>` sets a logical theme and synchronises it across targets
- `hue set <theme> --dry-run` shows a diff of the update without applying any changes

# Development
Make sure you have [Rust](https://rust-lang.org/tools/install/) installed.

## Pre-commit Hooks
- Install [prek](https://github.com/j178/prek)
- Install the local Git hooks `prek install`
- Manual run with `prek run`
