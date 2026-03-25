use std::iter::once;
use std::path::Path;

use comfy_table::{Cell, Color, ContentArrangement, Table as DisplayTable, presets::UTF8_FULL};
use console::style;
use similar::TextDiff;

use crate::catalog::ThemeMapping;
use crate::target::Target;

pub fn render_theme_table<'a, I>(current_theme: Option<&str>, themes: I) -> String
where
    I: IntoIterator<Item = (&'a str, &'a ThemeMapping)>,
{
    let mut table = DisplayTable::new();

    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(
            once("theme")
                .chain(Target::ALL.iter().map(Target::label))
                .collect::<Vec<_>>(),
        );

    for (name, mapping) in themes {
        let theme_name = if current_theme == Some(name) {
            format!("{name} (current)")
        } else {
            name.to_string()
        };

        table.add_row(
            once(Cell::new(theme_name))
                .chain(Target::ALL.iter().map(|target| {
                    let present = target.mapped_theme(mapping).is_some();

                    let color = if present { Color::Green } else { Color::Red };
                    let marker = if present { "✓" } else { "✘" };

                    Cell::new(marker).fg(color)
                }))
                .collect::<Vec<_>>(),
        );
    }

    table.to_string()
}

pub fn diff(original: &str, updated: &str, path: &Path) -> String {
    let diff = TextDiff::from_lines(original, updated)
        .unified_diff()
        .context_radius(3)
        .header(&path.display().to_string(), &path.display().to_string())
        .to_string();

    let mut output = String::with_capacity(diff.len());

    for line in diff.split_inclusive('\n') {
        if line.starts_with('-') {
            output.push_str(&format!("{}", style(line).red()));
        } else if line.starts_with('+') {
            output.push_str(&format!("{}", style(line).green()));
        } else {
            output.push_str(line);
        }
    }

    output
}
