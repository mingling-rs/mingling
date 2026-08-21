//! Internal helpers for generating editor configuration files.
//!
//! This module contains utilities for writing config files only when their
//! contents change, serializing settings in the repo's preferred JSON style,
//! and converting between the flat `rust-analyzer.*` key format used by VS Code
//! and the nested structure expected by the Zed editor.

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
