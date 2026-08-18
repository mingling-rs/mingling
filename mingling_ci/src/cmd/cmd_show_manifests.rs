use std::path::PathBuf;

use mingling::{
    Grouped,
    macros::{buffer, command, r_println, renderer},
};

use prettytable::{
    Cell, Row, Table,
    format::{FormatBuilder, LinePosition, LineSeparator},
};

use crate::res::{Manifests, package_name};

#[command(node = "show-manifests")]
pub fn show_manifests(manifests: &Manifests) -> ResultPrintManifests {
    let mut entries: Vec<ManifestEntry> = manifests
        .path
        .iter()
        .map(|path| ManifestEntry {
            name: package_name(path),
            path: path.clone(),
        })
        .collect();
    entries.sort_by(|a, b| a.path.cmp(&b.path));
    ResultPrintManifests { entries }
}

/// All manifests the CI will check, sorted by path.
#[derive(Grouped)]
pub struct ResultPrintManifests {
    pub entries: Vec<ManifestEntry>,
}

#[derive(Debug, Clone)]
pub struct ManifestEntry {
    pub name: String,
    pub path: PathBuf,
}

#[renderer(buffer)]
pub fn render_print_manifests(r: ResultPrintManifests) {
    let mut table = Table::new();

    table.set_format(
        FormatBuilder::new()
            .column_separator('│')
            .borders('│')
            .separator(LinePosition::Top, LineSeparator::new('─', '┬', '┌', '┐'))
            .separator(LinePosition::Title, LineSeparator::new('─', '┼', '├', '┤'))
            .separator(LinePosition::Bottom, LineSeparator::new('─', '┴', '└', '┘'))
            .padding(1, 1)
            .build(),
    );

    table.set_titles(Row::new(vec![
        Cell::new("#"),
        Cell::new("Package-Name"),
        Cell::new("Package-Path"),
    ]));

    for (index, entry) in r.entries.iter().enumerate() {
        table.add_row(Row::new(vec![
            Cell::new(&(index + 1).to_string()),
            Cell::new(&entry.name),
            Cell::new(&entry.path.to_string_lossy()),
        ]));
    }

    r_println!("{table}");
}
