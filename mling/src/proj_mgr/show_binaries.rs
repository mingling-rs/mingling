use std::path::PathBuf;

use colored::Colorize;
use mingling::{
    Groupped,
    macros::{chain, pack, r_print, r_println, renderer},
};
use serde::Serialize;

use crate::{
    Next,
    proj_mgr::{
        EntryShowBinaries,
        metadata::{CargoLockFile, read_metadata},
    },
    res::ResManifestPath,
};

#[derive(Serialize, Groupped)]
pub struct ResultBinaries {
    pub binaries: Vec<DataBinary>,
}

#[derive(Serialize)]
pub struct DataBinary {
    pub name: String,
    pub path: PathBuf,
}

#[chain]
pub fn handle_show_binaries(_args: EntryShowBinaries, manifest_path: &ResManifestPath) -> Next {
    let metadata = read_metadata(manifest_path.resolved()).unwrap();
    let CargoLockFile {
        packages,
        workspace_members,
        ..
    } = metadata;

    let binaries: Vec<DataBinary> = packages
        .into_iter()
        .filter(|pkg| workspace_members.contains(&pkg.id))
        .flat_map(|pkg| {
            pkg.targets
                .into_iter()
                .filter(|target| target.kind.iter().any(|k| k == "bin"))
                .map(move |target| DataBinary {
                    name: target.name,
                    path: PathBuf::from(pkg.manifest_path.clone())
                        .parent()
                        .unwrap_or(&PathBuf::from("."))
                        .join("src")
                        .join(&target.src_path),
                })
        })
        .collect();

    ResultBinaries { binaries }.to_render()
}

#[renderer]
pub fn render_binaries(binaries: ResultBinaries) -> String {
    r_println!("{}", "Binaries:".bright_cyan().bold());
    if let Some(max_name_len) = binaries.binaries.iter().map(|b| b.name.len()).max() {
        for binary in &binaries.binaries {
            r_println!(
                "  {:width$} `{}`",
                binary.name.bright_yellow().bold(),
                binary.path.display().to_string().italic(),
                width = max_name_len
            );
        }
    }
}
