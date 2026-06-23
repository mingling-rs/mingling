use std::io::Write;
use std::path::Path;

use crate::println_cargo_style;

/// Represents a parsed code block from a markdown file
#[derive(Debug, Clone)]
pub struct CodeBlock {
    /// Source file path (for reporting)
    pub source_file: String,
    /// The line number in source file where this block starts
    pub line: usize,
    /// The raw Rust source code
    pub code: String,
    /// Feature flags extracted from `// Features: [...]` comment
    pub features: Vec<String>,
    /// Whether the block had an explicit `// Features:` header
    pub has_features_header: bool,
    /// Whether the block has `// NOT VERIFIED` to opt out of testing
    pub not_verified: bool,
    /// External dependencies extracted from `// Dependencies:` comments
    pub external_deps: Vec<(String, String)>,
    /// Whether this block has a `fn main` entry point
    pub has_main: bool,
    /// Whether this block has `gen_program!()` call
    pub has_gen_program: bool,
}

/// Parse all ```rust code blocks from markdown content
pub fn parse_code_blocks(content: &str, source_file: &str) -> Vec<CodeBlock> {
    let mut blocks = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        if lines[i].trim() == "```rust" {
            if let Some(block) = parse_single_block(&lines, i, source_file) {
                blocks.push(block);
            }
            i += 1;
            while i < lines.len() && lines[i].trim() != "```" {
                i += 1;
            }
        }
        i += 1;
    }

    blocks
}

/// Parse a single code block starting at the ```rust line
fn parse_single_block(lines: &[&str], start: usize, source_file: &str) -> Option<CodeBlock> {
    let line_num = start + 1; // 1-based line number

    let mut code_lines: Vec<String> = Vec::new();
    let mut features: Vec<String> = Vec::new();
    let mut has_features_header = false;
    let mut not_verified = false;
    let mut external_deps: Vec<(String, String)> = Vec::new();
    let mut has_main = false;
    let mut has_gen_program = false;

    let mut idx = start + 1;
    let mut in_header = true;

    while idx < lines.len() {
        let raw_line = lines[idx];
        let trimmed = raw_line.trim();

        if trimmed == "```" {
            break;
        }

        // Parse header comments
        // Check for NOT VERIFIED marker
        if in_header && trimmed == "// NOT VERIFIED" {
            not_verified = true;
            idx += 1;
            continue;
        }

        if in_header && trimmed.starts_with("// ") {
            if trimmed.starts_with("// Features:") {
                has_features_header = true;
                let feat_str = trimmed.trim_start_matches("// Features:").trim();
                if feat_str.starts_with('[') && feat_str.ends_with(']') {
                    let inner = &feat_str[1..feat_str.len() - 1];
                    if !inner.is_empty() {
                        features = inner
                            .split(',')
                            .map(|s| s.trim().trim_matches('"').to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                    }
                }
                idx += 1;
                continue;
            }
            if trimmed == "// Dependencies:" {
                idx += 1;
                // Collect subsequent `// crate = "version"` lines
                while idx < lines.len() {
                    let next = lines[idx].trim();
                    if next == "```" {
                        break;
                    }
                    if next.starts_with("// ") {
                        let dep_line = next.trim_start_matches("// ").trim();
                        if let Some((name, ver)) = dep_line.split_once(" = ") {
                            external_deps.push((
                                name.trim().to_string(),
                                ver.trim().trim_matches('"').to_string(),
                            ));
                        }
                        idx += 1;
                    } else {
                        break;
                    }
                }
                continue;
            }
        }

        in_header = false;

        if raw_line.contains("fn main") {
            has_main = true;
        }
        if raw_line.contains("gen_program!") {
            has_gen_program = true;
        }

        code_lines.push(raw_line.to_string());
        idx += 1;
    }

    if code_lines.is_empty() {
        return None;
    }

    Some(CodeBlock {
        source_file: source_file.to_string(),
        line: line_num,
        code: code_lines.join("\n"),
        features,
        has_features_header,
        not_verified,
        external_deps,
        has_main,
        has_gen_program,
    })
}

/// Generate a Cargo.toml for a block
///
/// `manifest_path` is the full path to the Cargo.toml file being written; it is used to
/// compute the relative path to the `mingling` crate.
pub fn generate_cargo_toml(block: &CodeBlock, package_name: &str, manifest_path: &Path) -> String {
    let features_str = if !block.features.is_empty() {
        let feats: Vec<String> = block.features.iter().map(|f| format!("\"{f}\"")).collect();
        format!("features = [{}]", feats.join(", "))
    } else {
        String::new()
    };

    let mut extra_deps = String::new();
    for (name, version) in &block.external_deps {
        if !version.starts_with('{') {
            // Plain version string, e.g. "1"
            if name == "serde" || name == "clap" {
                extra_deps.push_str(&format!(
                    "{name} = {{ version = \"{version}\", features = [\"derive\"] }}\n"
                ));
            } else {
                extra_deps.push_str(&format!("{name} = \"{version}\"\n"));
            }
        } else {
            // Already in TOML inline table format, e.g. { version = "1", features = [...] }
            extra_deps.push_str(&format!("{name} = {version}\n"));
        }
    }

    let mingling_path = find_mingling_relative_path(manifest_path);

    let deps_section = if features_str.is_empty() {
        format!("[dependencies]\nmingling = {{ path = \"{mingling_path}\" }}\n{extra_deps}",)
    } else {
        format!(
            "[dependencies]\nmingling = {{ path = \"{mingling_path}\", {features_str} }}\n{extra_deps}",
        )
    };

    format!(
        r#"[package]
name = "{package_name}"
version = "0.0.0"
edition = "2024"

{deps_section}
[workspace]
"#
    )
}

/// Compute the relative path from a Cargo.toml's parent directory to the `mingling` crate.
///
/// The process current directory is expected to be the project root (where `mingling/` lives).
/// Returns a forward-slash path safe for embedding in TOML strings.
fn find_mingling_relative_path(manifest_path: &Path) -> String {
    let manifest_dir = manifest_path
        .parent()
        .expect("manifest_path has no parent directory");
    let cwd = std::env::current_dir().expect("failed to get current directory");

    // Strip cwd prefix to get the relative components of the manifest directory
    let relative_to_root = manifest_dir.strip_prefix(&cwd).unwrap_or(manifest_dir);
    let depth = relative_to_root.components().count();

    let mut result = String::new();
    for _ in 0..depth {
        result.push_str("../");
    }
    result.push_str("mingling");
    result
}

/// Generate main.rs for a block
///
/// Automatically prepends `use mingling::prelude::*;` if the block doesn't already have it.
pub fn generate_main_rs(block: &CodeBlock) -> String {
    let mut output = String::from("#![allow(dead_code)]\n#![allow(unused)]\n");

    if !block.code.contains("use mingling::prelude::*;") {
        output.push_str("#[allow(unused_imports)]\nuse mingling::prelude::*;\n\n");
    }

    output.push_str(&block.code);
    output.push('\n');

    if !block.has_main {
        output.push_str("\nfn main() {}\n");
    }

    if !block.has_gen_program {
        output.push_str("\nmingling::macros::gen_program!();\n");
    }

    output
}

/// Build a single code block as a Cargo project.
/// Returns (success, error_message).
pub fn build_block(
    src_dir: &Path,
    manifest_path: &Path,
    cargo_toml: &str,
    main_rs: &str,
) -> (bool, String) {
    if let Err(e) = std::fs::create_dir_all(src_dir) {
        return (false, format!("mkdir: {e}"));
    }

    // Write Cargo.toml
    if let Err(e) = std::fs::write(manifest_path, cargo_toml) {
        return (false, format!("write Cargo.toml: {e}"));
    }

    // Write main.rs
    if let Err(e) = std::fs::write(src_dir.join("main.rs"), main_rs) {
        return (false, format!("write main.rs: {e}"));
    }

    // Check code — inherit stderr so cargo output is real-time and colored
    let shell = if cfg!(target_os = "windows") {
        "powershell"
    } else {
        "sh"
    };
    let cmd = format!(
        "cargo check --color=always --manifest-path {}",
        manifest_path.to_string_lossy()
    );

    let mut child = match std::process::Command::new(shell)
        .arg("-c")
        .arg(&cmd)
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::piped())
        .current_dir(std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")))
        .spawn()
    {
        Ok(c) => c,
        Err(e) => return (false, format!("spawn: {e}")),
    };

    // Read stderr while it streams
    use std::io::BufRead;
    let stderr_handle = child.stderr.take().unwrap();
    let reader = std::io::BufReader::new(stderr_handle);
    let mut captured = String::new();
    for line in reader.lines() {
        match line {
            Ok(l) => {
                let _ = writeln!(std::io::stderr(), "{l}");
                captured.push_str(&l);
                captured.push('\n');
            }
            Err(_) => break,
        }
    }

    let status = child.wait().unwrap_or_else(|_| std::process::exit(1));
    let exit_code = status.code().unwrap_or(1);

    if exit_code == 0 {
        (true, String::new())
    } else {
        let mut last_lines: Vec<&str> = captured.lines().rev().take(20).collect();
        last_lines.reverse();
        let detail = last_lines.join("\n");
        (false, format!("exit code {exit_code}\n{detail}"))
    }
}

/// Compute a stable hash for a code block based on its dependency configuration.
///
/// Blocks with the same features and external dependencies produce the same hash,
/// allowing them to share a compiled crate and avoid redundant recompilation.
///
/// Hash input (all sorted for stability):
/// - Sorted mingling feature strings
/// - Sorted external dependency names
/// - Sorted external dependency versions
/// - Sorted external deps as `name=version` pairs
pub fn compute_block_hash(block: &CodeBlock) -> String {
    let mut features: Vec<&str> = block.features.iter().map(|s| s.as_str()).collect();
    features.sort();
    let features_str = features.join(",");

    let mut dep_names: Vec<&str> = block
        .external_deps
        .iter()
        .map(|(n, _)| n.as_str())
        .collect();
    dep_names.sort();
    let dep_names_str = dep_names.join(",");

    let mut dep_versions: Vec<&str> = block
        .external_deps
        .iter()
        .map(|(_, v)| v.as_str())
        .collect();
    dep_versions.sort();
    let dep_versions_str = dep_versions.join(",");

    let mut deps: Vec<String> = block
        .external_deps
        .iter()
        .map(|(n, v)| format!("{n}={v}"))
        .collect();
    deps.sort();
    let deps_str = deps.join(",");

    let canonical = format!("{features_str}\n{dep_names_str}\n{dep_versions_str}\n{deps_str}");

    // FNV-1a 64-bit hash — stable across runs (no random seed)
    let mut hash: u64 = 0xcbf29ce484222325;
    for &byte in canonical.as_bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }

    format!("{:016x}", hash)
}

/// Determine if a block should be treated as a test candidate.
/// A block is NOT testable only if it has `// NOT VERIFIED` marker.
pub fn is_block_testable(block: &CodeBlock) -> bool {
    !block.not_verified
}

/// Write a summary report
pub fn write_summary_report(
    path: &Path,
    title: &str,
    results: &[(String, usize, bool, String)],
    total: usize,
    passed: usize,
    failed: usize,
) {
    let mut content = String::new();
    content.push_str(&format!("# {title}\n\n"));
    content.push_str(&format!(
        "Tested **{total}** code blocks: **{passed}** passed, **{failed}** failed.\n\n"
    ));
    content.push_str("## Results\n\n");
    content.push_str("| Block | File | Line | Status |\n");
    content.push_str("|-------|------|------|--------|\n");

    for (i, (file, line, ok, _)) in results.iter().enumerate() {
        let status = if *ok { "PASS" } else { "FAIL" };
        let short_file = file.rsplit('/').next().unwrap_or(file);
        content.push_str(&format!(
            "| {} | {} | {} | {status} |\n",
            i + 1,
            short_file,
            line
        ));
    }

    let has_failures = results.iter().any(|(_, _, ok, _)| !ok);
    if has_failures {
        content.push_str("\n## Failed Blocks\n\n");
        for (i, (file, line, ok, err)) in results.iter().enumerate() {
            if !ok {
                content.push_str(&format!(
                    "### Block {} (`{}`, line {})\n\n```\n{err}\n```\n\n",
                    i + 1,
                    file,
                    line
                ));
            }
        }
    }

    std::fs::write(path, &content).unwrap_or_else(|e| {
        eprintln!("Warning: failed to write {path:?}: {e}");
    });

    println_cargo_style!("Report: written to {}", path.display());
}
