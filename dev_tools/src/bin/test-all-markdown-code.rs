use std::path::{Path, PathBuf};

use colored::Colorize;
use tools::verify::{
    build_block, generate_cargo_toml, generate_main_rs, is_block_testable, parse_code_blocks,
    write_summary_report,
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

fn main() {
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

    // Collect all markdown files from config
    let mut files: Vec<(String, PathBuf)> = Vec::new();

    // README
    let readme_path = PathBuf::from(&config.verified.readme);
    if readme_path.exists() {
        files.push(("README".to_string(), readme_path));
        println_cargo_style!("Source: found README.md");
    }

    // English docs
    if let Some(pattern) = &config.verified.documents_en_us {
        let base = pattern.trim_end_matches("/**").trim_end_matches('*');
        let dir = PathBuf::from(base);
        if dir.exists() && dir.is_dir() {
            collect_md_files(&dir, &mut files, "en");
            println_cargo_style!("Source: found docs/pages/");
        }
    }

    // Chinese docs
    if let Some(pattern) = &config.verified.documents_zh_cn {
        let base = pattern.trim_end_matches("/**").trim_end_matches('*');
        let dir = PathBuf::from(base);
        if dir.exists() && dir.is_dir() {
            collect_md_files(&dir, &mut files, "zh_CN");
            println_cargo_style!("Source: found docs/_zh_CN/pages/");
        }
    }

    if files.is_empty() {
        eprintln_cargo_style!("No markdown files found to verify");
        std::process::exit(1);
    }

    // Ensure temp directory exists
    let temp_dir = PathBuf::from(".temp/docs-test");
    let _ = std::fs::remove_dir_all(&temp_dir);

    let mut all_blocks: Vec<(String, Vec<tools::verify::CodeBlock>)> = Vec::new();

    for (label, path) in &files {
        let content = std::fs::read_to_string(path).unwrap_or_else(|e| {
            eprintln_cargo_style!("Failed to read {}: {}", path.display(), e);
            String::new()
        });
        let source_file = format!("{label}/{}", path.file_name().unwrap().to_string_lossy());
        let blocks = parse_code_blocks(&content, &source_file);
        let testable: Vec<_> = blocks.into_iter().filter(is_block_testable).collect();
        if !testable.is_empty() {
            all_blocks.push((label.clone(), testable));
        }
    }

    let total_testable: usize = all_blocks.iter().map(|(_, b)| b.len()).sum();

    if total_testable == 0 {
        println_cargo_style!("No testable code blocks found");
        return;
    }

    println_cargo_style!(
        "Test: found {total_testable} testable code blocks across {} files",
        all_blocks.len()
    );

    // Build each block
    let mut block_index = 0usize;
    let mut passed = 0usize;
    let mut failed = 0usize;
    let mut results: Vec<(String, usize, bool, String)> = Vec::new();

    for (_label, blocks) in &all_blocks {
        for block in blocks {
            block_index += 1;

            let block_label = format!(
                "Block {} ({}:{})",
                block_index, block.source_file, block.line
            );
            print!("  Testing {block_label} ... ");

            let package_name = format!("test-block-{block_index}");

            let cargo_toml = generate_cargo_toml(block, &package_name);
            let main_rs = generate_main_rs(block);
            let src_dir = temp_dir.join("src");
            let manifest_path = temp_dir.join("Cargo.toml");

            let (ok, err) = build_block(&src_dir, &manifest_path, &cargo_toml, &main_rs);
            if ok {
                println!("{}", "passed".bold().bright_green());
                passed += 1;
                results.push((block.source_file.clone(), block.line, true, String::new()));
            } else {
                println!("{}", "failed".bold().bright_red());
                failed += 1;
                results.push((block.source_file.clone(), block.line, false, err.clone()));
                eprintln_cargo_style!(format!("  {block_label} FAILED:\n{err}"));
            }
        }
    }

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
