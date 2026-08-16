pub mod cmd_install;
pub mod cmd_internal_loadpkgs;
pub mod cmd_pkg_disable;
pub mod cmd_pkg_enable;
pub mod cmd_pkg_show;
pub mod cmd_uninstall;

use std::path::PathBuf;

use mingling::{
    Grouped, Program, RenderResult, Wrap,
    macros::{program_setup, r_println, renderer},
};

use crate::{ThisProgram, eprintln_cargo, hprintln_cargo};

#[derive(Grouped, Default)]
pub struct ErrorRootPackageNotFound;

#[derive(Grouped, Default)]
pub struct ErrorNoDataDirectory;

#[derive(Grouped, Wrap)]
pub struct ErrorPackageSpecInvalid(String);

#[derive(Grouped, Default)]
pub struct ErrorPackageNameRequired;

/// The `mingling/packages` packages directory under the user's data directory.
#[derive(Debug, Default, Clone)]
pub struct ResPackagesDir {
    pub path: PathBuf,
}

#[program_setup]
pub fn package_manager_setup(program: &mut Program<ThisProgram>) {
    let path = dirs::data_dir()
        .map(|data_dir| data_dir.join("mingling").join("packages"))
        .unwrap_or_default();
    program.with_resource(ResPackagesDir { path });
}

#[renderer]
pub fn render_error_root_package_not_found(_: ErrorRootPackageNotFound) -> RenderResult {
    let mut r = RenderResult::new();
    eprintln_cargo!(r, "failed to determine the root package");
    r_println!(r, "");
    hprintln_cargo!(
        r,
        "Run `mling install` / `mling uninstall` inside a Cargo workspace"
    );
    r
}

#[renderer]
pub fn render_error_no_data_directory(_: ErrorNoDataDirectory) -> RenderResult {
    let mut r = RenderResult::new();
    eprintln_cargo!(r, "failed to determine the data directory");
    r
}

#[renderer]
pub fn render_error_package_spec_invalid(err: ErrorPackageSpecInvalid) -> RenderResult {
    let mut r = RenderResult::new();
    eprintln_cargo!(r, "invalid package spec: {}", err.0);
    r
}

#[renderer]
pub fn render_error_package_name_required(_: ErrorPackageNameRequired) -> RenderResult {
    let mut r = RenderResult::new();
    eprintln_cargo!(r, "a package name is required");
    r
}
