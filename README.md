# Hue
Hue is a CLI tool which synchronises themes across targets, check `themes.toml` for available mappings!

You can also override the themes file locally for any other themes you have.

Currently, Hue supports the following targets:
- [Ghostty](https://ghostty.org/)
- [Zed](https://zed.dev/)

Please create an issue or PR if you'd like to add support for other targets.

# Installation
`cargo install hue-cli`

# Usage
- `hue current` prints out the current set theme
- `hue list` prints out available themes
- `hue set <theme>` sets a logical theme and synchronises it across targets
- `hue set <theme> --dry-run` shows a diff of the update without applying any changes
