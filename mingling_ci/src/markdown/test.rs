//! Parallel execution of markdown test projects.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};

use super::project::{
    MarkdownTestProject, generate_build_rs, generate_cargo_toml, generate_main_rs,
};

/// Temporary root for the generated test crates.
const TEMP_BASE: &str = ".temp/doc-test";

/// Runs the given projects in parallel.
///
/// Projects sharing a dependency hash share one temporary crate (written
/// serially within the group); groups run in parallel. Progress is shown on
/// stderr; failures print immediately. Returns the number of failed blocks.
pub(crate) async fn try_test_markdown_project(projs: Vec<MarkdownTestProject>) -> usize {
    // Group by dependency hash for crate sharing.
    let mut groups: BTreeMap<String, Vec<MarkdownTestProject>> = BTreeMap::new();
    for proj in projs {
        groups.entry(proj.compute_hash()).or_default().push(proj);
    }

    let total: usize = groups.values().map(Vec::len).sum();
    let pb = ProgressBar::new(total as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(&format!(
                "{} [{{bar:28}}] {{pos}}/{{len}}: {{msg}}",
                "     Testing".bold().bright_cyan()
            ))
            .unwrap()
            .progress_chars("=> "),
    );
    pb.set_message("blocks");

    // One blocking task per group; blocks within a group are serial because
    // they share the same crate directory.
    let mut handles = Vec::new();
    for (hash, blocks) in groups {
        let pb = pb.clone();
        handles.push(tokio::task::spawn_blocking(move || {
            let crate_dir = PathBuf::from(TEMP_BASE).join(&hash);
            let src_dir = crate_dir.join("src");
            let manifest_path = crate_dir.join("Cargo.toml");
            let cargo_toml = generate_cargo_toml(&blocks[0], &manifest_path);

            let mut failed = 0;
            for proj in &blocks {
                let label = format!("{}:{}", proj.source_file, proj.line);
                pb.set_message(label.clone());

                let main_rs = if proj.is_build_time {
                    generate_build_rs(proj)
                } else {
                    generate_main_rs(proj)
                };
                let (ok, err) = build_block(
                    &src_dir,
                    &manifest_path,
                    &cargo_toml,
                    &main_rs,
                    proj.is_build_time,
                );
                pb.inc(1);

                if !ok {
                    failed += 1;
                    // Plain stderr: `pb.println` is swallowed on non-TTY (CI).
                    eprintln!("  {} {label}", "failed".bold().bright_red());
                    eprintln!("  {label} FAILED:\n{err}");
                }
            }
            failed
        }));
    }

    let mut fail_count = 0;
    for handle in handles {
        if let Ok(failed) = handle.await {
            fail_count += failed;
        }
    }

    pb.finish_and_clear();
    fail_count
}

/// Writes the temporary crate files and runs `cargo check`.
///
/// When `is_build_time` is true, the content goes to `build.rs` with a stub
/// `main.rs`; otherwise it goes to `src/main.rs`.
fn build_block(
    src_dir: &Path,
    manifest_path: &Path,
    cargo_toml: &str,
    content: &str,
    is_build_time: bool,
) -> (bool, String) {
    if let Err(e) = std::fs::create_dir_all(src_dir) {
        return (false, format!("mkdir: {e}"));
    }
    if let Err(e) = std::fs::write(manifest_path, cargo_toml) {
        return (false, format!("write Cargo.toml: {e}"));
    }

    if is_build_time {
        let crate_dir = manifest_path
            .parent()
            .expect("manifest path has a parent directory");
        if let Err(e) = std::fs::write(crate_dir.join("build.rs"), content) {
            return (false, format!("write build.rs: {e}"));
        }
        if let Err(e) = std::fs::write(src_dir.join("main.rs"), "fn main() {}\n") {
            return (false, format!("write main.rs: {e}"));
        }
    } else if let Err(e) = std::fs::write(src_dir.join("main.rs"), content) {
        return (false, format!("write main.rs: {e}"));
    }

    let output = std::process::Command::new("cargo")
        .args(["check", "--color=always", "--manifest-path"])
        .arg(manifest_path)
        .output();
    match output {
        Ok(output) if output.status.success() => (true, String::new()),
        Ok(output) => {
            let mut log = String::from_utf8_lossy(&output.stdout).into_owned();
            log.push_str(&String::from_utf8_lossy(&output.stderr));
            let lines: Vec<&str> = log.lines().collect();
            let tail = &lines[lines.len().saturating_sub(20)..];
            let exit = output
                .status
                .code()
                .map_or_else(|| "?".to_string(), |c| c.to_string());
            (false, format!("exit code {exit}\n{}", tail.join("\n")))
        }
        Err(e) => (false, format!("failed to run cargo: {e}")),
    }
}
