use colored::Colorize;
use mingling::{
    Groupped, RenderResult,
    macros::{chain, pack, renderer},
};
use serde::Serialize;
use std::io::Write as _;

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
pub fn render_workspace_directory(prev: ResultWorkspaceDirectory) -> RenderResult {
    let mut result = RenderResult::default();
    writeln!(result, "{}", prev.path.bright_cyan().bold()).ok();
    result
}

#[renderer]
pub fn render_target_directory(prev: ResultTargetDirectory) -> RenderResult {
    let mut result = RenderResult::default();
    writeln!(result, "{}", prev.path.bright_cyan().bold()).ok();
    result
}
