pub mod cmd_install;
pub mod cmd_internal_loadpkgs;
pub mod cmd_pkg_disable;
pub mod cmd_pkg_enable;
pub mod cmd_pkg_show;
pub mod cmd_uninstall;

use std::path::PathBuf;

use mingling::{
    Program, RenderResult,
    macros::{pack_err, program_setup, r_println, renderer},
};

use crate::{ThisProgram, eprintln_cargo, hprintln_cargo};

pack_err!(ErrorRootPackageNotFound);
pack_err!(ErrorNoDataDirectory);
pack_err!(ErrorPackageSpecInvalid = String);
pack_err!(ErrorPackageNameRequired);

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
    eprintln_cargo!(r, "invalid package spec: {}", err.info);
    r
}

#[renderer]
pub fn render_error_package_name_required(_: ErrorPackageNameRequired) -> RenderResult {
    let mut r = RenderResult::new();
    eprintln_cargo!(r, "a package name is required");
    r
}
