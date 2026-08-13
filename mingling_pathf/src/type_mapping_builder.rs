// Doc Not Optimize
//! This module contains the matching patterns for `mingling_pathf`.
//! It provides the core logic for analyzing crate types and generating
//! type mapping files used by the pathfinder system.

use std::collections::HashSet;
use std::fmt::Write as FmtWrite;
use std::path::Path;

use crate::config::PathfinderConfig;
use crate::error::MinglingPathfinderError;
use crate::module_pathf;
use crate::pattern_analyzer;

/// Analyzes the Mingling types of the specified crate directory and generates mapping files to the specified output directory.
///
/// `crate_dir` — crate root directory (i.e., the directory containing Cargo.toml)
/// `output_dir` — directory where mapping files will be written
/// `config` — pathfinder configuration (e.g., [`dispatch_tree`] detection)
///
/// Mapping file format per line: `TypeName = crate::module::path::TypeName`
///
/// # Errors
///
/// Returns a [`MinglingPathfinderError`] if the module analysis fails, the output
/// directory cannot be created, or the mapping files cannot be written.
pub fn analyze_and_build_type_mapping_for(
    crate_dir: &Path,
    output_dir: &Path,
    config: &PathfinderConfig,
) -> Result<(), MinglingPathfinderError> {
    let module_mapping = module_pathf::analyze(crate_dir)?;
    let analyzer = pattern_analyzer::init_with_config(config);

    let mut type_mappings: Vec<(String, String, bool)> = Vec::new();

    for item in module_mapping {
        let file_abs = crate_dir.join(item.file_path());
        if !file_abs.is_file() {
            continue;
        }

        let module_path = item.module_path();
        let Ok(analyze_items) = analyzer.analyze_file_items(&file_abs) else {
            continue;
        };

        for ai in analyze_items {
            let full_path = if ai.is_foreign {
                format!("{}::{}", ai.module, ai.item_name)
            } else if ai.module.is_empty() {
                format!("{}::{}", module_path, ai.item_name)
            } else {
                format!("{}::{}::{}", module_path, ai.module, ai.item_name)
            };
            type_mappings.push((ai.item_name, full_path, ai.is_module));
        }
    }

    // Sort by full path (ASCII order)
    type_mappings.sort_by(|a, b| a.1.cmp(&b.1));

    // Deduplicate by type name, keeping the first occurrence
    let mut seen = HashSet::new();
    type_mappings.retain(|(name, _, _)| seen.insert(name.clone()));

    // Create output directory
    std::fs::create_dir_all(output_dir)?;

    // Write files
    let output_path = output_dir.join("MAPPING");
    let type_using_path = output_dir.join("type_using.rs");

    let mut content_mapping = String::new();
    for (name, path, _) in &type_mappings {
        let _ = writeln!(content_mapping, "{name} = {path}");
    }
    std::fs::write(&output_path, content_mapping)?;

    let mut content_using = String::new();
    for (_, path, is_module) in &type_mappings {
        if *is_module {
            let _ = writeln!(content_using, "use {path}::*;");
        } else {
            let _ = writeln!(content_using, "use {path};");
        }
    }
    std::fs::write(&type_using_path, content_using)?;

    Ok(())
}

/// Convenience version to be called from `build.rs`, automatically reading configuration
/// from environment variables.
///
/// Reads `CARGO_PKG_NAME` and `OUT_DIR`, and outputs to `{OUT_DIR}/{CARGO_PKG_NAME}/`.
///
/// # Errors
///
/// Returns a [`MinglingPathfinderError`] if the required environment variables
/// (`CARGO_PKG_NAME`, `OUT_DIR`) are not set, the current directory cannot be
/// determined, or the type mapping generation fails.
pub fn analyze_and_build_type_mapping() -> Result<(), MinglingPathfinderError> {
    let crate_name = std::env::var("CARGO_PKG_NAME").map_err(|_| {
        MinglingPathfinderError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "CARGO_PKG_NAME not set (not running in build.rs?)",
        ))
    })?;

    let out_dir = std::env::var("OUT_DIR").map_err(|_| {
        MinglingPathfinderError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "OUT_DIR not set (not running in build.rs?)",
        ))
    })?;

    let crate_dir = std::env::current_dir()?;
    let output_dir = Path::new(&out_dir).join(&crate_name);

    analyze_and_build_type_mapping_for(&crate_dir, &output_dir, &PathfinderConfig::default())?;

    // Notify Cargo to re-run build.rs when source files change
    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-env-changed=CARGO_CFG_TARGET_OS");
    println!("cargo:rerun-if-env-changed=CARGO_CFG_TARGET_ARCH");

    Ok(())
}
