use std::path::PathBuf;

use mingling_pathf::error::MinglingPathfinderError;

/// The directory where pathf's build artifacts are stored for the current
/// crate: `{target_directory}/mingling/{CARGO_PKG_NAME}`.
pub fn output_dir() -> Result<PathBuf, MinglingPathfinderError> {
    Ok(mingling_pathf::build_output_dir()?.join(crate_name()))
}

/// Runs the pathf type-mapping analysis for the current crate at compile time
/// (replacing the previous `build.rs` call).
pub fn analyze_and_build_type_mapping() -> Result<(), MinglingPathfinderError> {
    mingling_pathf::analyze_and_build_type_mapping()
}

fn crate_name() -> String {
    std::env::var("CARGO_PKG_NAME").unwrap_or_default()
}
