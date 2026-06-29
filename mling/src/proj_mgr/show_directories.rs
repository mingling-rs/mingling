use colored::Colorize;
use mingling::{
    Groupped,
    macros::{chain, pack, r_println, renderer},
};
use serde::Serialize;

use crate::{
    Next,
    proj_mgr::{EntryShowTargetDirectories, EntryShowWorkspaceDirectory, metadata::read_metadata},
    res::ResManifestPath,
};

#[derive(Serialize, Groupped)]
pub struct ResultWorkspaceDirectory {
    pub path: String,
}

#[derive(Serialize, Groupped)]
pub struct ResultTargetDirectory {
    pub path: String,
}

#[chain]
pub fn handle_show_workspace_directory(
    _args: EntryShowWorkspaceDirectory,
    manifest_path: &ResManifestPath,
) -> Next {
    let metadata = read_metadata(manifest_path.resolved()).unwrap();
    ResultWorkspaceDirectory {
        path: metadata.workspace_root,
    }
    .to_render()
}

#[chain]
pub fn handle_show_target_directory(
    _args: EntryShowTargetDirectories,
    manifest_path: &ResManifestPath,
) -> Next {
    let metadata = read_metadata(manifest_path.resolved()).unwrap();
    ResultTargetDirectory {
        path: metadata.target_directory,
    }
    .to_render()
}

#[renderer]
pub fn render_workspace_directory(prev: ResultWorkspaceDirectory) {
    r_println!("{}", prev.path.bright_cyan().bold());
}

#[renderer]
pub fn render_target_directory(prev: ResultTargetDirectory) {
    r_println!("{}", prev.path.bright_cyan().bold());
}
