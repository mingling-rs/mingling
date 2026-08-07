use std::{fs, io};

use mingling::{
    Grouped, Routable, ShellContext, Suggest, SuggestItem,
    macros::{
        arg, buffer, chain, command, completion, metadata, pack, pack_err, r_println, renderer,
        routeify,
    },
    metadata::Description,
    picker::{EntryPicker, PickerArg},
};

use crate::{
    Next,
    pkg_mgr::{
        ErrorNoDataDirectory, ErrorPackageNameRequired, ErrorPackageSpecInvalid, ResPackagesDir,
    },
};

/// Positional argument: package name
pub static ARG_NAME: PickerArg<String> = arg![String];

pack_err!(ErrorPackageNotEnabled = String);

// The name of the package to disable
pack!(StatePkgDisable = String);

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
        .pick_or_route(&ARG_NAME, || ErrorPackageNameRequired::default().to_chain())
        .to_result()?;
    let packages_dir = &packages_dir.path;
    if packages_dir.as_os_str().is_empty() {
        return ErrorNoDataDirectory::default().to_chain();
    }
    if name.contains('/') || name.contains('\\') || name.contains("..") || name.contains('@') {
        return ErrorPackageSpecInvalid::new(name).to_chain();
    }

    StatePkgDisable::new(name).to_chain()
}

#[chain(routeify)]
pub fn handle_state_pkg_disable(p: StatePkgDisable, packages_dir: &ResPackagesDir) -> Next {
    let name = p.inner;
    let packages_dir = &packages_dir.path;
    if packages_dir.as_os_str().is_empty() {
        return ErrorNoDataDirectory::default().to_chain();
    }

    let file = packages_dir.join(&name);
    if !file.is_file() {
        return ErrorPackageNotEnabled::new(name).to_chain();
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

#[renderer(buffer)]
pub fn render_result_pkg_disable(r: ResultPkgDisable) {
    r_println!("Disabled {}", r.name);
}

#[renderer(buffer)]
pub fn render_error_package_not_enabled(err: ErrorPackageNotEnabled) {
    r_println!("error: package is not enabled: {}", err.info);
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
