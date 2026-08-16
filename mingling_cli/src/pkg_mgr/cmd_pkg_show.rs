use std::{collections::BTreeMap, fs, io};

use colored::Colorize;
use mingling::{
    Grouped, RenderResult, Routable,
    macros::{buffer, command, metadata, r_println, renderer, routeify},
    metadata::Description,
};

use crate::{
    Next, eprintln_cargo,
    pkg_mgr::{ErrorNoDataDirectory, ResPackagesDir},
};

#[derive(Debug, Default, Clone)]
pub struct PkgShowEntry {
    pub name: String,

    /// Enabled version, from the content of the enable file.
    pub enabled: Option<String>,

    /// Installed versions, newest first.
    pub versions: Vec<String>,
}

#[derive(Debug, Default, Grouped)]
pub struct ResultPkgShow {
    pub packages: Vec<PkgShowEntry>,
}

#[metadata(EntryPkgShow)]
pub fn desc_pkg_show() -> Description {
    "Show locally installed packages".into()
}

#[derive(Grouped, Default)]
pub struct ErrorNoPackagesInstalled;

#[command(node = "pkg-show", routeify)]
pub fn package_show(packages_dir: &ResPackagesDir) -> Next {
    let packages_dir = &packages_dir.path;
    if packages_dir.as_os_str().is_empty() {
        return ErrorNoDataDirectory.to_chain();
    }

    let mut entries: BTreeMap<String, PkgShowEntry> = BTreeMap::new();
    for entry in fs::read_dir(packages_dir).map_err(|e| {
        io::Error::new(
            e.kind(),
            format!("failed to read {}: {e}", packages_dir.display()),
        )
    })? {
        let entry = entry.map_err(|e| {
            io::Error::new(e.kind(), format!("failed to read directory entry: {e}"))
        })?;
        let file_type = entry.file_type().map_err(|e| {
            io::Error::new(e.kind(), format!("failed to inspect directory entry: {e}"))
        })?;
        let name = entry.file_name().to_string_lossy().into_owned();

        if file_type.is_dir() {
            // Installed version directory: `name@version`
            if let Some((pkg, version)) = name.split_once('@') {
                let pkg = pkg.to_string();
                let entry = entries.entry(pkg.clone()).or_default();
                entry.name = pkg;
                entry.versions.push(version.to_string());
            }
        } else if file_type.is_file() {
            // Enable file: `name`, content is the enabled version
            let content = fs::read_to_string(entry.path()).map_err(|e| {
                io::Error::new(
                    e.kind(),
                    format!("failed to read {}: {e}", entry.path().display()),
                )
            })?;
            let entry = entries.entry(name.clone()).or_default();
            entry.name = name;
            entry.enabled = Some(content.trim().to_string());
        }
    }

    for pkg in entries.values_mut() {
        pkg.versions.sort_by(|a, b| compare_versions(b, a));
    }

    let packages: Vec<PkgShowEntry> = entries.into_values().collect();

    if packages.is_empty() {
        return ErrorNoPackagesInstalled.into();
    }

    ResultPkgShow { packages }.to_chain()
}

#[renderer(buffer)]
pub fn render_result_pkg_show(r: ResultPkgShow) {
    for pkg in r.packages {
        if let Some(enabled) = &pkg.enabled {
            r_println!("{}", format!("{} ({})", pkg.name, enabled).bright_cyan());
        } else {
            r_println!("{}", pkg.name);
        }
        for version in pkg.versions {
            r_println!("  {version}");
        }
    }
}

#[renderer]
pub fn render_error_no_packages_installed(_: ErrorNoPackagesInstalled) -> RenderResult {
    let mut r = RenderResult::new();
    eprintln_cargo!(r, "No packages installed");
    r
}

/// Newest first; unparsable versions sort last, compared lexicographically.
fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
    match (semver::Version::parse(a), semver::Version::parse(b)) {
        (Ok(x), Ok(y)) => x.cmp(&y),
        (Ok(_), Err(_)) => std::cmp::Ordering::Greater,
        (Err(_), Ok(_)) => std::cmp::Ordering::Less,
        (Err(_), Err(_)) => a.cmp(b),
    }
}
