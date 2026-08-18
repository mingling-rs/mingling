use mingling::{Program, macros::program_setup};

use crate::ThisProgram;

/// Manifest that declares the documented feature list.
///
/// Path is relative to the repo root (the CI's working directory).
const FEATURES_MANIFEST: &str = "./mingling/Cargo.toml";

/// The docs.rs feature list of `mingling`, the single source of truth for the
/// feature combinations used by CI checks.
#[derive(Default, Clone)]
pub struct ResFeatureList {
    pub list: Vec<String>,
}

#[program_setup]
pub fn features_setup(p: &mut Program<ThisProgram>) {
    p.with_resource(ResFeatureList {
        list: docs_rs_features(),
    });
}

/// Reads `[package.metadata.docs.rs].features` from `mingling/Cargo.toml`.
#[must_use]
fn docs_rs_features() -> Vec<String> {
    let Ok(content) = std::fs::read_to_string(FEATURES_MANIFEST) else {
        return Vec::new();
    };
    let Ok(toml_value) = content.parse::<toml::Value>() else {
        return Vec::new();
    };
    toml_value
        .get("package")
        .and_then(|p| p.get("metadata"))
        .and_then(|m| m.get("docs"))
        .and_then(|d| d.get("rs"))
        .and_then(|rs| rs.get("features"))
        .and_then(|f| f.as_array())
        .map(|features| {
            features
                .iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}
