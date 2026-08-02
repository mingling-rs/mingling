#![allow(unused_imports)]

pub use mingling_pathf::config::*;
pub use mingling_pathf::module_pathf::*;
pub use mingling_pathf::pattern_analyzer::*;
pub use mingling_pathf::patterns::*;

use std::path::Path;

/// Wraps `analyze_and_build_type_mapping_for` with config derived from
/// the crate's feature flags (e.g., `dispatch_tree`).
pub fn analyze_and_build_type_mapping_for(
    crate_dir: &Path,
    output_dir: &Path,
) -> Result<(), crate::error::MinglingPathfinderError> {
    let config = mingling_pathf::config::PathfinderConfig {
        use_dispatch_tree: cfg!(feature = "dispatch_tree"),
    };
    mingling_pathf::analyze_and_build_type_mapping_for(crate_dir, output_dir, &config)
}

/// Wraps `analyze_and_build_type_mapping` (build.rs convenience) with config.
pub fn analyze_and_build_type_mapping() -> Result<(), crate::error::MinglingPathfinderError> {
    let config = mingling_pathf::config::PathfinderConfig {
        use_dispatch_tree: cfg!(feature = "dispatch_tree"),
    };
    let crate_dir =
        std::env::current_dir().map_err(crate::error::MinglingPathfinderError::IoError)?;
    let crate_name = std::env::var("CARGO_PKG_NAME").map_err(|_| {
        crate::error::MinglingPathfinderError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "CARGO_PKG_NAME not set",
        ))
    })?;
    let out_dir = std::env::var("OUT_DIR").map_err(|_| {
        crate::error::MinglingPathfinderError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "OUT_DIR not set",
        ))
    })?;
    let output_dir = Path::new(&out_dir).join(&crate_name);
    mingling_pathf::analyze_and_build_type_mapping_for(&crate_dir, &output_dir, &config)?;
    println!("cargo:rerun-if-changed=src/");
    Ok(())
}
