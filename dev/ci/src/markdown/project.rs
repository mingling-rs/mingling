//! Model of a testable rust code block extracted from markdown: its dependency
//! configuration (features + deps) and the code itself.

use std::fmt::Write as _;
use std::path::Path;

/// A single testable `rust` code block, modeled as a test project.
pub(crate) struct MarkdownTestProject {
    pub features: Vec<String>,
    pub deps: Vec<(String, String)>,
    pub code: String,
    pub is_build_time: bool,
    pub has_main: bool,
    pub has_gen_program: bool,
    pub source_file: String,
    pub line: usize,
}

impl MarkdownTestProject {
    /// FNV-1a 64-bit hash over the dependency configuration (features + deps).
    ///
    /// Blocks with the same hash share one temporary crate and avoid redundant
    /// recompilation. The input is sorted so the hash is stable.
    #[must_use]
    pub fn compute_hash(&self) -> String {
        let mut features: Vec<&str> = self.features.iter().map(String::as_str).collect();
        features.sort_unstable();
        let mut dep_names: Vec<&str> = self.deps.iter().map(|(n, _)| n.as_str()).collect();
        dep_names.sort_unstable();
        let mut dep_versions: Vec<&str> = self.deps.iter().map(|(_, v)| v.as_str()).collect();
        dep_versions.sort_unstable();
        let mut deps: Vec<String> = self.deps.iter().map(|(n, v)| format!("{n}={v}")).collect();
        deps.sort();

        let canonical = format!(
            "{}\n{}\n{}\n{}",
            features.join(","),
            dep_names.join(","),
            dep_versions.join(","),
            deps.join(",")
        );

        // FNV-1a 64-bit — stable across runs (no random seed).
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        for &byte in canonical.as_bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        format!("{hash:016x}")
    }
}

/// Parses all fenced `rust` blocks from markdown content.
///
/// Blocks marked `// NOT VERIFIED` are skipped.
pub(crate) fn parse_markdown(content: &str, source_file: &str) -> Vec<MarkdownTestProject> {
    let mut projects = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        if lines[i].trim() == "```rust" {
            if let Some(proj) = parse_block(&lines, i, source_file) {
                projects.push(proj);
            }
            while i < lines.len() && lines[i].trim() != "```" {
                i += 1;
            }
        }
        i += 1;
    }
    projects
}

/// Parses a single code block starting at a `rust` fence line.
fn parse_block(lines: &[&str], start: usize, source_file: &str) -> Option<MarkdownTestProject> {
    let mut code_lines: Vec<String> = Vec::new();
    let mut features: Vec<String> = Vec::new();
    let mut not_verified = false;
    let mut deps: Vec<(String, String)> = Vec::new();
    let mut has_main = false;
    let mut has_gen_program = false;
    let mut is_build_time = false;

    let mut idx = start + 1;
    let mut in_header = true;

    while idx < lines.len() {
        let raw_line = lines[idx];
        let trimmed = raw_line.trim();

        if trimmed == "```" {
            break;
        }

        // `@@@` lines: hidden in the rendered docs (filtered by a docsify
        // plugin) but must still compile.
        if let Some(stripped) = trimmed.strip_prefix("@@@") {
            in_header = false;
            let code = stripped.trim_start();
            if code.contains("fn main") {
                has_main = true;
            }
            if code.contains("gen_program!") {
                has_gen_program = true;
            }
            code_lines.push(code.to_string());
            idx += 1;
            continue;
        }

        if in_header && trimmed == "// NOT VERIFIED" {
            not_verified = true;
            idx += 1;
            continue;
        }
        if in_header && trimmed == "// BUILD TIME" {
            is_build_time = true;
            idx += 1;
            continue;
        }
        if in_header && trimmed.starts_with("// ") {
            if let Some(feat_str) = trimmed.strip_prefix("// Features:") {
                let feat_str = feat_str.trim();
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
                while idx < lines.len() {
                    let next = lines[idx].trim();
                    if next == "```" {
                        break;
                    }
                    if let Some(dep_line) = next.strip_prefix("// ") {
                        if let Some((name, ver)) = dep_line.split_once(" = ") {
                            deps.push((
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

    if code_lines.is_empty() || not_verified {
        return None;
    }

    Some(MarkdownTestProject {
        features,
        deps,
        code: code_lines.join("\n"),
        is_build_time,
        has_main,
        has_gen_program,
        source_file: source_file.to_string(),
        line: start + 1,
    })
}

/// Builds the extra `[dependencies]` entries declared by a block's
/// `// Dependencies:` header comments.
///
/// Markdown blocks declare companion crates like this:
///
/// ```text
/// // Dependencies:
/// // serde = "1"
/// // clap = "4"
/// // tokio = { version = "1", features = ["full"] }
/// ```
///
/// Each `name = value` pair becomes one dependency of the generated test
/// crate (in addition to `mingling` itself), so doc blocks can freely use
/// external crates without repeating the whole manifest.
///
/// # Special case: serde / clap
///
/// Doc blocks pervasively derive serialization and argument parsing:
/// structural-renderer examples use `#[derive(Serialize)]`, the clap examples
/// use `#[derive(Parser)]` — and those derives live behind the `derive`
/// feature of `serde` / `clap`. Requiring every block to spell out
/// `// serde = { version = "1", features = ["derive"] }` would be
/// boilerplate repeated dozens of times, so the two crates automatically get
/// `features = ["derive"]` appended.
///
/// Version values starting with `{` are inline tables (e.g. `tokio` with a
/// `features` list above) and are passed through verbatim — they already
/// carry their own features and must not be rewritten.
fn build_extra_deps(proj: &MarkdownTestProject) -> String {
    let mut extra_deps = String::new();
    for (name, version) in &proj.deps {
        if version.starts_with('{') {
            // Inline table (path/features/…): the block already expressed its
            // full dependency, so emit it unchanged.
            let _ = writeln!(extra_deps, "{name} = {version}");
        } else if name == "serde" || name == "clap" {
            // serde/clap derive: `#[derive(Serialize, Deserialize)]` and
            // `#[derive(Parser)]` are used everywhere in the docs; auto-enable
            // the `derive` feature to keep blocks terse.
            let _ = writeln!(
                extra_deps,
                "{name} = {{ version = \"{version}\", features = [\"derive\"] }}"
            );
        } else {
            // Plain `name = "version"`.
            let _ = writeln!(extra_deps, "{name} = \"{version}\"");
        }
    }
    extra_deps
}

/// Generates the `Cargo.toml` for a project.
///
/// `manifest_path` is used to compute the relative path to the `mingling` crate.
pub(crate) fn generate_cargo_toml(proj: &MarkdownTestProject, manifest_path: &Path) -> String {
    let features_str = if proj.features.is_empty() {
        String::new()
    } else {
        let feats: Vec<String> = proj.features.iter().map(|f| format!("\"{f}\"")).collect();
        format!("features = [{}]", feats.join(", "))
    };

    let extra_deps = build_extra_deps(proj);

    let mingling_path = find_mingling_relative_path(manifest_path);
    let deps_section = if proj.features.is_empty() {
        format!("[dependencies]\nmingling = {{ path = \"{mingling_path}\" }}\n{extra_deps}")
    } else {
        format!(
            "[dependencies]\nmingling = {{ path = \"{mingling_path}\", {features_str} }}\n{extra_deps}"
        )
    };

    // Build-time projects mirror the features into [build-dependencies] so
    // build.rs sees the same feature set.
    let build_deps_section = if proj.is_build_time {
        let feats: Vec<String> = proj.features.iter().map(|f| format!("\"{f}\"")).collect();
        let build_feats = if feats.is_empty() {
            String::new()
        } else {
            format!("features = [{}]", feats.join(", "))
        };
        format!(
            "\n[build-dependencies]\nmingling = {{ path = \"{mingling_path}\", {build_feats} }}\n"
        )
    } else {
        String::new()
    };

    format!(
        r#"[package]
	name = "test-doc"
	version = "0.0.0"
	edition = "2024"

{deps_section}{build_deps_section}
[workspace]
"#
    )
}

/// Computes the relative path from a manifest's parent directory to `mingling`.
///
/// The process current directory is expected to be the project root.
fn find_mingling_relative_path(manifest_path: &Path) -> String {
    let manifest_dir = manifest_path
        .parent()
        .expect("manifest path has no parent directory");
    let cwd = std::env::current_dir().expect("failed to get current directory");

    let relative_to_root = manifest_dir.strip_prefix(&cwd).unwrap_or(manifest_dir);
    let depth = relative_to_root.components().count();

    let mut result = String::new();
    for _ in 0..depth {
        result.push_str("../");
    }
    result.push_str("mingling");
    result
}

/// Generates `main.rs` for a project.
///
/// Automatically prepends `use mingling::prelude::*;` and appends `fn main() {}`
/// and `gen_program!()` when the block does not provide them.
pub(crate) fn generate_main_rs(proj: &MarkdownTestProject) -> String {
    let mut output = String::from("#![allow(dead_code)]\n#![allow(unused)]\n");

    if !proj.code.contains("use mingling::prelude::*;") {
        output.push_str("#[allow(unused_imports)]\nuse mingling::prelude::*;\n\n");
    }
    output.push_str(&proj.code);
    output.push('\n');

    if !proj.has_main {
        output.push_str("\nfn main() {}\n");
    }
    if !proj.has_gen_program {
        output.push_str("\nmingling::macros::gen_program!();\n");
    }
    output
}

/// Generates `build.rs` for a build-time project: the code wrapped in
/// `fn main() { }` unless the block already provides one.
pub(crate) fn generate_build_rs(proj: &MarkdownTestProject) -> String {
    let mut output = String::from("#![allow(dead_code)]\n#![allow(unused)]\n");
    if proj.has_main {
        output.push_str(&proj.code);
    } else {
        output.push_str("fn main() {\n");
        for line in proj.code.lines() {
            output.push_str("    ");
            output.push_str(line);
            output.push('\n');
        }
        output.push_str("}\n");
    }
    output
}
