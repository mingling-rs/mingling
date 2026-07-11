use std::path::{Path, PathBuf};
use std::process;
use tools::dependency_order::{display_dependency_order, find_workspace_root};
use tools::{
    eprintln_cargo_style, println_cargo_style, run_cmd_capture_with_dir, run_cmd_with_dir,
};

/// Read the version from `[workspace.package].version` in the root Cargo.toml.
fn read_workspace_version(workspace_root: &Path) -> Option<String> {
    let content = std::fs::read_to_string(workspace_root.join("Cargo.toml")).ok()?;
    let value: toml::Value = content.parse().ok()?;
    value
        .get("workspace")?
        .get("package")?
        .get("version")?
        .as_str()
        .map(String::from)
}

/// Replace `path = "crate_name"` with `version = "VERSION"` in the file content.
fn update_path_to_version(content: &str, crate_name: &str, version: &str) -> String {
    let old = format!("path = \"{}\"", crate_name);
    let new = format!("version = \"{}\"", version);
    content.replace(&old, &new)
}

/// Restore the backup and abort.
fn abort(toml_path: &Path) -> ! {
    let backup_path = toml_path.with_extension("toml.bkup");
    if backup_path.exists() {
        if let Err(e) = std::fs::copy(&backup_path, toml_path) {
            eprintln_cargo_style!("failed to restore Cargo.toml from backup: {}", e);
        } else {
            eprintln_cargo_style!("restored Cargo.toml from backup");
        }
    }
    process::exit(1);
}

fn main() {
    // 1. Compute dependency order
    let order = display_dependency_order();
    if order.is_empty() {
        eprintln_cargo_style!("could not find workspace root or mingling crates");
        process::exit(1);
    }

    // 2. Locate workspace root
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let workspace_root = match find_workspace_root(&cwd) {
        Some(r) => r,
        None => {
            eprintln_cargo_style!("could not find workspace root");
            process::exit(1);
        }
    };

    // 3. Read current version
    let version = match read_workspace_version(&workspace_root) {
        Some(v) => v,
        None => {
            eprintln_cargo_style!("could not read workspace.package.version from Cargo.toml");
            process::exit(1);
        }
    };

    println_cargo_style!("Version: detected version {}", version);

    let toml_path = workspace_root.join("Cargo.toml");
    let backup_path = workspace_root.join("Cargo.toml.bkup");

    // 4. Backup Cargo.toml
    if let Err(e) = std::fs::copy(&toml_path, &backup_path) {
        eprintln_cargo_style!("failed to backup Cargo.toml: {}", e);
        process::exit(1);
    }
    println_cargo_style!("Backup: created Cargo.toml.bkup");

    // 5. Iterate crates in dependency order
    for crate_name in &order {
        let name = match crate_name.to_str() {
            Some(n) => n,
            None => {
                eprintln_cargo_style!("invalid crate path: {}", crate_name.display());
                abort(&toml_path);
            }
        };

        let crate_dir = workspace_root.join(crate_name);

        // 5a. Publish (capture output to check for warnings)
        println_cargo_style!("Publish: {}", name);
        let output = run_cmd_capture_with_dir("cargo publish", &crate_dir);
        let output_str = match output {
            Ok(o) => o,
            Err((code, _)) => {
                eprintln_cargo_style!("{} publish failed (exit code {})", name, code);
                abort(&toml_path);
            }
        };

        // Reject any warnings
        let warnings: Vec<&str> = output_str
            .lines()
            .filter(|l| l.to_lowercase().contains("warning:"))
            .collect();
        if !warnings.is_empty() {
            for w in &warnings {
                eprintln_cargo_style!("warning detected: {}", w);
            }
            eprintln_cargo_style!(
                "{} publish rejected due to {} warning(s)",
                name,
                warnings.len()
            );
            abort(&toml_path);
        }

        // 5b. Replace path with version in Cargo.toml
        let content = match std::fs::read_to_string(&toml_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln_cargo_style!("failed to read Cargo.toml: {}", e);
                abort(&toml_path);
            }
        };
        let new_content = update_path_to_version(&content, name, &version);
        if let Err(e) = std::fs::write(&toml_path, &new_content) {
            eprintln_cargo_style!("failed to write Cargo.toml: {}", e);
            abort(&toml_path);
        }
        println_cargo_style!("Dependency: {} path -> version \"{}\"", name, version);

        // 5c. Verify workspace compiles
        println_cargo_style!("Check: workspace");
        if let Err(code) = run_cmd_with_dir("cargo check --workspace", &workspace_root) {
            eprintln_cargo_style!("cargo check --workspace failed (exit code {})", code);
            abort(&toml_path);
        }

        println_cargo_style!("Done: `{}` published", name);
    }

    println_cargo_style!("Done: all crates published successfully");
}
