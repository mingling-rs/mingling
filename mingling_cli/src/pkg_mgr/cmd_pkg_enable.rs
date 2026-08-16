use std::{fs, io};

use mingling::{
    Grouped, RenderResult, Routable, ShellContext, Suggest, SuggestItem, Wrap,
    macros::{arg, chain, command, completion, metadata, renderer, routeify},
    metadata::Description,
    picker::{EntryPicker, PickerArg},
};

use crate::{
    Next, eprintln_cargo,
    pkg_mgr::{
        ErrorNoDataDirectory, ErrorPackageNameRequired, ErrorPackageSpecInvalid, ResPackagesDir,
    },
    println_cargo,
};

/// Positional argument: package spec (`foo`, `foo@0`, `foo@0.1`, `foo@0.1.2`)
pub static ARG_SPEC: PickerArg<String> = arg![String];

#[derive(Grouped, Wrap)]
pub struct ErrorNoMatchingVersion(String);

#[derive(Grouped, Wrap)]
pub struct StatePkgEnable((String, String));

#[derive(Debug, Default, Grouped)]
pub struct ResultPkgEnable {
    pub name: String,
    pub version: String,
}

#[metadata(EntryPkgEnable)]
pub fn desc_pkg_enable() -> Description {
    "Enable the specified package".into()
}

#[command(node = "pkg-enable", routeify)]
pub fn package_enable(args: EntryPkgEnable, packages_dir: &ResPackagesDir) -> Next {
    let spec = args
        .pick_or_route(&ARG_SPEC, || ErrorPackageNameRequired.to_chain())
        .to_result()?;
    let packages_dir = &packages_dir.path;
    if packages_dir.as_os_str().is_empty() {
        return ErrorNoDataDirectory.to_chain();
    }
    if spec.contains('/') || spec.contains('\\') || spec.contains("..") {
        return ErrorPackageSpecInvalid(spec).to_chain();
    }

    let (name, version_part) = match spec.split_once('@') {
        Some((name, version)) => (name.to_string(), Some(version.to_string())),
        None => (spec.clone(), None),
    };

    // Collect every installed version matching the spec, pick the newest
    let mut candidates: Vec<(String, semver::Version)> = Vec::new();
    if let Ok(entries) = fs::read_dir(packages_dir) {
        for entry in entries.flatten() {
            if !entry.file_type().is_ok_and(|t| t.is_dir()) {
                continue;
            }
            let Some(dir_name) = entry.file_name().to_str().map(str::to_owned) else {
                continue;
            };
            let Some((dir_pkg, dir_version)) = dir_name.split_once('@') else {
                continue;
            };
            if dir_pkg != name {
                continue;
            }
            if let Some(part) = &version_part
                && !version_matches(dir_version, part)
            {
                continue;
            }
            if let Ok(version) = semver::Version::parse(dir_version) {
                candidates.push((dir_name, version));
            }
        }
    }

    let Some((_, version)) = candidates.into_iter().max_by(|a, b| a.1.cmp(&b.1)) else {
        return ErrorNoMatchingVersion(spec).to_chain();
    };

    StatePkgEnable((name, version.to_string())).to_chain()
}

#[chain(routeify)]
pub fn handle_state_pkg_enable(p: StatePkgEnable, packages_dir: &ResPackagesDir) -> Next {
    let (name, version) = p.0;
    let packages_dir = &packages_dir.path;
    if packages_dir.as_os_str().is_empty() {
        return ErrorNoDataDirectory.to_chain();
    }

    let file = packages_dir.join(&name);
    fs::write(&file, &version).map_err(|e| {
        io::Error::new(e.kind(), format!("failed to write {}: {e}", file.display()))
    })?;

    ResultPkgEnable { name, version }.to_chain()
}

#[renderer]
pub fn render_result_pkg_enable(result: ResultPkgEnable) -> RenderResult {
    let mut r = RenderResult::new();
    println_cargo!(r, "Enabled: {}@{}", result.name, result.version);
    r
}

#[renderer]
pub fn render_error_no_matching_version(err: ErrorNoMatchingVersion) -> RenderResult {
    let mut r = RenderResult::new();
    eprintln_cargo!(r, "no matching version for: {}", err.0);
    r
}

#[completion(EntryPkgEnable)]
pub fn complete_pkg_enable(ctx: ShellContext, packages_dir: &ResPackagesDir) -> Suggest {
    if ctx.previous_word != "pkg-enable" {
        return Suggest::FileCompletion;
    }
    let mut suggest = Suggest::new();
    if let Ok(entries) = fs::read_dir(&packages_dir.path) {
        for entry in entries.flatten() {
            if !entry.file_type().is_ok_and(|t| t.is_dir()) {
                continue;
            }
            if let Some(name) = entry.file_name().to_str() {
                suggest.insert(SuggestItem::Simple(name.to_string()));
            }
        }
    }
    suggest
}

/// Whether `version` falls under the partial spec, compared segment by segment.
/// Pre-release suffixes are ignored during matching, e.g. `0.2.1` matches `0.2.1-rc10`.
fn version_matches(version: &str, partial: &str) -> bool {
    let version = version.split('-').next().unwrap_or(version);
    let version_parts: Vec<&str> = version.split('.').collect();
    let partial_parts: Vec<&str> = partial.split('.').collect();
    if partial_parts.len() > version_parts.len() {
        return false;
    }
    partial_parts
        .iter()
        .zip(version_parts.iter())
        .all(|(p, v)| p == v)
}
