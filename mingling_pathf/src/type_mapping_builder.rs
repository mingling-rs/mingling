use std::collections::HashSet;
use std::path::{Path};

use crate::error::MinglingPathfinderError;
use crate::module_pathf;
use crate::pattern_analyzer;

/// Analyzes the Mingling types of the specified crate directory and generates mapping files to the specified output directory.
///
/// `crate_dir` — crate root directory (i.e., the directory containing Cargo.toml)
/// `output_dir` — directory where mapping files will be written
///
/// Mapping file format per line: `TypeName = crate::module::path::TypeName`
pub fn analyze_and_build_type_mapping_for(
    crate_dir: &Path,
    output_dir: &Path,
) -> Result<(), MinglingPathfinderError> {
    let module_mapping = module_pathf::analyze(crate_dir)?;
    let analyzer = pattern_analyzer::init();

    let mut type_mappings: Vec<(String, String)> = Vec::new();

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
            let full_path = if ai.module.is_empty() {
                format!("{}::{}", module_path, ai.item_name)
            } else {
                format!("{}::{}::{}", module_path, ai.module, ai.item_name)
            };
            type_mappings.push((ai.item_name, full_path));
        }
    }

    // Sort by full path (ASCII order)
    type_mappings.sort_by(|a, b| a.1.cmp(&b.1));

    // Deduplicate by type name, keeping the first occurrence
    let mut seen = HashSet::new();
    type_mappings.retain(|(name, _)| seen.insert(name.clone()));

    // Create output directory
    std::fs::create_dir_all(output_dir)?;

    // Write files
    let output_path = output_dir.join("MAPPING");
    let type_using_path = output_dir.join("type_using.rs");

    let mut content_mapping = String::new();
    for (name, path) in &type_mappings {
        content_mapping.push_str(&format!("{name} = {path}\n"));
    }
    std::fs::write(&output_path, content_mapping)?;

    let mut content_using = String::new();
    for (_, path) in &type_mappings {
        content_using.push_str(&format!("use {path};\n"));
    }
    std::fs::write(&type_using_path, content_using)?;

    Ok(())
}

/// Convenience version to be called from `build.rs`, automatically reading configuration
/// from environment variables.
///
/// Reads `CARGO_PKG_NAME` and `OUT_DIR`, and outputs to `{OUT_DIR}/{CARGO_PKG_NAME}/`.
pub fn analyze_and_build_type_mapping() -> Result<(), MinglingPathfinderError> {
    let crate_name = std::env::var("CARGO_PKG_NAME")
        .map_err(|_| MinglingPathfinderError::IoError(
            std::io::Error::new(std::io::ErrorKind::NotFound,
                "CARGO_PKG_NAME not set (not running in build.rs?)")
        ))?;

    let out_dir = std::env::var("OUT_DIR")
        .map_err(|_| MinglingPathfinderError::IoError(
            std::io::Error::new(std::io::ErrorKind::NotFound,
                "OUT_DIR not set (not running in build.rs?)")
        ))?;

    let crate_dir = std::env::current_dir()?;
    let output_dir = Path::new(&out_dir).join(&crate_name);

    analyze_and_build_type_mapping_for(&crate_dir, &output_dir)?;

    // Notify Cargo to re-run build.rs when source files change
    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-env-changed=CARGO_CFG_TARGET_OS");
    println!("cargo:rerun-if-env-changed=CARGO_CFG_TARGET_ARCH");

    Ok(())
}
