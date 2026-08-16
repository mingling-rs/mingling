use std::{env, fs, io, path::PathBuf, process::Command};

use cargo_metadata::TargetKind;
use mingling::{
    Grouped, LazyRes, RenderResult, Routable, ShellContext, Suggest, Wrap,
    macros::{arg, chain, command, completion, metadata, renderer, routeify, suggest},
    metadata::Description,
    picker::{EntryPicker, PickerArg, value::Flag},
};

use crate::{
    Next, eprintln_cargo,
    metadata::setup::ResMetadata,
    pkg_mgr::{ErrorNoDataDirectory, ErrorRootPackageNotFound, ResPackagesDir},
    println_cargo,
};

#[derive(Grouped, Wrap)]
pub struct ErrorBuildFailed(String);

#[derive(Grouped, Wrap)]
pub struct ErrorBinaryNotFound(String);

#[derive(Grouped, Wrap)]
pub struct ErrorPkgEnableFailed(String);

/// Flag: `--enable` — run `mling pkg-enable` after a successful install
/// to enable the package being installed.
pub static ARG_ENABLE: PickerArg<Flag> = arg![enable: Flag];

/// Resolved install paths, used by the build step.
#[derive(Debug, Default, Grouped)]
pub struct StateInstallBuild {
    pub workspace_root: PathBuf,
    pub install_dir: PathBuf,
    pub release_dir: PathBuf,
    pub exe_suffix: &'static str,
    pub enable: bool,
}

/// State after `cargo build --release`, used by the copy step.
#[derive(Debug, Default, Grouped)]
pub struct StateInstallCopy {
    pub install_dir: PathBuf,
    pub release_dir: PathBuf,
    pub exe_suffix: &'static str,
    pub installed: Vec<PathBuf>,
    pub enable: bool,
}

/// State after the copy step when `--enable` was given: run `mling pkg-enable`.
#[derive(Debug, Default, Grouped)]
pub struct StateInstallEnable {
    pub install_dir: PathBuf,
    pub installed: Vec<PathBuf>,
    pub name: String,
    pub version: String,
}

#[derive(Debug, Default, Grouped)]
pub struct ResultInstall {
    pub install_dir: PathBuf,
    pub installed: Vec<PathBuf>,
}

/// Parse arguments and resolve the install directory:
/// {data_dir}/mingling/packages/{PACKAGE_NAME}@{PACKAGE_VERSION}
#[metadata(EntryInstall)]
pub fn desc_install() -> Description {
    "Install the project to the Mingling package list".into()
}

#[command(routeify)]
pub fn install(
    args: EntryInstall,
    packages_dir: &ResPackagesDir,
    metadata: &mut LazyRes<ResMetadata>,
) -> Next {
    let enable = args.pick(&ARG_ENABLE).to_result()?;
    let metadata = metadata.get_ref().data();
    let packages_dir = &packages_dir.path;
    if packages_dir.as_os_str().is_empty() {
        return ErrorNoDataDirectory.to_chain();
    }

    let root_package = metadata
        .root_package()
        .or_else(|| metadata.workspace_packages().first().copied())
        .ok_or(ErrorRootPackageNotFound)?;

    StateInstallBuild {
        workspace_root: metadata.workspace_root.clone().into_std_path_buf(),
        install_dir: packages_dir.join(format!("{}@{}", root_package.name, root_package.version)),
        release_dir: metadata
            .target_directory
            .join("release")
            .into_std_path_buf(),
        exe_suffix: env::consts::EXE_SUFFIX,
        enable: enable.bool(),
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
        .map_err(|e| ErrorBuildFailed(format!("failed to run `cargo build --release`: {e}")))?;
    if !status.success() {
        return ErrorBuildFailed(format!("`cargo build --release` failed with {status}"))
            .to_chain();
    }

    StateInstallCopy {
        install_dir: state.install_dir,
        release_dir: state.release_dir,
        exe_suffix: state.exe_suffix,
        installed: vec![],
        enable: state.enable,
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
                return ErrorBinaryNotFound(bin_file).to_chain();
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

    if state.enable {
        let root_package = metadata
            .root_package()
            .or_else(|| metadata.workspace_packages().first().copied())
            .ok_or(ErrorRootPackageNotFound)?;
        return StateInstallEnable {
            install_dir: state.install_dir,
            installed: state.installed,
            name: root_package.name.to_string(),
            version: root_package.version.to_string(),
        }
        .to_chain();
    }

    ResultInstall {
        install_dir: state.install_dir,
        installed: state.installed,
    }
    .to_chain()
}

/// Step 3 (optional): enable the installed package via `mling pkg-enable`
/// when `--enable` was given.
#[chain(routeify)]
pub fn handle_state_install_enable(state: StateInstallEnable) -> Next {
    let spec = format!("{}@{}", state.name, state.version);
    let status = Command::new("mling")
        .args(["pkg-enable", &spec])
        .status()
        .map_err(|e| {
            ErrorPkgEnableFailed(format!("failed to run `mling pkg-enable {spec}`: {e}"))
        })?;
    if !status.success() {
        return ErrorPkgEnableFailed(format!("`mling pkg-enable {spec}` failed with {status}"))
            .to_chain();
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
        println_cargo!(r, "Copy: {}", file.display());
    }
    r
}

#[renderer]
pub fn render_error_build_failed(err: ErrorBuildFailed) -> RenderResult {
    let mut r = RenderResult::new();
    eprintln_cargo!(r, "{}", err.0);
    r
}

#[renderer]
pub fn render_error_binary_not_found(err: ErrorBinaryNotFound) -> RenderResult {
    let mut r = RenderResult::new();
    eprintln_cargo!(r, "binary not found: {}", err.0);
    r
}

#[renderer]
pub fn render_error_pkg_enable_failed(err: ErrorPkgEnableFailed) -> RenderResult {
    let mut r = RenderResult::new();
    eprintln_cargo!(r, "{}", err.0);
    r
}

#[completion(EntryInstall)]
pub fn complete_install(ctx: &ShellContext) -> Suggest {
    if ctx.previous_word != "install" {
        return Suggest::FileCompletion;
    }
    suggest! {
        ARG_ENABLE: "Enable the package after installing (runs `mling pkg-enable`)"
    }
}
