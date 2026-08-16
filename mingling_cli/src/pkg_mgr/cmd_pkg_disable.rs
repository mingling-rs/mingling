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

/// Positional argument: package name
pub static ARG_NAME: PickerArg<String> = arg![String];

#[derive(Grouped, Wrap)]
pub struct ErrorPackageNotEnabled(String);

// The name of the package to disable
#[derive(Grouped, Wrap)]
pub struct StatePkgDisable(String);

#[derive(Debug, Default, Grouped)]
pub struct ResultPkgDisable {
    pub name: String,
    pub removed: bool,
}

#[metadata(EntryPkgDisable)]
pub fn desc_pkg_disable() -> Description {
    "Disable the specified package".into()
}

#[command(node = "pkg-disable", routeify)]
pub fn package_disable(args: EntryPkgDisable, packages_dir: &ResPackagesDir) -> Next {
    let name = args
        .pick_or_route(&ARG_NAME, || ErrorPackageNameRequired.to_chain())
        .to_result()?;
    let packages_dir = &packages_dir.path;
    if packages_dir.as_os_str().is_empty() {
        return ErrorNoDataDirectory.to_chain();
    }
    if name.contains('/') || name.contains('\\') || name.contains("..") || name.contains('@') {
        return ErrorPackageSpecInvalid(name).to_chain();
    }

    StatePkgDisable(name).to_chain()
}

#[chain(routeify)]
pub fn handle_state_pkg_disable(p: StatePkgDisable, packages_dir: &ResPackagesDir) -> Next {
    let name = p.0;
    let packages_dir = &packages_dir.path;
    if packages_dir.as_os_str().is_empty() {
        return ErrorNoDataDirectory.to_chain();
    }

    let file = packages_dir.join(&name);
    if !file.is_file() {
        return ErrorPackageNotEnabled(name).to_chain();
    }
    fs::remove_file(&file).map_err(|e| {
        io::Error::new(
            e.kind(),
            format!("failed to remove {}: {e}", file.display()),
        )
    })?;

    ResultPkgDisable {
        name,
        removed: true,
    }
    .to_chain()
}

#[renderer]
pub fn render_result_pkg_disable(result: ResultPkgDisable) -> RenderResult {
    let mut r = RenderResult::new();
    println_cargo!(r, "Disabled: {}", result.name);
    r
}

#[renderer]
pub fn render_error_package_not_enabled(err: ErrorPackageNotEnabled) -> RenderResult {
    let mut r = RenderResult::new();
    eprintln_cargo!(r, "package is not enabled: {}", err.0);
    r
}

#[completion(EntryPkgDisable)]
pub fn complete_pkg_disable(ctx: &ShellContext, packages_dir: &ResPackagesDir) -> Suggest {
    if ctx.previous_word != "pkg-disable" {
        return Suggest::FileCompletion;
    }
    let mut suggest = Suggest::new();
    if let Ok(entries) = fs::read_dir(&packages_dir.path) {
        for entry in entries.flatten() {
            if !entry.file_type().is_ok_and(|t| t.is_file()) {
                continue;
            }
            if let Some(name) = entry.file_name().to_str() {
                suggest.insert(SuggestItem::Simple(name.to_string()));
            }
        }
    }
    suggest
}
