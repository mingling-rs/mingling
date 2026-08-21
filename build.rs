//! This module initializes the Mingling local repository workspace.
//!
//! It automatically generates necessary files for development and binds
//! the repository's shared git hooks.

use serde_json::Value;
use std::env::current_dir;
use std::fs;
use std::path::Path;
use std::process::Command;

#[path = "dev/build/ra_setting.rs"]
mod ra_settings;

fn main() {
    gen_fake_cargo_toml_in_temp_dir();
    gen_rust_analyzer_config_for_editors();
    bind_git_hooks();
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

/// Bind the shared git hooks in `dev/git_hooks/` to this repository.
///
/// Sets `core.hooksPath` so Git runs the hooks checked into the repo instead
/// of private copies in `.git/hooks/`, keeping every clone in sync.
fn bind_git_hooks() {
    const HOOKS_DIR: &str = "dev/git_hooks";

    // Re-run this build script whenever hooks are added, removed, or changed.
    println!("cargo:rerun-if-changed={HOOKS_DIR}");

    // Git only runs hooks that are executable; the scripts are tracked as
    // non-executable, so grant the permission bits once.
    #[cfg(unix)]
    make_hooks_executable(&current_dir().unwrap().join(HOOKS_DIR));

    // Skip the write when the repository is already bound, so builds stay quiet.
    let current = Command::new("git")
        .args(["config", "--get", "core.hooksPath"])
        .output();
    if current.is_ok_and(|out| {
        out.status.success() && String::from_utf8_lossy(&out.stdout).trim() == HOOKS_DIR
    }) {
        return;
    }

    match Command::new("git")
        .args(["config", "core.hooksPath", HOOKS_DIR])
        .status()
    {
        Ok(status) if status.success() => eprintln!("bound git hooks in `{HOOKS_DIR}`"),
        _ => eprintln!("warning: failed to bind git hooks in `{HOOKS_DIR}`"),
    }
}

/// Grant the execute permission to every hook script in `dir`.
#[cfg(unix)]
fn make_hooks_executable(dir: &Path) {
    use std::os::unix::fs::PermissionsExt;

    let Ok(entries) = fs::read_dir(dir) else {
        eprintln!("warning: `{}` not found, skip git hooks", dir.display());
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(metadata) = fs::metadata(&path) else {
            continue;
        };
        if metadata.is_file() && metadata.permissions().mode() & 0o111 == 0 {
            let mode = metadata.permissions().mode() | 0o755;
            fs::set_permissions(&path, fs::Permissions::from_mode(mode)).unwrap();
        }
    }
}
