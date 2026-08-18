//! This module initializes the Mingling local repository workspace.
//!
//! It automatically generates necessary files for development.

use serde_json::Value;
use std::env::current_dir;
use std::fs;

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

/// Internal helpers for generating editor configuration files.
///
/// This module contains utilities for writing config files only when their
/// contents change, serializing settings in the repo's preferred JSON style,
/// and converting between the flat `rust-analyzer.*` key format used by VS Code
/// and the nested structure expected by the Zed editor.
mod ra_settings {

    use std::fs;
    use std::path::Path;

    use serde::Serialize;
    use serde_json::{Map, Value};

    /// Write `content` to `path` only when it differs, so editors do not reload
    /// unchanged settings files on every build. Returns whether a write happened.
    pub fn write_if_changed(path: &Path, content: &str) -> bool {
        if fs::read_to_string(path).is_ok_and(|existing| existing == content) {
            return false;
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, content).unwrap();
        true
    }

    /// Serialize a value as 4-space-indented JSON (matching the repo's hand-written
    /// config style), with a trailing newline.
    pub fn to_pretty_json(value: &Value) -> String {
        let mut buf = Vec::new();
        let mut ser = serde_json::Serializer::with_formatter(
            &mut buf,
            serde_json::ser::PrettyFormatter::with_indent(b"    "),
        );
        value.serialize(&mut ser).unwrap();
        String::from_utf8(buf).unwrap() + "\n"
    }

    /// Convert VSCode-style flat `rust-analyzer.*` keys into the nested
    /// `lsp.rust-analyzer.initialization_options` structure that Zed expects.
    pub fn to_zed_settings(vscode: &Value) -> Value {
        let mut init = Map::new();
        let Some(source) = vscode.as_object() else {
            return Value::Object(init);
        };

        let unknown: Vec<&String> = source
            .keys()
            .filter(|k| !k.starts_with("rust-analyzer."))
            .collect();
        if !unknown.is_empty() {
            eprintln!(
                "warning: ignoring non-rust-analyzer keys when generating Zed config: {unknown:?}"
            );
        }

        for (key, value) in source {
            if let Some(rest) = key.strip_prefix("rust-analyzer.") {
                insert_nested(&mut init, rest, value.clone());
            }
        }

        let mut rust_analyzer = Map::new();
        rust_analyzer.insert("initialization_options".into(), Value::Object(init));
        let mut lsp = Map::new();
        lsp.insert("rust-analyzer".into(), Value::Object(rust_analyzer));
        let mut root = Map::new();
        root.insert("lsp".into(), Value::Object(lsp));
        Value::Object(root)
    }

    /// Insert `value` at the dotted path `a.b.c` inside `map`, creating any
    /// intermediate objects on the way.
    fn insert_nested(map: &mut Map<String, Value>, path: &str, value: Value) {
        let Some((head, tail)) = path.split_once('.') else {
            map.insert(path.to_string(), value);
            return;
        };

        let child = map
            .entry(head.to_string())
            .or_insert_with(|| Value::Object(Map::new()));
        match child {
            Value::Object(child) => insert_nested(child, tail, value),
            // A scalar already occupies this path; promote it to an object.
            _ => {
                let mut fresh = Map::new();
                insert_nested(&mut fresh, tail, value);
                *child = Value::Object(fresh);
            }
        }
    }
}
