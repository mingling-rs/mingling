use std::{fs, io, path::PathBuf};

use mingling::{
    Grouped, LazyRes, Routable, ShellContext, Suggest, SuggestItem,
    macros::{
        arg, buffer, chain, command, completion, metadata, pack, r_println, renderer, routeify,
    },
    metadata::Description,
    picker::{EntryPicker, PickerArg},
};

use crate::{
    Next,
    metadata::setup::ResMetadata,
    pkg_mgr::{
        ErrorNoDataDirectory, ErrorPackageSpecInvalid, ErrorRootPackageNotFound, ResPackagesDir,
    },
};

/// Optional positional argument: package spec (`name` or `name@version`)
pub static ARG_PACKAGE: PickerArg<Option<String>> = arg![Option<String>];

pack!(StateUninstallPackages = Vec<String>);

#[derive(Debug, Default, Grouped)]
pub struct ResultUninstall {
    pub removed: Vec<PathBuf>,
    pub not_installed: Vec<PathBuf>,
}

/// `{data_dir}/.mingling`
#[metadata(EntryUninstall)]
pub fn desc_uninstall() -> Description {
    "Uninstall the project from the Mingling package list".into()
}

#[command(routeify)]
pub fn uninstall(
    args: EntryUninstall,
    packages_dir: &ResPackagesDir,
    metadata: &mut LazyRes<ResMetadata>,
) -> Next {
    let spec = args.pick(&ARG_PACKAGE).to_result()?;
    let packages_dir = &packages_dir.path;
    if packages_dir.as_os_str().is_empty() {
        return ErrorNoDataDirectory::default().to_chain();
    }

    let targets = match spec {
        // No spec: uninstall the version installed by the current project
        None => {
            let metadata = metadata.get_ref().data();
            let root_package = metadata
                .root_package()
                .or_else(|| metadata.workspace_packages().first().copied())
                .ok_or(ErrorRootPackageNotFound::default())?;
            vec![format!("{}@{}", root_package.name, root_package.version)]
        }
        // `name` matches every installed version, `name@version` matches exactly
        Some(spec) => {
            if spec.contains('/') || spec.contains('\\') || spec.contains("..") {
                return ErrorPackageSpecInvalid::new(spec).to_chain();
            }
            if spec.contains('@') {
                vec![spec]
            } else {
                let prefix = format!("{spec}@");
                if !packages_dir.is_dir() {
                    Vec::new()
                } else {
                    fs::read_dir(packages_dir)
                        .map_err(|e| {
                            io::Error::new(
                                e.kind(),
                                format!("failed to read {}: {e}", packages_dir.display()),
                            )
                        })?
                        .filter_map(|e| e.ok())
                        .filter_map(|e| e.file_name().into_string().ok())
                        .filter(|name| name.starts_with(&prefix))
                        .collect()
                }
            }
        }
    };

    StateUninstallPackages::new(targets).to_chain()
}

#[chain(routeify)]
pub fn handle_state_uninstall_packages(
    p: StateUninstallPackages,
    packages_dir: &ResPackagesDir,
) -> Next {
    let packages_dir = &packages_dir.path;
    if packages_dir.as_os_str().is_empty() {
        return ErrorNoDataDirectory::default().to_chain();
    }
    let mut removed = Vec::new();
    let mut not_installed = Vec::new();

    for name in p.inner {
        let dir = packages_dir.join(&name);
        if !dir.exists() {
            not_installed.push(dir);
            continue;
        }
        fs::remove_dir_all(&dir).map_err(|e| {
            io::Error::new(e.kind(), format!("failed to remove {}: {e}", dir.display()))
        })?;
        removed.push(dir);
    }

    ResultUninstall {
        removed,
        not_installed,
    }
    .to_chain()
}

#[renderer(buffer)]
pub fn render_result_uninstall(r: ResultUninstall) {
    if r.removed.is_empty() && r.not_installed.is_empty() {
        r_println!("No matching packages installed");
    } else {
        for dir in r.removed {
            r_println!("Uninstalled: {}", dir.display());
        }
        for dir in r.not_installed {
            r_println!("Not installed: {}", dir.display());
        }
    }
}

#[completion(EntryUninstall)]
pub fn complete_uninstall(ctx: &ShellContext, packages_dir: &ResPackagesDir) -> Suggest {
    if ctx.previous_word != "uninstall" {
        return Suggest::FileCompletion;
    }
    let mut suggest = Suggest::new();
    if let Ok(entries) = fs::read_dir(&packages_dir.path) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                suggest.insert(SuggestItem::Simple(name.to_string()));
            }
        }
    }
    suggest
}
