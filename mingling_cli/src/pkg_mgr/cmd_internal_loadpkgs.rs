use std::{
    fs, io,
    path::{Path, PathBuf},
};

use mingling::{
    Grouped, Routable, Wrap,
    macros::{buffer, command, r_println, renderer, routeify},
};

use crate::{
    Next,
    pkg_mgr::{ErrorNoDataDirectory, ResPackagesDir},
};

// Version directory paths of every enabled package.
#[derive(Grouped, Wrap)]
pub struct ResultLoadPkgsPaths(Vec<PathBuf>);

// Completion script paths of every enabled package.
#[derive(Grouped, Wrap)]
pub struct ResultLoadPkgsComps(Vec<PathBuf>);

#[command(node = "__loadpkgs_path", routeify)]
pub fn load_packages_paths(packages_dir: &ResPackagesDir) -> Next {
    let packages_dir = &packages_dir.path;
    if packages_dir.as_os_str().is_empty() {
        return ErrorNoDataDirectory.to_chain();
    }
    let paths = enabled_version_dirs(packages_dir).map_err(|e| {
        io::Error::new(
            e.kind(),
            format!("failed to read {}: {e}", packages_dir.display()),
        )
    })?;
    ResultLoadPkgsPaths(paths).to_chain()
}

#[command(node = "__loadpkgs_comp_scripts", routeify)]
pub fn load_packages_comp_scripts(packages_dir: &ResPackagesDir) -> Next {
    let packages_dir = &packages_dir.path;
    if packages_dir.as_os_str().is_empty() {
        return ErrorNoDataDirectory.to_chain();
    }
    let scripts = comp_scripts(packages_dir).map_err(|e| {
        io::Error::new(
            e.kind(),
            format!("failed to read {}: {e}", packages_dir.display()),
        )
    })?;
    ResultLoadPkgsComps(scripts).to_chain()
}

#[renderer(buffer)]
pub fn render_result_load_pkgs_paths(r: ResultLoadPkgsPaths) {
    for path in r.0 {
        r_println!("{}", path.display());
    }
}

#[renderer(buffer)]
pub fn render_result_load_pkgs_comps(r: ResultLoadPkgsComps) {
    for path in r.0 {
        r_println!("{}", path.display());
    }
}

/// Version directories of every enabled package.
///
/// An enable file is a `name` file (no `@`) whose content is the enabled version.
fn enabled_version_dirs(packages_dir: &Path) -> Result<Vec<PathBuf>, io::Error> {
    let mut dirs = Vec::new();
    for entry in fs::read_dir(packages_dir)? {
        let entry = entry?;
        if !entry.file_type().is_ok_and(|t| t.is_file()) {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.contains('@') {
            continue;
        }
        let version = fs::read_to_string(entry.path())?;
        dirs.push(packages_dir.join(format!("{name}@{}", version.trim())));
    }
    dirs.sort();
    Ok(dirs)
}

/// Completion scripts inside every enabled version directory.
fn comp_scripts(packages_dir: &Path) -> Result<Vec<PathBuf>, io::Error> {
    let mut scripts = Vec::new();
    for dir in enabled_version_dirs(packages_dir)? {
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.contains("_comp") {
                scripts.push(entry.path());
            }
        }
    }
    scripts.sort();
    Ok(scripts)
}
