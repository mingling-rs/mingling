pub mod cmd_install;
pub mod cmd_uninstall;

use std::path::PathBuf;

use mingling::{
    Program,
    macros::{buffer, pack_err, program_setup, r_println, renderer},
};

use crate::ThisProgram;

pack_err!(ErrorRootPackageNotFound);
pack_err!(ErrorNoDataDirectory);

/// The `.mingling` packages directory under the user's data directory.
#[derive(Debug, Default, Clone)]
pub struct ResPackagesDir {
    pub path: PathBuf,
}

#[program_setup]
pub fn package_manager_setup(program: &mut Program<ThisProgram>) {
    let path = dirs::data_dir()
        .map(|data_dir| data_dir.join(".mingling"))
        .unwrap_or_default();
    program.with_resource(ResPackagesDir { path });
}

#[renderer(buffer)]
pub fn render_error_root_package_not_found(_: ErrorRootPackageNotFound) {
    r_println!("error: failed to determine the root package");
    r_println!("");
    r_println!("Run `mling install` / `mling uninstall` inside a Cargo workspace");
}

#[renderer(buffer)]
pub fn render_error_no_data_directory(_: ErrorNoDataDirectory) {
    r_println!("error: failed to determine the data directory");
}
