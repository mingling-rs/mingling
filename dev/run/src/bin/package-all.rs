use std::collections::HashMap;
use std::path::{Path, PathBuf};

use flate2::read::GzDecoder;
use serde::Deserialize;
use tar::Archive;
use toml::Table as TomlTable;
use tools::{
    dependency_order::find_workspace_root, eprintln_cargo_style, println_cargo_style,
    run_cmd_capture_with_dir, wprintln_cargo_style,
};

/// A single member from `cargo metadata` output.
#[derive(Deserialize, Debug)]
struct MetadataPackage {
    name: String,
    version: String,
    manifest_path: String,
}

/// The top-level metadata structure.
#[derive(Deserialize, Debug)]
struct Metadata {
    #[allow(dead_code)]
    workspace_root: String,
    packages: Vec<MetadataPackage>,
}

fn main() {
    // 1. Determine project root
    let cwd = std::env::current_dir().expect("failed to get current working directory");
    let workspace_root = find_workspace_root(&cwd).expect("not inside a Cargo workspace");
    println_cargo_style!("Workspace: {}", workspace_root.display());

    let pre_release_dir = workspace_root.join(".temp/pre-release");

    // 2. Clean `.temp/pre-release/`
    println_cargo_style!("Clean: .temp/pre-release/");
    let _ = std::fs::remove_dir_all(&pre_release_dir);
    std::fs::create_dir_all(&pre_release_dir).expect("failed to create .temp/pre-release/");

    // 3. Run `cargo metadata` to get workspace members info
    println_cargo_style!("Metadata: querying workspace members");
    let metadata_json = run_cmd_capture_with_dir(
        "cargo metadata --format-version 1 --no-deps",
        &workspace_root,
    )
    .unwrap_or_else(|(code, _msg)| {
        eprintln_cargo_style!(format!("cargo metadata failed (exit {code}):\n{{msg}}"));
        std::process::exit(1);
    });

    let metadata: Metadata = serde_json::from_str(&metadata_json).unwrap_or_else(|e| {
        eprintln_cargo_style!("failed to parse cargo metadata: {}", e);
        std::process::exit(1);
    });

    // Filter workspace members: skip the root virtual manifest
    let workspace_root_str = workspace_root.to_string_lossy().replace('\\', "/");
    let members: Vec<&MetadataPackage> = metadata
        .packages
        .iter()
        .filter(|p| {
            let mp = p.manifest_path.replace('\\', "/");
            mp.starts_with(&workspace_root_str)
                && mp != format!("{}/Cargo.toml", workspace_root_str)
        })
        .collect();

    if members.is_empty() {
        eprintln_cargo_style!("No workspace members found!");
        std::process::exit(1);
    }

    // Print member info
    for m in &members {
        println_cargo_style!("Member: {}@{}", m.name, m.version);
    }

    // Build version map: crate_name -> version
    let mut version_map: HashMap<String, String> = HashMap::new();
    for m in &members {
        version_map.insert(m.name.clone(), m.version.clone());
    }

    // Collect unique member directories that need to be copied
    let mut member_dirs: Vec<PathBuf> = Vec::new();
    for m in &members {
        let dir = Path::new(&m.manifest_path)
            .parent()
            .expect("manifest_path has no parent");
        let canonical_dir = std::fs::canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf());
        let canonical_root =
            std::fs::canonicalize(&workspace_root).unwrap_or_else(|_| workspace_root.to_path_buf());
        let relative = canonical_dir
            .strip_prefix(&canonical_root)
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|_| PathBuf::from(dir.file_name().unwrap_or_default()));
        if !member_dirs.contains(&relative) {
            member_dirs.push(relative);
        }
    }

    // 4. Copy files to the temp directory, preserving the workspace directory structure
    println_cargo_style!("Copy: project structure to .temp/pre-release/");

    copy_dir(
        &workspace_root.join(".cargo"),
        &pre_release_dir.join(".cargo"),
    );

    for dir in &member_dirs {
        let src = workspace_root.join(dir);
        let dst = pre_release_dir.join(dir);
        copy_dir(&src, &dst);
    }

    copy_file(
        &workspace_root.join("Cargo.toml"),
        &pre_release_dir.join("Cargo.toml"),
    );
    copy_file(
        &workspace_root.join("Cargo.lock"),
        &pre_release_dir.join("Cargo.lock"),
    );

    // 5. Fully resolve ALL workspace inheritance in every member's Cargo.toml,
    //    so each crate becomes monomorphic (no `workspace = true` references).
    //    Then strip `[workspace.dependencies]` and `[workspace.package]` from the
    //    root Cargo.toml, since they are no longer needed.
    println_cargo_style!("Resolve: inline all workspace inheritance");

    // Parse workspace config from the COPIED root Cargo.toml
    let root_cargo_path = pre_release_dir.join("Cargo.toml");
    let root_content = std::fs::read_to_string(&root_cargo_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", root_cargo_path.display()));

    let (ws_package, ws_deps) = parse_workspace_config(&root_content);

    // Resolve each member's Cargo.toml
    for dir in &member_dirs {
        let member_cargo = pre_release_dir.join(dir).join("Cargo.toml");
        if !member_cargo.exists() {
            continue;
        }
        let member_content = std::fs::read_to_string(&member_cargo)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", member_cargo.display()));
        let resolved =
            resolve_member_manifest(&member_content, dir, &ws_package, &ws_deps, &version_map);
        std::fs::write(&member_cargo, &resolved)
            .unwrap_or_else(|e| panic!("failed to write {}: {e}", member_cargo.display()));
    }

    // Strip [workspace.dependencies], [workspace.package], and root [package]
    // from root Cargo.toml, making it a pure virtual manifest.
    let stripped = strip_workspace_config(&root_content);
    std::fs::write(&root_cargo_path, &stripped)
        .unwrap_or_else(|e| panic!("failed to write {}: {e}", root_cargo_path.display()));

    println_cargo_style!("Package: running cargo package --workspace --no-verify");

    // 6. Run cargo package in the temp directory
    let package_ok = run_cmd_capture_with_dir(
        "cargo package --workspace --no-verify --color always",
        &pre_release_dir,
    );

    match &package_ok {
        Ok(out) => {
            println!("{out}");
        }
        Err((code, msg)) => {
            // Print output but don't fail yet
            eprintln_cargo_style!(format!("cargo package exited with code {code}:"));
            println!("{msg}");
        }
    }

    // 7. Copy built packages back to .temp/target/package
    let temp_package_dir = workspace_root.join(".temp/target/package");
    std::fs::create_dir_all(&temp_package_dir)
        .unwrap_or_else(|e| panic!("failed to create {}: {e}", temp_package_dir.display()));

    // cargo package puts .crate files in target/package
    let pre_release_target_package = pre_release_dir.join(".temp/target/package");
    if pre_release_target_package.exists() {
        println_cargo_style!("Copy: packages to .temp/target/package");
        copy_dir_contents(&pre_release_target_package, &temp_package_dir);
    } else {
        wprintln_cargo_style!("No packages found in .temp/pre-release/.temp/target/package");
    }

    // 8. Export each crate as a standalone project from the .crate packages.
    //    The .crate files contain the final publish-ready Cargo.toml with all
    //    workspace/path deps already resolved by `cargo package`.
    let release_dir = workspace_root.join(".temp/release");
    println_cargo_style!("Export: standalone crates to .temp/release/");
    let _ = std::fs::remove_dir_all(&release_dir);
    std::fs::create_dir_all(&release_dir)
        .unwrap_or_else(|e| panic!("failed to create {}: {e}", release_dir.display()));

    for entry in std::fs::read_dir(&temp_package_dir).expect("failed to read target/package") {
        let entry = entry.expect("failed to read entry");
        let path = entry.path();
        if path.extension().is_none_or(|e| e != "crate") {
            continue;
        }

        // .crate files are gzipped tarballs. Extract to .temp/release/<name>/
        // fname is like "mingling-0.3.0"
        let fname = path.file_stem().unwrap().to_string_lossy().to_string();

        // Derive crate directory name by stripping the version suffix
        // mingling-0.3.0 -> mingling, arg-picker-0.1.0 -> arg-picker
        let crate_dir_name = fname
            .rfind('-')
            .and_then(|dash| {
                // Check if what follows looks like a semver
                let ver_part = &fname[dash + 1..];
                if ver_part.chars().next().is_some_and(|c| c.is_ascii_digit()) {
                    Some(&fname[..dash])
                } else {
                    None
                }
            })
            .unwrap_or(&fname)
            .to_string();

        let target_dir = release_dir.join(&crate_dir_name);
        std::fs::create_dir_all(&target_dir)
            .unwrap_or_else(|e| panic!("failed to create {}: {e}", target_dir.display()));

        // Extract using flate2 + tar (cross-platform)
        let file = match std::fs::File::open(&path) {
            Ok(f) => f,
            Err(e) => {
                eprintln_cargo_style!("Failed to open {}: {e}", path.display());
                continue;
            }
        };
        let decoder = GzDecoder::new(file);
        let mut archive = Archive::new(decoder);
        if let Err(e) = archive.unpack(&target_dir) {
            eprintln_cargo_style!("Failed to extract {}: {e}", fname);
            continue;
        }

        // Move the contents from the inner dir up one level
        // .crate contains a single top-level dir named after the package
        let inner = target_dir.join(&fname);
        if inner.exists() {
            for inner_entry in std::fs::read_dir(&inner).expect("failed to read inner dir") {
                let inner_entry = inner_entry.expect("failed to read entry");
                let inner_path = inner_entry.path();
                let dest = target_dir.join(inner_path.file_name().unwrap());
                if dest.exists() {
                    let _ = std::fs::remove_dir_all(&dest);
                }
                std::fs::rename(&inner_path, &dest).unwrap_or_else(|e| {
                    panic!(
                        "failed to rename {} -> {}: {e}",
                        inner_path.display(),
                        dest.display()
                    )
                });
            }
            let _ = std::fs::remove_dir_all(&inner);
        }

        // Clean up: remove .orig file (only the normalized Cargo.toml is needed)
        let _ = std::fs::remove_file(target_dir.join("Cargo.toml.orig"));
        // Also remove Cargo.lock — standalone crate doesn't need it for publish
        let _ = std::fs::remove_file(target_dir.join("Cargo.lock"));

        // Append an empty [workspace] section so each crate is a valid workspace root
        let cargo_toml_path = target_dir.join("Cargo.toml");
        if cargo_toml_path.exists() {
            let mut cargo_content = std::fs::read_to_string(&cargo_toml_path)
                .unwrap_or_else(|e| panic!("failed to read {}: {e}", cargo_toml_path.display()));
            // Only append if there isn't already a [workspace] section
            if !cargo_content.contains("\n[workspace]\n")
                && !cargo_content.ends_with("\n[workspace]\n")
            {
                cargo_content.push_str("\n[workspace]\n");
                std::fs::write(&cargo_toml_path, &cargo_content).unwrap_or_else(|e| {
                    panic!("failed to write {}: {e}", cargo_toml_path.display())
                });
            }
        }

        println_cargo_style!("Export: {}", crate_dir_name);
    }

    println_cargo_style!("Done: .temp/release/ is ready");

    // If package failed, report it
    if package_ok.is_err() {
        eprintln_cargo_style!("cargo package reported errors above");
        std::process::exit(1);
    }
}

/// Parse `[workspace.package]` and `[workspace.dependencies]` from the root Cargo.toml.
/// Returns (package_fields, dep_values).
fn parse_workspace_config(
    content: &str,
) -> (HashMap<String, String>, HashMap<String, toml::Value>) {
    let table: TomlTable = content.parse().expect("failed to parse root Cargo.toml");

    let mut package_fields = HashMap::new();
    if let Some(workspace) = table.get("workspace").and_then(|w| w.as_table())
        && let Some(pkg_table) = workspace.get("package").and_then(|p| p.as_table())
    {
        for (k, v) in pkg_table {
            if let Some(s) = v.as_str() {
                package_fields.insert(k.clone(), s.to_string());
            }
        }
    }

    let mut dep_values = HashMap::new();
    if let Some(workspace) = table.get("workspace").and_then(|w| w.as_table())
        && let Some(deps_table) = workspace.get("dependencies").and_then(|d| d.as_table())
    {
        for (k, v) in deps_table {
            dep_values.insert(k.clone(), v.clone());
        }
    }

    (package_fields, dep_values)
}

/// Serialize a `toml::Value` into Cargo-toml-compatible inline representation.
fn toml_value_str(v: &toml::Value) -> String {
    match v {
        toml::Value::String(s) => format!("\"{}\"", s),
        toml::Value::Table(t) => {
            let items: Vec<String> = t
                .iter()
                .map(|(k, val)| format!("{} = {}", k, toml_value_str(val)))
                .collect();
            format!("{{ {} }}", items.join(", "))
        }
        toml::Value::Array(a) => {
            let items: Vec<String> = a.iter().map(toml_value_str).collect();
            format!("[{}]", items.join(", "))
        }
        toml::Value::Boolean(b) => b.to_string(),
        toml::Value::Integer(i) => i.to_string(),
        toml::Value::Float(f) => f.to_string(),
        toml::Value::Datetime(dt) => format!("\"{}\"", dt),
    }
}

/// Compute the relative path from `member_rel_dir` to `target_path`.
/// Both are relative to workspace root.
/// e.g. member_rel_dir="mingling", target_path="mingling_core" → "../mingling_core"
fn make_path_relative_to_member(target_path: &str, member_rel_dir: &Path) -> String {
    if member_rel_dir.as_os_str().is_empty() || member_rel_dir == Path::new(".") {
        return target_path.to_string();
    }
    let depth = member_rel_dir.components().count();
    let mut result = PathBuf::new();
    for _ in 0..depth {
        result.push("..");
    }
    result.push(target_path);
    result.to_string_lossy().to_string()
}

/// Resolve a dependency definition from `[workspace.dependencies]` to an inline string.
/// If the definition contains a path to a workspace member, add `version = "..."`
/// and adjust the path to be relative to the member's directory.
fn resolve_dep_def(
    dep_name: &str,
    dep_def: &toml::Value,
    member_rel_dir: &Path,
    version_map: &HashMap<String, String>,
) -> String {
    match dep_def {
        toml::Value::String(ver) => {
            format!("\"{}\"", ver)
        }
        toml::Value::Table(t) => {
            let mut resolved = t.clone();
            let has_path = t.contains_key("path");
            let is_ws_member = version_map.contains_key(dep_name);

            // Fix path to be relative to member's directory
            if has_path && let Some(path_val) = t.get("path").and_then(|v| v.as_str()) {
                let rel = make_path_relative_to_member(path_val, member_rel_dir);
                resolved.insert("path".to_string(), toml::Value::String(rel));
            }

            // Add version for workspace member path deps
            if has_path
                && is_ws_member
                && let Some(version) = version_map.get(dep_name)
            {
                resolved.insert("version".to_string(), toml::Value::String(version.clone()));
            }

            let items: Vec<String> = resolved
                .iter()
                .map(|(k, val)| format!("{} = {}", k, toml_value_str(val)))
                .collect();
            format!("{{ {} }}", items.join(", "))
        }
        _ => toml_value_str(dep_def),
    }
}

/// Merge an inline `{ workspace = true, optional = true, ... }` with the workspace definition.
/// Returns the full resolved dependency value string (without the leading `dep_name = `).
fn merge_inline_dep(
    inline_rest: &str,
    dep_name: &str,
    dep_def: &toml::Value,
    member_rel_dir: &Path,
    version_map: &HashMap<String, String>,
) -> String {
    // inline_rest is the part after `=`: `{ workspace = true, optional = true }`
    let inner = inline_rest
        .trim()
        .strip_prefix('{')
        .and_then(|s| s.strip_suffix('}'))
        .unwrap_or("");

    match dep_def {
        toml::Value::String(ver) => {
            // Workspace def is just a version string
            // Collect extras: everything except `workspace = true`
            let extras: Vec<&str> = inner
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty() && *s != "workspace = true")
                .collect();

            if extras.is_empty() {
                format!("\"{}\"", ver)
            } else {
                // Serialize as inline table: version + extras
                let mut parts = vec![format!("version = \"{}\"", ver)];
                parts.extend(extras.iter().map(|s| s.to_string()));
                format!("{{ {} }}", parts.join(", "))
            }
        }
        toml::Value::Table(t) => {
            // Start from workspace def
            let mut merged = t.clone();

            // Fix path to be relative to member's directory
            if let Some(path_val) = t.get("path").and_then(|v| v.as_str()) {
                let rel = make_path_relative_to_member(path_val, member_rel_dir);
                merged.insert("path".to_string(), toml::Value::String(rel));
            }

            // If this dep is a workspace member with a path dep, add version
            if t.contains_key("path")
                && version_map.contains_key(dep_name)
                && let Some(version) = version_map.get(dep_name)
            {
                merged.insert("version".to_string(), toml::Value::String(version.clone()));
            }

            // Apply extra fields from the inline
            for piece in inner.split(',').map(|s| s.trim()) {
                let piece = piece.trim();
                if piece.is_empty() || piece == "workspace = true" {
                    continue;
                }
                // Parse `key = value` pairs
                if let Some((raw_key, raw_val)) = piece.split_once('=') {
                    let k = raw_key.trim();
                    let v = raw_val.trim();
                    if k.is_empty() {
                        continue;
                    }
                    // Try to infer the value type
                    if v == "true" {
                        merged.insert(k.to_string(), toml::Value::Boolean(true));
                    } else if v == "false" {
                        merged.insert(k.to_string(), toml::Value::Boolean(false));
                    } else if v.starts_with('"') && v.ends_with('"') {
                        merged.insert(
                            k.to_string(),
                            toml::Value::String(v[1..v.len() - 1].to_string()),
                        );
                    } else if v.starts_with('[') && v.ends_with(']') {
                        // Simple array parsing: strings only
                        let arr: Vec<toml::Value> = v[1..v.len() - 1]
                            .split(',')
                            .map(|s| {
                                let s = s.trim().trim_matches('"');
                                toml::Value::String(s.to_string())
                            })
                            .collect();
                        merged.insert(k.to_string(), toml::Value::Array(arr));
                    } else if let Ok(n) = v.parse::<i64>() {
                        merged.insert(k.to_string(), toml::Value::Integer(n));
                    } else if let Ok(f) = v.parse::<f64>() {
                        merged.insert(k.to_string(), toml::Value::Float(f));
                    } else {
                        // Treat as string
                        merged.insert(k.to_string(), toml::Value::String(v.to_string()));
                    }
                }
            }

            let items: Vec<String> = merged
                .iter()
                .map(|(k, val)| format!("{} = {}", k, toml_value_str(val)))
                .collect();
            format!("{{ {} }}", items.join(", "))
        }
        _ => toml_value_str(dep_def),
    }
}

/// Resolve ALL workspace inheritance in a single member crate's Cargo.toml:
///   - `version.workspace = true` → `version = "0.3.0"`
///   - `dep.workspace = true` → inline the full definition from ws_deps
///   - `dep = { workspace = true, ... }` → merge with ws_deps definition
fn resolve_member_manifest(
    content: &str,
    member_rel_dir: &Path,
    ws_package: &HashMap<String, String>,
    ws_deps: &HashMap<String, toml::Value>,
    version_map: &HashMap<String, String>,
) -> String {
    let mut result = String::new();
    let mut in_package = false;
    let mut in_section_with_deps = false;

    for line in content.lines() {
        let trimmed = line.trim();
        let indent: String = line.chars().take_while(|c| c.is_whitespace()).collect();

        // Track sections
        if trimmed.starts_with('[') {
            in_package = trimmed == "[package]";
            in_section_with_deps = trimmed.starts_with("[dependencies")
                || trimmed.starts_with("[build-dependencies")
                || trimmed.starts_with("[dev-dependencies");
            result.push_str(line);
            result.push('\n');
            continue;
        }

        // [package] section: resolve `field.workspace = true`
        if in_package && trimmed.ends_with(".workspace = true") {
            let key = trimmed.strip_suffix(".workspace = true").unwrap().trim();
            if let Some(value) = ws_package.get(key) {
                result.push_str(&format!("{indent}{key} = \"{value}\"\n"));
                continue;
            }
            // Also check workspace.dependencies (for fields like `version.workspace`)
            // when the member has its own version field inherited from workspace.package
        }

        // Dependency sections
        if in_section_with_deps {
            // Shorthand: `foo.workspace = true`
            if trimmed.ends_with(".workspace = true") {
                let key = trimmed.strip_suffix(".workspace = true").unwrap().trim();
                if let Some(dep_def) = ws_deps.get(key) {
                    let resolved = resolve_dep_def(key, dep_def, member_rel_dir, version_map);
                    result.push_str(&format!("{indent}{key} = {resolved}\n"));
                    continue;
                }
            }

            // Inline: `foo = { workspace = true, optional = true, ... }`
            if let Some(eq_pos) = trimmed.find("= {")
                && trimmed.contains("workspace = true")
            {
                let dep_name = trimmed[..eq_pos].trim();
                if let Some(dep_def) = ws_deps.get(dep_name) {
                    let after_eq = trimmed[eq_pos + 1..].trim();
                    let merged =
                        merge_inline_dep(after_eq, dep_name, dep_def, member_rel_dir, version_map);
                    result.push_str(&format!("{indent}{dep_name} = {merged}\n"));
                    continue;
                }
            }
        }

        result.push_str(line);
        result.push('\n');
    }

    result
}

/// Remove `[workspace.dependencies]`, `[workspace.package]`, and the root `[package]`
/// section from root Cargo.toml, making it a pure virtual manifest.
/// Keeps `[workspace]` with `members`, `resolver`, `exclude` so packaging still works.
fn strip_workspace_config(content: &str) -> String {
    let mut result = String::new();
    let mut in_ws_deps = false;
    let mut in_ws_package = false;
    let mut in_root_package = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed == "[workspace.dependencies]" {
            in_ws_deps = true;
            continue;
        }
        if trimmed == "[workspace.package]" {
            in_ws_package = true;
            continue;
        }
        if trimmed == "[package]" && !in_ws_deps && !in_ws_package {
            // Remove the root [package] section entirely (virtual manifest)
            in_root_package = true;
            continue;
        }

        if in_ws_deps {
            if trimmed.starts_with('[') {
                in_ws_deps = false;
            } else {
                continue;
            }
        }

        if in_ws_package {
            if trimmed.starts_with('[') {
                in_ws_package = false;
            } else {
                continue;
            }
        }

        if in_root_package {
            if trimmed.starts_with('[') {
                in_root_package = false;
            } else {
                continue;
            }
        }

        result.push_str(line);
        result.push('\n');
    }

    result
}

/// Recursively copy a directory.
fn copy_dir(src: &Path, dst: &Path) {
    copy_dir_filtered(src, dst, &|_: &Path| true)
}

/// Recursively copy a directory with a filter function.
/// The filter receives the source path and returns `true` if the entry should be copied.
fn copy_dir_filtered(src: &Path, dst: &Path, filter: &dyn Fn(&Path) -> bool) {
    if !src.exists() {
        return;
    }
    if !filter(src) {
        return;
    }
    std::fs::create_dir_all(dst)
        .unwrap_or_else(|e| panic!("failed to create {}: {e}", dst.display()));

    for entry in std::fs::read_dir(src).expect("failed to read directory") {
        let entry = entry.expect("failed to read entry");
        let entry_type = entry.file_type().expect("failed to get file type");
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if !filter(&src_path) {
            continue;
        }

        if entry_type.is_dir() {
            copy_dir_filtered(&src_path, &dst_path, filter);
        } else if entry_type.is_file() || entry_type.is_symlink() {
            copy_file(&src_path, &dst_path);
        }
    }
}

/// Copy a file, creating parent directories as needed.
/// If src is a symlink, copies the target content (follow symlinks).
fn copy_file(src: &Path, dst: &Path) {
    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent)
            .unwrap_or_else(|e| panic!("failed to create {}: {e}", parent.display()));
    }

    let resolved = if src.is_symlink() {
        let target = std::fs::read_link(src)
            .unwrap_or_else(|e| panic!("failed to read symlink {}: {e}", src.display()));
        if target.is_relative() {
            src.parent().unwrap().join(target)
        } else {
            target
        }
    } else {
        src.to_path_buf()
    };

    std::fs::copy(&resolved, dst).unwrap_or_else(|e| {
        panic!(
            "failed to copy {} -> {}: {e}",
            resolved.display(),
            dst.display()
        )
    });
}

/// Copy all files/directories from one directory into another.
fn copy_dir_contents(src: &Path, dst: &Path) {
    if !src.exists() {
        return;
    }
    std::fs::create_dir_all(dst)
        .unwrap_or_else(|e| panic!("failed to create {}: {e}", dst.display()));

    for entry in std::fs::read_dir(src).expect("failed to read directory") {
        let entry = entry.expect("failed to read entry");
        let entry_type = entry.file_type().expect("failed to get file type");
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if entry_type.is_dir() {
            copy_dir(&src_path, &dst_path);
        } else if entry_type.is_file() || entry_type.is_symlink() {
            copy_file(&src_path, &dst_path);
        }
    }
}
