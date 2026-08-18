use std::collections::HashMap;
use std::path::{Path, PathBuf};

use mingling::{Program, macros::program_setup};

use crate::ThisProgram;

/// Directories whose manifests are excluded from CI checks.
///
/// Path is relative to the crate source file (`mingling_ci/src/res/`).
const IGNORED_DIRS_FILE: &str = include_str!("../../../.config/ci-ignored-dirs.txt");

/// All `Cargo.toml` manifests the CI will check.
#[derive(Default, Clone)]
pub struct Manifests {
    pub path: Vec<PathBuf>,
    /// Package name -> its manifest path.
    pub package_dirs: HashMap<String, PathBuf>,
}

#[program_setup]
pub fn manifests_setup(p: &mut Program<ThisProgram>) {
    let path = cargo_tomls();
    let package_dirs = path.iter().map(|p| (package_name(p), p.clone())).collect();
    p.with_resource(Manifests { path, package_dirs });
}

/// Recursively collects every `Cargo.toml` under the current directory,
/// skipping the legacy `.run` CI directory and any directory listed in
/// `.config/ci-ignored-dirs.txt`.
#[must_use]
fn cargo_tomls() -> Vec<PathBuf> {
    let ignored = ignored_dirs();
    let mut cargo_tomls = Vec::new();
    let mut dirs = vec![PathBuf::from(".")];
    while let Some(dir) = dirs.pop() {
        if is_ignored(&dir.to_string_lossy(), &ignored) {
            continue;
        }
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    // Skip the legacy `.run` CI directory
                    if path.file_name().and_then(|n| n.to_str()) == Some(".run") {
                        continue;
                    }
                    dirs.push(path);
                } else if path.file_name().and_then(|n| n.to_str()) == Some("Cargo.toml") {
                    cargo_tomls.push(path);
                }
            }
        }
    }
    cargo_tomls
}

/// Parses `.config/ci-ignored-dirs.txt` into directory prefixes:
/// non-empty lines that do not start with `#`, with the trailing `/` stripped
/// (e.g. `./.temp/` → `./.temp`).
fn ignored_dirs() -> Vec<String> {
    IGNORED_DIRS_FILE
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.trim_end_matches('/').to_string())
        .collect()
}

/// Whether `path` (a walk directory, e.g. `./.temp` or `./examples`) is inside
/// one of the ignored directories.
fn is_ignored(path: &str, ignored: &[String]) -> bool {
    ignored.iter().any(|dir| {
        path.strip_prefix(dir.as_str())
            .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'))
    })
}

/// Extracts the package name from a `Cargo.toml`.
///
/// Falls back to the parent directory name (e.g. `mingling_core/Cargo.toml` →
/// `mingling_core`, workspace root → `(root)`), matching the legacy CI.
fn package_name(path: &Path) -> String {
    let fallback = || {
        path.parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("(root)")
            .to_string()
    };

    let Ok(content) = std::fs::read_to_string(path) else {
        return fallback();
    };
    let Ok(toml_value) = content.parse::<toml::Table>() else {
        return fallback();
    };
    toml_value
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .map_or_else(fallback, str::to_string)
}
