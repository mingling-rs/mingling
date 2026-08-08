use std::{env, fs, io, path::PathBuf, process::Command};

use cargo_metadata::TargetKind;
use mingling::{
    Grouped, LazyRes, RenderResult, Routable,
    macros::{chain, command, metadata, pack_err, r_println, renderer, routeify},
    metadata::Description,
};

use crate::{
    Next, eprintln_cargo,
    metadata::setup::ResMetadata,
    pkg_mgr::{ErrorNoDataDirectory, ErrorRootPackageNotFound, ResPackagesDir},
    println_cargo,
};

pack_err!(ErrorBuildFailed = String);
pack_err!(ErrorBinaryNotFound = String);

/// Resolved install paths, used by the build step.
#[derive(Debug, Default, Grouped)]
pub struct StateInstallBuild {
    pub workspace_root: PathBuf,
    pub install_dir: PathBuf,
    pub release_dir: PathBuf,
    pub exe_suffix: &'static str,
}

/// State after `cargo build --release`, used by the copy step.
#[derive(Debug, Default, Grouped)]
pub struct StateInstallCopy {
    pub install_dir: PathBuf,
    pub release_dir: PathBuf,
    pub exe_suffix: &'static str,
    pub installed: Vec<PathBuf>,
}

#[derive(Debug, Default, Grouped)]
pub struct ResultInstall {
    pub install_dir: PathBuf,
    pub installed: Vec<PathBuf>,
}

/// Parse arguments and resolve the install directory:
/// {data_dir}/.mingling/{PACKAGE_NAME}@{PACKAGE_VERSION}
#[metadata(EntryInstall)]
pub fn desc_install() -> Description {
    "Install the project to the Mingling package list".into()
}

#[command(routeify)]
pub fn install(packages_dir: &ResPackagesDir, metadata: &mut LazyRes<ResMetadata>) -> Next {
    let metadata = metadata.get_ref().data();
    let packages_dir = &packages_dir.path;
    if packages_dir.as_os_str().is_empty() {
        return ErrorNoDataDirectory::default().to_chain();
    }

    let root_package = metadata
        .root_package()
        .or_else(|| metadata.workspace_packages().first().copied())
        .ok_or(ErrorRootPackageNotFound::default())?;

    StateInstallBuild {
        workspace_root: metadata.workspace_root.clone().into_std_path_buf(),
        install_dir: packages_dir.join(format!("{}@{}", root_package.name, root_package.version)),
        release_dir: metadata
            .target_directory
            .join("release")
            .into_std_path_buf(),
        exe_suffix: env::consts::EXE_SUFFIX,
    }
    .to_chain()
}

/// Step 1: build release binaries.
#[chain(routeify)]
pub fn handle_state_install_build(state: StateInstallBuild) -> Next {
    let status = Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(&state.workspace_root)
        .status()
        .map_err(|e| {
            ErrorBuildFailed::new(format!("failed to run `cargo build --release`: {e}"))
        })?;
    if !status.success() {
        return ErrorBuildFailed::new(format!("`cargo build --release` failed with {status}"))
            .to_chain();
    }

    StateInstallCopy {
        install_dir: state.install_dir,
        release_dir: state.release_dir,
        exe_suffix: state.exe_suffix,
        installed: vec![],
    }
    .to_chain()
}

/// Step 2: copy binaries and completion scripts.
#[chain(routeify)]
pub fn handle_state_install_copy(
    mut state: StateInstallCopy,
    metadata: &mut LazyRes<ResMetadata>,
) -> Next {
    let metadata = metadata.get_ref().data();
    fs::create_dir_all(&state.install_dir).map_err(|e| {
        io::Error::new(
            e.kind(),
            format!("failed to create {}: {e}", state.install_dir.display()),
        )
    })?;

    for package in metadata.workspace_packages() {
        let bin_targets: Vec<_> = package
            .targets
            .iter()
            .filter(|target| target.kind.contains(&TargetKind::Bin))
            .collect();

        for target in bin_targets {
            let bin_file = format!("{}{}", target.name, state.exe_suffix);
            let src = state.release_dir.join(&bin_file);
            if !src.is_file() {
                return ErrorBinaryNotFound::new(bin_file).to_chain();
            }
            let dst = state.install_dir.join(&bin_file);
            fs::copy(&src, &dst).map_err(|e| {
                io::Error::new(e.kind(), format!("failed to copy {}: {e}", src.display()))
            })?;
            state.installed.push(dst);
        }
    }

    // Completion scripts are generated into the build profile directory
    // (OUT_DIR/../../../), copy every one whose name contains `_comp`,
    // regardless of its suffix
    for entry in fs::read_dir(&state.release_dir).map_err(|e| {
        io::Error::new(
            e.kind(),
            format!("failed to read {}: {e}", state.release_dir.display()),
        )
    })? {
        let entry = entry.map_err(|e| {
            io::Error::new(e.kind(), format!("failed to read directory entry: {e}"))
        })?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.contains("_comp") {
            let dst = state.install_dir.join(&name);
            fs::copy(entry.path(), &dst)
                .map_err(|e| io::Error::new(e.kind(), format!("failed to copy {name}: {e}")))?;
            state.installed.push(dst);
        }
    }

    ResultInstall {
        install_dir: state.install_dir,
        installed: state.installed,
    }
    .to_chain()
}

#[renderer]
pub fn render_result_install(result: ResultInstall) -> RenderResult {
    let mut r = RenderResult::new();
    println_cargo!(r, "Installed: {}", result.install_dir.display());
    for file in result.installed {
        r_println!(r, "  {}", file.display());
    }
    r
}

#[renderer]
pub fn render_error_build_failed(err: ErrorBuildFailed) -> RenderResult {
    let mut r = RenderResult::new();
    eprintln_cargo!(r, "{}", err.info);
    r
}

#[renderer]
pub fn render_error_binary_not_found(err: ErrorBinaryNotFound) -> RenderResult {
    let mut r = RenderResult::new();
    eprintln_cargo!(r, "binary not found: {}", err.info);
    r
}
