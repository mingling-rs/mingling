use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};

use colored::Colorize;
use indicatif::ProgressBar;
use tools::verify::{
    build_block, compute_block_hash, generate_cargo_toml, generate_main_rs, is_block_testable,
    parse_code_blocks, write_summary_report,
};
use tools::{eprintln_cargo_style, println_cargo_style};

/// Config from verified-docs.toml
#[derive(serde::Deserialize)]
struct Config {
    verified: VerifiedPaths,
}

#[derive(serde::Deserialize)]
struct VerifiedPaths {
    readme: String,
    #[serde(default)]
    documents_en_us: Option<String>,
    #[serde(default)]
    documents_zh_cn: Option<String>,
}

#[tokio::main]
async fn main() {
    #[cfg(windows)]
    let _ = colored::control::set_virtual_terminal(true);

    let config_path = PathBuf::from("verified-docs.toml");
    if !config_path.exists() {
        eprintln_cargo_style!("verified-docs.toml not found in current directory");
        std::process::exit(1);
    }

    let config: Config = {
        let content = std::fs::read_to_string(&config_path).unwrap_or_else(|_e| {
            eprintln_cargo_style!("Failed to read verified-docs.toml");
            std::process::exit(1);
        });
        toml::from_str(&content).unwrap_or_else(|_e| {
            eprintln_cargo_style!("Failed to parse verified-docs.toml");
            std::process::exit(1);
        })
    };

    // Parse optional path argument from env args
    let single_file: Option<PathBuf> = {
        let args: Vec<String> = env::args().collect();
        if args.len() > 1 {
            let p = PathBuf::from(&args[1]);
            if p.exists() {
                Some(p)
            } else {
                eprintln_cargo_style!("error: specified file '{}' does not exist", args[1]);
                std::process::exit(1);
            }
        } else {
            None
        }
    };

    // Collect all markdown files from config
    let mut files: Vec<(String, PathBuf)> = Vec::new();

    // README
    let readme_path = PathBuf::from(&config.verified.readme);
    if readme_path.exists() {
        files.push(("README".to_string(), readme_path));
    }

    // English docs
    if let Some(pattern) = &config.verified.documents_en_us {
        let base = pattern.trim_end_matches("/**").trim_end_matches('*');
        let dir = PathBuf::from(base);
        if dir.exists() && dir.is_dir() {
            collect_md_files(&dir, &mut files, "en");
        }
    }

    // Chinese docs
    if let Some(pattern) = &config.verified.documents_zh_cn {
        let base = pattern.trim_end_matches("/**").trim_end_matches('*');
        let dir = PathBuf::from(base);
        if dir.exists() && dir.is_dir() {
            collect_md_files(&dir, &mut files, "zh_CN");
        }
    }

    // If a single file was specified, filter the list to only that file
    if let Some(ref target) = single_file {
        let target_canon = std::fs::canonicalize(target).unwrap_or_else(|_| target.clone());
        files.retain(|(_, path)| {
            std::fs::canonicalize(path)
                .map(|p| p == target_canon)
                .unwrap_or(false)
        });
        if files.is_empty() {
            eprintln_cargo_style!(
                "error: specified file '{}' is not among the configured documentation files",
                target.display()
            );
            std::process::exit(1);
        }
    }

    if files.is_empty() {
        eprintln_cargo_style!("No markdown files found to verify");
        std::process::exit(1);
    }

    // Parse all code blocks into a flat list with global indices
    let mut flat_blocks: Vec<(usize, tools::verify::CodeBlock)> = Vec::new();

    for (label, path) in &files {
        let content = std::fs::read_to_string(path).unwrap_or_else(|e| {
            eprintln_cargo_style!("Failed to read {}: {}", path.display(), e);
            String::new()
        });
        let source_file = format!("{label}/{}", path.file_name().unwrap().to_string_lossy());
        let blocks = parse_code_blocks(&content, &source_file);
        let testable: Vec<_> = blocks.into_iter().filter(is_block_testable).collect();
        for block in testable {
            let idx = flat_blocks.len() + 1; // 1-based global index
            flat_blocks.push((idx, block));
        }
    }

    let total_testable = flat_blocks.len();

    if total_testable == 0 {
        println_cargo_style!("No testable code blocks found");
        return;
    }

    // Create a shared progress bar
    let bar = ProgressBar::new(total_testable as u64);
    bar.set_style(
        indicatif::ProgressStyle::default_bar()
            .template(&format!(
                "{} [{{bar:28}}] {{pos}}/{{len}}: {{msg}}",
                "     Testing".bold().bright_cyan()
            ))
            .unwrap()
            .progress_chars("=> "),
    );
    bar.set_message("blocks");

    // Group blocks by dependency hash
    let mut groups: HashMap<String, Vec<(usize, tools::verify::CodeBlock)>> = HashMap::new();
    for (idx, block) in flat_blocks {
        let hash = compute_block_hash(&block);
        groups.entry(hash).or_default().push((idx, block));
    }

    let temp_base = PathBuf::from(".temp/doc-test");

    // Sort groups by hash for deterministic output order
    let mut group_vec: Vec<(String, Vec<(usize, tools::verify::CodeBlock)>)> =
        groups.into_iter().collect();
    group_vec.sort_by(|a, b| a.0.cmp(&b.0));

    // Spawn a blocking task per group — groups run in parallel, blocks within a group are serial
    let mut handles = Vec::new();
    for (hash, blocks) in group_vec {
        let temp_base = temp_base.clone();
        let bar = bar.clone(); // clone shares the same underlying progress
        let handle = tokio::task::spawn_blocking(move || {
            let crate_dir = temp_base.join(&hash);
            let src_dir = crate_dir.join("src");
            let manifest_path = crate_dir.join("Cargo.toml");

            // Generate a single Cargo.toml for the whole group (all blocks share same deps)
            let first_block = &blocks[0].1;
            let cargo_toml = generate_cargo_toml(first_block, "test-doc", &manifest_path);

            let mut group_results: Vec<(String, usize, bool, String)> = Vec::new();
            for (block_idx, block) in &blocks {
                let block_label =
                    format!("Block {block_idx} ({}:{})", block.source_file, block.line);

                bar.set_message(block_label.clone());

                let main_rs = generate_main_rs(block);
                let (ok, err) = build_block(&src_dir, &manifest_path, &cargo_toml, &main_rs);
                if ok {
                    bar.inc(1);
                } else {
                    bar.inc(1);
                    bar.println(format!(
                        "  {} {block_label}",
                        "failed".bold().bright_red()
                    ));
                    bar.println(format!("  {block_label} FAILED:\n{err}"));
                }
                group_results.push((block.source_file.clone(), block.line, ok, err));
            }
            group_results
        });
        handles.push(handle);
    }

    // Collect results from all groups
    let mut results: Vec<(String, usize, bool, String)> = Vec::new();
    let mut passed = 0usize;
    let mut failed = 0usize;

    for handle in handles {
        match handle.await {
            Ok(group_results) => {
                for (file, line, ok, err) in group_results {
                    if ok {
                        passed += 1;
                    } else {
                        failed += 1;
                    }
                    results.push((file, line, ok, err));
                }
            }
            Err(e) => {
                eprintln_cargo_style!("Task panicked: {}", e);
                std::process::exit(1);
            }
        }
    }

    bar.finish_and_clear();

    let result_msg = format!("Result: {passed}/{total_testable} blocks passed");
    println_cargo_style!(result_msg);

    write_summary_report(
        Path::new(".temp/DOCS-TEST-RESULT.md"),
        "Documentation Code Block Test Report",
        &results,
        total_testable,
        passed,
        failed,
    );

    if failed > 0 {
        let fail_msg = format!("{failed} block(s) failed to build");
        eprintln_cargo_style!(fail_msg);
        std::process::exit(1);
    }

    println_cargo_style!("Done: All verified code blocks build successfully!");
}

/// Recursively collect all `.md` files under a directory
fn collect_md_files(dir: &Path, files: &mut Vec<(String, PathBuf)>, lang: &str) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_md_files(&path, files, lang);
            } else if path.extension().is_some_and(|ext| ext == "md") {
                files.push((lang.to_string(), path));
            }
        }
    }
}
