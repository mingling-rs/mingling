//! This module initializes the Mingling local repository workspace.
//!
//! It automatically generates necessary files for development.

use serde_json::Value;
use std::env::current_dir;
use std::fs;

#[path = "dev/build/ra_setting.rs"]
mod ra_settings;

fn main() {
    gen_fake_cargo_toml_in_temp_dir();
    gen_rust_analyzer_config_for_editors();
}

/// Generate a fake Cargo workspace to prevent temporary repositories under `.temp/`
/// from finding the root directory
fn gen_fake_cargo_toml_in_temp_dir() {
    fs::write(
        current_dir().unwrap().join(".temp/Cargo.toml"),
        "[workspace]",
    )
    .unwrap();
}

/// Generate Rust Analyzer configuration for editors.
///
/// Copies the editor configuration at `dev/configs/rust-analyzer.json` to each
/// editor's configuration file.
///
/// Supported editors:
/// - `Zed Editor` : ".zed/settings.json"
/// - `VS Code`: ".vscode/settings.json"
fn gen_rust_analyzer_config_for_editors() {
    // Re-run this build script whenever the source config changes.
    println!("cargo:rerun-if-changed=dev/configs/rust-analyzer.json");

    let root = current_dir().unwrap();
    let source = root.join("dev/configs/rust-analyzer.json");
    let Ok(source_content) = fs::read_to_string(&source) else {
        eprintln!(
            "warning: `{}` not found, skip editor config generation",
            source.display()
        );
        return;
    };

    // VS Code uses the same flat `rust-analyzer.*` key format, so copy as-is.
    let vscode_path = root.join(".vscode/settings.json");
    if ra_settings::write_if_changed(&vscode_path, &source_content) {
        eprintln!("generated {}", vscode_path.display());
    }

    // Zed nests the same settings under `lsp.rust-analyzer.initialization_options`.
    let Ok(config) = serde_json::from_str::<Value>(&source_content) else {
        eprintln!(
            "warning: failed to parse `{}` as JSON, skip Zed config",
            source.display()
        );
        return;
    };
    let zed_path = root.join(".zed/settings.json");
    if ra_settings::write_if_changed(
        &zed_path,
        &ra_settings::to_pretty_json(&ra_settings::to_zed_settings(&config)),
    ) {
        eprintln!("generated {}", zed_path.display());
    }
}
