use std::io::Write as _;
use std::process::exit;

use tools::{
    cargo_tomls, crate_name_from, eprintln_cargo_style, println_cargo_style, run_cmd, run_parallel,
};

fn get_ignore_dirs() -> Vec<String> {
    vec![".temp".to_string(), "mling/res".to_string()]
}

fn print_help() {
    println!("Usage: ci [options]");
    println!();
    println!("Options:");
    println!("  -h, --help              Print this help message");
    println!("  -y                      Auto-confirm temporary commits");
    println!("  --refresh-docs          Refresh documentation files");
    println!("  --test-docs             Run documentation tests (build, clippy, test)");
    println!("  --test-codes            Test examples and documentation code blocks");
    println!();
    println!("If no specific options are given, all checks are run.");
}

fn main() {
    #[cfg(windows)]
    let _ = colored::control::set_virtual_terminal(true);
    println!("{}", include_str!("../../../docs/res/ci_banner.txt"));

    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "-h" || a == "--help") {
        print_help();
        return;
    }

    let auto_yes = args.iter().any(|a| a == "-y");

    let test_docs = args.iter().any(|a| a == "--test-docs");
    let refresh_docs = args.iter().any(|a| a == "--refresh-docs");
    let test_codes = args.iter().any(|a| a == "--test-codes");
    let any_specified = test_docs || refresh_docs || test_codes;
    let run_all = !any_specified;

    let needs_commit_temp = !{ run_cmd!("git diff-index --quiet HEAD --").is_ok() };

    if needs_commit_temp {
        if auto_yes {
            run_cmd!("git add .").unwrap();
            run_cmd!("git commit -m \"[DO NOT PUSH] CI TEMP [DO NOT PUSH]\"").unwrap();
        } else {
            print!("Working tree is not clean, temporarily commit? [y/N]:");
            std::io::stdout().flush().unwrap();
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            let input = input.trim();
            if input == "y" || input == "Y" || input == "yes" || input == "Yes" {
                run_cmd!("git add .").unwrap();
                run_cmd!("git commit -m \"[DO NOT PUSH] CI TEMP [DO NOT PUSH]\"").unwrap();
            } else {
                eprintln_cargo_style!("Aborting.");
                exit(2)
            }
        }
    }

    if let Err(exit_code) = ci(test_docs, test_codes, run_all) {
        restore_workspace(needs_commit_temp).unwrap();
        exit(exit_code)
    }

    let is_worktree_clean = run_cmd!("git diff-index --quiet HEAD --").is_ok();
    if !is_worktree_clean {
        eprintln_cargo_style!("The repository was contaminated during CI, failing!");

        // Print git status
        println!();
        let _ = run_cmd!("git status");

        if needs_commit_temp {
            restore_workspace(true).unwrap();
        }
        exit(1)
    }

    println_cargo_style!("Done: All check passed!");

    if needs_commit_temp {
        restore_workspace(true).unwrap();
    }
}

fn restore_workspace(undo_commit: bool) -> Result<(), i32> {
    run_cmd!("git reset --hard --quiet")?;
    if undo_commit {
        run_cmd!("git reset --soft HEAD~1 --quiet")?;
        run_cmd!("git reset --quiet")?;
    }
    Ok(())
}

fn ci(test_docs: bool, test_codes: bool, run_all: bool) -> Result<(), i32> {
    if run_all || test_codes {
        println_cargo_style!("Phase: Scan and build all crates");
        build_all()?;

        println_cargo_style!("Phase: Run clippy for all crates");
        clippy_all()?;

        println_cargo_style!("Phase: Test all crates");
        test_all()?;
    }

    if run_all || test_docs {
        println_cargo_style!("Phase: Test all examples");
        test_examples()?;

        println_cargo_style!("Phase: Verify all *.md document code blocks are compilable");
        test_docs_code_blocks()?;

        println_cargo_style!("Phase: Check all documentation is up to date");
        docs_refresh()?;
    }

    run_cmd!("git add --renormalize .")?;

    Ok(())
}

fn test_examples() -> Result<(), i32> {
    run_cmd!("cargo run --manifest-path dev_tools/Cargo.toml --color always --bin test-examples")
}

fn test_docs_code_blocks() -> Result<(), i32> {
    run_cmd!(
        "cargo run --manifest-path dev_tools/Cargo.toml --color always --bin test-all-markdown-code"
    )
}

fn build_all() -> Result<(), i32> {
    let ignore_dirs = get_ignore_dirs();
    let cargo_tomls = cargo_tomls();
    let mut tasks = Vec::new();
    for cargo_toml in cargo_tomls {
        let path = cargo_toml.parent().unwrap_or(std::path::Path::new(""));
        let path_str = path.to_string_lossy();
        if ignore_dirs.iter().any(|d| path_str.contains(d.as_str())) {
            continue;
        }
        let label = format!("Build: {}", cargo_toml.to_string_lossy());
        let crate_name = crate_name_from(&cargo_toml);
        let cmd = format!(
            "cargo build --manifest-path {} --color always",
            cargo_toml.to_string_lossy()
        );
        tasks.push((label, crate_name, cmd));
    }
    run_parallel("Building", tasks)
}

fn clippy_all() -> Result<(), i32> {
    let ignore_dirs = get_ignore_dirs();
    let cargo_tomls = cargo_tomls();
    let mut tasks = Vec::new();
    for cargo_toml in cargo_tomls {
        let path = cargo_toml.parent().unwrap_or(std::path::Path::new(""));
        let path_str = path.to_string_lossy();
        if ignore_dirs.iter().any(|d| path_str.contains(d.as_str())) {
            continue;
        }
        let label = format!("Clippy: {}", cargo_toml.to_string_lossy());
        let crate_name = crate_name_from(&cargo_toml);
        let cmd = format!(
            "cargo clippy --manifest-path {} --color always -- -D warnings",
            cargo_toml.to_string_lossy()
        );
        tasks.push((label, crate_name, cmd));
    }
    run_parallel("Clippy", tasks)
}

fn test_all() -> Result<(), i32> {
    let ignore_dirs = get_ignore_dirs();
    let cargo_tomls = cargo_tomls();
    let mut tasks = Vec::new();
    for cargo_toml in cargo_tomls {
        let path = cargo_toml.parent().unwrap_or(std::path::Path::new(""));
        let path_str = path.to_string_lossy();
        if ignore_dirs.iter().any(|d| path_str.contains(d.as_str())) {
            continue;
        }
        let label = format!("Testing: {}", cargo_toml.to_string_lossy());
        let crate_name = crate_name_from(&cargo_toml);
        let cmd = format!(
            "cargo test --manifest-path {} --color always",
            cargo_toml.to_string_lossy()
        );
        tasks.push((label, crate_name, cmd));
    }
    run_parallel("Testing", tasks)
}

fn docs_refresh() -> Result<(), i32> {
    println_cargo_style!("Refresh: document at `./docs/`");

    run_cmd!("cargo run --manifest-path dev_tools/Cargo.toml --bin docs-code-box-fix")?;
    run_cmd!("cargo run --manifest-path dev_tools/Cargo.toml --bin docsify-sidebar-gen")?;
    run_cmd!("cargo run --manifest-path dev_tools/Cargo.toml --bin refresh-docs")?;
    run_cmd!("cargo run --manifest-path dev_tools/Cargo.toml --bin refresh-feature-mod")?;
    run_cmd!("cargo run --manifest-path dev_tools/Cargo.toml --bin sync-examples")?;

    Ok(())
}
