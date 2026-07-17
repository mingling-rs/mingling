use std::path::{Path, PathBuf};

use serde::Deserialize;
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
    let mut version_map: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    for m in &members {
        version_map.insert(m.name.clone(), m.version.clone());
    }

    // Collect unique member directories that need to be copied
    // Compute relative path by stripping workspace_root prefix
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

    // Copy .cargo directory
    copy_dir(
        &workspace_root.join(".cargo"),
        &pre_release_dir.join(".cargo"),
    );

    // Copy each member directory
    for dir in &member_dirs {
        let src = workspace_root.join(dir);
        let dst = pre_release_dir.join(dir);
        copy_dir(&src, &dst);
    }

    // Copy root Cargo.toml and Cargo.lock
    copy_file(
        &workspace_root.join("Cargo.toml"),
        &pre_release_dir.join("Cargo.toml"),
    );
    copy_file(
        &workspace_root.join("Cargo.lock"),
        &pre_release_dir.join("Cargo.lock"),
    );

    // 5. Replace workspace dependency paths with version numbers in the root Cargo.toml
    //    For workspace-member crates, keep path so cargo can resolve locally during
    //    `cargo package --workspace`. `cargo package` automatically converts path deps
    //    to version deps in the final .crate manifest.
    println_cargo_style!("Patch: resolve workspace dependency versions");

    let pre_release_cargo = pre_release_dir.join("Cargo.toml");
    let content = std::fs::read_to_string(&pre_release_cargo)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", pre_release_cargo.display()));

    let patched = patch_workspace_deps(&content, &version_map);

    std::fs::write(&pre_release_cargo, &patched)
        .unwrap_or_else(|e| panic!("failed to write {}: {e}", pre_release_cargo.display()));

    // Member cargo.toml files: replace direct `path = "..."` deps (pointing to other
    // workspace members) with version qualification, so `cargo package` can produce
    // a valid .crate without path dependencies.
    for dir in &member_dirs {
        let member_cargo = pre_release_dir.join(dir).join("Cargo.toml");
        if !member_cargo.exists() {
            continue;
        }
        let member_content = std::fs::read_to_string(&member_cargo)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", member_cargo.display()));
        let member_patched = patch_member_path_deps(&member_content, &version_map);
        if member_patched != member_content {
            std::fs::write(&member_cargo, &member_patched)
                .unwrap_or_else(|e| panic!("failed to write {}: {e}", member_cargo.display()));
        }
    }

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

        // Extract using tar + gzip
        let extract_cmd = format!(
            "tar -xzf \"{}\" -C \"{}\"",
            path.display(),
            target_dir.display()
        );
        let extract_ok = std::process::Command::new("sh")
            .arg("-c")
            .arg(&extract_cmd)
            .status()
            .expect("failed to run tar");

        if !extract_ok.success() {
            eprintln_cargo_style!("Failed to extract {}", fname);
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

        println_cargo_style!("Export: {}", crate_dir_name);
    }

    println_cargo_style!("Done: .temp/release/ is ready");

    // If package failed, report it
    if package_ok.is_err() {
        eprintln_cargo_style!("cargo package reported errors above");
        std::process::exit(1);
    }
}

/// Replace path-based workspace dependencies with version strings.
///
/// Keeps the path form so that `cargo package --workspace` resolves to local workspace
/// members, but also adds `version = "..."` so the generated .crate has the correct
/// version dependency.
fn patch_workspace_deps(
    content: &str,
    version_map: &std::collections::HashMap<String, String>,
) -> String {
    let mut result = String::new();
    let mut in_workspace_deps = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed == "[workspace.dependencies]" {
            in_workspace_deps = true;
            result.push_str(line);
            result.push('\n');
            continue;
        }

        // Detect end of workspace.dependencies section
        if in_workspace_deps && trimmed.starts_with('[') {
            in_workspace_deps = false;
        }

        if in_workspace_deps
            && let Some(dep_name) = trimmed.split('=').next().map(|s| s.trim())
            && let Some(version) = version_map.get(dep_name)
        {
            let indent = line
                .chars()
                .take_while(|c| c.is_whitespace())
                .collect::<String>();

            if trimmed.contains("path =") {
                let path_value = extract_path_value(trimmed);
                let patched_line: String = if let Some(pv) = path_value {
                    trimmed.replace(
                        &format!("path = \"{}\"", pv),
                        &format!("path = \"{}\", version = \"{}\"", pv, version),
                    )
                } else {
                    trimmed.to_string()
                };
                result.push_str(&format!("{indent}{patched_line}\n"));
            } else {
                result.push_str(&format!("{indent}{dep_name} = \"{version}\"\n"));
            }
            continue;
        }

        result.push_str(line);
        result.push('\n');
    }

    result
}

/// Extract the path value from a dependency line like:
/// `mingling_core = { path = "mingling_core", default-features = false }`
fn extract_path_value(line: &str) -> Option<String> {
    let line = line.trim();
    if let Some(start) = line.find("path = \"") {
        let after_path = &line[start + 8..];
        if let Some(end) = after_path.find('"') {
            return Some(after_path[..end].to_string());
        }
    }
    None
}

/// Patch a member crate's Cargo.toml: add `version = "..."` to direct `path = "..."`
/// dependencies that point to other workspace members.
fn patch_member_path_deps(
    content: &str,
    version_map: &std::collections::HashMap<String, String>,
) -> String {
    let mut result = String::new();
    let mut in_deps = false;
    let mut in_build_deps = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed == "[dependencies]" || trimmed.starts_with("[dependencies.") {
            in_deps = true;
            in_build_deps = false;
        } else if trimmed == "[build-dependencies]" || trimmed.starts_with("[build-dependencies.") {
            in_build_deps = true;
            in_deps = false;
        } else if trimmed.starts_with('[') {
            in_deps = false;
            in_build_deps = false;
        }

        if (in_deps || in_build_deps)
            && trimmed.contains("path = \"")
            && !trimmed.contains("workspace = true")
            && let Some(dep_name) = trimmed.split('=').next().map(|s| s.trim())
            && let Some(version) = version_map.get(dep_name.trim_end_matches(".workspace"))
            && !trimmed.contains("version = \"")
        {
            let indent = line
                .chars()
                .take_while(|c| c.is_whitespace())
                .collect::<String>();
            let path_val = extract_path_value(trimmed).unwrap_or_default();
            let patched = trimmed.replace(
                &format!("path = \"{path_val}\""),
                &format!("path = \"{path_val}\", version = \"{version}\""),
            );
            result.push_str(&format!("{indent}{patched}\n"));
            continue;
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
