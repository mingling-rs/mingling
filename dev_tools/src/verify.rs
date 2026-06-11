use std::path::Path;

use crate::{println_cargo_style, run_cmd_and_capture_stderr};

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
pub fn generate_cargo_toml(block: &CodeBlock, package_name: &str) -> String {
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

    let deps_section = if features_str.is_empty() {
        format!(
            "[dependencies]\nmingling = {{ path = \"{}\" }}\n{extra_deps}",
            find_mingling_relative_path()
        )
    } else {
        format!(
            "[dependencies]\nmingling = {{ path = \"{}\", {features_str} }}\n{extra_deps}",
            find_mingling_relative_path()
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

/// Find the relative path from the temp test directory to mingling crate
fn find_mingling_relative_path() -> &'static str {
    // Tests run from project root, temp is under .temp/
    "../../mingling"
}

/// Generate main.rs for a block
pub fn generate_main_rs(block: &CodeBlock) -> String {
    let mut output = String::from("#![allow(dead_code)]\n\n");

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

    // Build with release
    match run_cmd_and_capture_stderr!(
        "cargo build --release --manifest-path {}",
        manifest_path.to_string_lossy()
    ) {
        Ok(_) => (true, String::new()),
        Err((code, log)) => {
            let mut last_lines: Vec<&str> = log.lines().rev().take(20).collect();
            last_lines.reverse();
            let detail = last_lines.join("\n");
            (false, format!("exit code {code}\n{detail}"))
        }
    }
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
