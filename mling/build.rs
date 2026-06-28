use std::path::Path;
use std::process::Command;

use mingling::build::build_comp_scripts;
use mingling::builds::analyze_and_build_type_mapping;

fn main() {
    build_version_info();
    build_completion();
    analyze_and_build_type_mapping().unwrap();
}

fn build_version_info() {
    // Read version from CARGO_PKG_VERSION (inherited from workspace Cargo.toml)
    let version = env!("CARGO_PKG_VERSION");

    // Get git commit hash (first 9 characters)
    let commit_hash = Command::new("git")
        .args(["rev-parse", "--short=9", "HEAD"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Get date from git commit, fallback to current date
    let date = Command::new("git")
        .args(["log", "-1", "--format=%ad", "--date=format:%Y-%m-%d"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d").to_string());

    let version_string = format!("mling {version} ({commit_hash} {date})");

    let out_dir = Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("src")
        .join("helps")
        .join("version.txt");

    std::fs::write(&out_dir, version_string).expect("failed to write version.txt");

    println!("cargo::rerun-if-changed=../Cargo.toml");
    println!("cargo::rerun-if-changed=../Cargo.lock");
    println!("cargo::rerun-if-changed=.git/HEAD");
}

fn build_completion() {
    build_comp_scripts("mling").unwrap();
}
