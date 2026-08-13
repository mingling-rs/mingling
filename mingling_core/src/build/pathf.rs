#![allow(unused_imports)]

pub use mingling_pathf::config::*;
pub use mingling_pathf::module_pathf::*;
pub use mingling_pathf::pattern_analyzer::*;
pub use mingling_pathf::patterns::*;

use std::path::Path;

/// Analyzes and builds a type mapping for a specific crate.
///
/// Accepts `crate_dir` and `output_dir`, and invokes `pathf` to build the type mapping.
///
/// # Arguments
///
/// - `crate_dir`: Root directory of the crate's source code to analyze (usually `CARGO_MANIFEST_DIR`).
/// - `output_dir`: Output directory for generated artifacts (type mapping data).
///
/// # Returns
///
/// - On success: returns `Ok(())`;
/// - On failure: returns the corresponding `MinglingPathfinderError`.
///
/// # Example
///
/// ```
/// # #[cfg(all(feature = "build", feature = "pathf"))] {
/// use mingling_core::build::analyze_and_build_type_mapping_for;
/// use std::path::Path;
///
/// let crate_dir = Path::new(".");
/// let output_dir = Path::new(".temp/target/out");
/// analyze_and_build_type_mapping_for(crate_dir, output_dir).expect("analysis failed");
/// # }
/// ```
pub fn analyze_and_build_type_mapping_for(
    crate_dir: &Path,
    output_dir: &Path,
) -> Result<(), crate::error::MinglingPathfinderError> {
    let config = mingling_pathf::config::PathfinderConfig {
        use_dispatch_tree: cfg!(feature = "dispatch_tree"),
    };
    mingling_pathf::analyze_and_build_type_mapping_for(crate_dir, output_dir, &config)
}

/// # Analyzes and builds a type mapping
///
/// This function reads the current crate directory (`CARGO_PKG_NAME`) and output directory (`OUT_DIR`)
/// from environment variables, automatically combines them into the target output path, and invokes
/// the underlying analysis logic. Suitable for use in `build.rs`.
///
/// It also sends the `cargo:rerun-if-changed=src/` directive to Cargo so that a rebuild is
/// automatically triggered when source code changes.
///
/// # Prerequisites
///
/// This function depends on the following environment variables, which are typically set
/// automatically during a Cargo build:
///
/// - `CARGO_PKG_NAME`: Name of the current crate.
/// - `OUT_DIR`: Build output directory provided by Cargo.
///
/// If these variables are missing, a corresponding [`MinglingPathfinderError`](crate::error::MinglingPathfinderError)
/// is returned.
///
/// # Returns
///
/// Returns `Ok(())` on success; returns a corresponding
/// [`MinglingPathfinderError`](crate::error::MinglingPathfinderError) on failure.
///
/// # Example
///
/// ```
/// # #[cfg(all(feature = "build", feature = "pathf"))] {
/// use mingling_core::build::analyze_and_build_type_mapping;
///
/// fn main() {
///     analyze_and_build_type_mapping().expect("failed to build type mapping");
/// }
/// # }
/// ```

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
