use std::{fs, io::Error};

use just_template::{Template, tmpl};

/// Generate lint module registry file (src/lints/mod.rs)
///
/// Read all Rust source files in the src/lints/ directory (excluding mod.rs itself),
/// automatically generate module declarations and pub use export statements, and write them to mod.rs.
pub fn gen_mod_file() -> Result<(), Error> {
    let root = std::env::current_dir()?;
    let lints_dir = root.join("src").join("lints");
    let tmpl_file = root.join("tmpls").join("lints.tmpl");
    let mod_file = root.join("src").join("lints.rs");

    let mut template = Template::from(fs::read_to_string(tmpl_file).unwrap());

    // Collect all .rs file names (without extension), excluding mod
    let mut entries: Vec<String> = fs::read_dir(&lints_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .filter_map(|e| {
            e.path()
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
        })
        .filter(|name| name != "mod")
        .collect();

    entries.sort();

    // Generate module declarations and re-export statements
    for name in &entries {
        tmpl!(template, impls {
            mod_name = name
        })
    }

    // Generate run_all_lints call arms
    for name in &entries {
        tmpl!(template, calls {
            name = name
        })
    }

    fs::write(&mod_file, template.to_string())?;

    Ok(())
}

/// Generate lint metadata registry
///
/// Parses each `.rs` file in `src/lints/` (excluding mod.rs and _init.rs),
/// extracts doc-comment metadata, and writes the result as JSON.
pub fn gen_lint_registry() -> Result<(), Error> {
    let root = std::env::current_dir()?;
    let lints_dir = root.join("src").join("lints");

    let mut lints: Vec<serde_json::Value> = Vec::new();

    for entry in fs::read_dir(&lints_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let stem = match path.file_stem().and_then(|s| s.to_str()) {
            Some(s) => s.to_string(),
            None => continue,
        };
        if stem == "mod" {
            continue;
        }

        let content = fs::read_to_string(&path)?;
        if let Some(meta) = parse_lint_file(&content, &stem) {
            lints.push(meta);
        }
    }

    lints.sort_by(|a, b| {
        a["name"]
            .as_str()
            .unwrap_or("")
            .cmp(b["name"].as_str().unwrap_or(""))
    });

    let json = serde_json::json!({ "lints": lints });
    let out_path = root.join("registry.json");
    fs::write(&out_path, serde_json::to_string_pretty(&json).unwrap())?;
    Ok(())
}

/// Parse a single lint `.rs` file and return its metadata as JSON.
fn parse_lint_file(content: &str, name: &str) -> Option<serde_json::Value> {
    // Collect leading doc comment lines (//!)
    let mut doc_lines: Vec<String> = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("//!") {
            doc_lines.push(rest.trim().to_string());
        } else if !doc_lines.is_empty() {
            break;
        }
    }

    // Title: first non-empty doc line
    let title = doc_lines
        .iter()
        .find(|l| !l.is_empty())
        .cloned()
        .unwrap_or_default();

    // Summary: between ## Summary and ## Metadata
    let mut summary = String::new();
    let mut in_summary = false;
    for line in &doc_lines {
        if line.contains("## Summary") {
            in_summary = true;
            continue;
        }
        if line.contains("## Metadata") {
            in_summary = false;
        }
        if in_summary && !line.starts_with("## ") {
            if !summary.is_empty() {
                summary.push('\n');
            }
            summary.push_str(line.trim());
        }
    }

    // Metadata section
    let mut author = String::new();
    let mut default_level = String::from("warn");
    let mut active_on = String::from("File");
    let mut in_metadata = false;

    for line in &doc_lines {
        if line.contains("## Metadata") {
            in_metadata = true;
            continue;
        }
        if in_metadata {
            if line.starts_with("## ") {
                break;
            }
            if let Some(val) = line
                .strip_prefix("Author:")
                .or_else(|| line.strip_prefix("Author："))
            {
                author = val.trim().trim_matches('`').to_string();
            }
            if let Some(val) = line
                .strip_prefix("Default:")
                .or_else(|| line.strip_prefix("Default："))
            {
                let raw = val.trim().trim_matches('`');
                if raw == "warn" || raw == "allow" || raw == "deny" {
                    default_level = raw.to_string();
                }
            }
        }
    }

    // Extract active_on from function signature (last occurrence = actual fn)
    if let Some(pos) = content.rfind("pub fn linter(") {
        let after = &content[pos..];
        if let Some(colon) = after.find(':') {
            let type_part = &after[colon + 1..].trim();
            let raw = type_part
                .trim_start_matches("syn::")
                .split([' ', ')', ',', '\n'])
                .next()
                .unwrap_or("File");
            // Strip "Item" prefix: ItemFn → Fn, ItemStruct → Struct, etc.
            active_on = raw.strip_prefix("Item").unwrap_or(raw).to_string();
        }
    }

    let mut meta = serde_json::Map::new();
    meta.insert("author".into(), serde_json::Value::String(author));
    meta.insert("default".into(), serde_json::Value::String(default_level));
    meta.insert("active_on".into(), serde_json::Value::String(active_on));

    Some(serde_json::json!({
        "name": name,
        "title": title,
        "summary": summary.trim(),
        "metadata": meta,
    }))
}
