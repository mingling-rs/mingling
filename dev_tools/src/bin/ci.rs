use std::io::Write as _;
use std::process::exit;

use tools::{cargo_tomls, eprintln_cargo_style, println_cargo_style, run_cmd};

fn main() {
    #[cfg(windows)]
    let _ = colored::control::set_virtual_terminal(true);
    println!("{}", include_str!("../../../docs/res/ci_banner.txt"));

    let args: Vec<String> = std::env::args().collect();
    let auto_yes = args.iter().any(|a| a == "-y");

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

    if let Err(exit_code) = ci() {
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

fn ci() -> Result<(), i32> {
    build_all()?;
    clippy_all()?;
    test_all()?;
    test_examples()?;
    test_docs_code_blocks()?;
    docs_refresh()?;

    run_cmd!("git add --renormalize .")?;

    Ok(())
}

fn test_examples() -> Result<(), i32> {
    println_cargo_style!("Testing: examples");
    run_cmd!("cargo run --manifest-path dev_tools/Cargo.toml --bin test-examples")
}

fn test_docs_code_blocks() -> Result<(), i32> {
    println_cargo_style!("Testing: documentation code blocks");
    run_cmd!("cargo run --manifest-path dev_tools/Cargo.toml --bin test-all-markdown-code")
}

fn build_all() -> Result<(), i32> {
    let cargo_tomls = cargo_tomls();
    for cargo_toml in cargo_tomls {
        println_cargo_style!("Build: {}", cargo_toml.to_string_lossy());
        run_cmd!(
            "cargo build --manifest-path {}",
            cargo_toml.to_string_lossy()
        )?;
    }

    Ok(())
}

fn clippy_all() -> Result<(), i32> {
    let cargo_tomls = cargo_tomls();
    for cargo_toml in cargo_tomls {
        println_cargo_style!("Clippy: {}", cargo_toml.to_string_lossy());
        run_cmd!(
            "cargo clippy --manifest-path {} -- -D warnings",
            cargo_toml.to_string_lossy()
        )?;
    }

    Ok(())
}

fn test_all() -> Result<(), i32> {
    let cargo_tomls = cargo_tomls();
    for cargo_toml in cargo_tomls {
        println_cargo_style!("Testing: {}", cargo_toml.to_string_lossy());
        run_cmd!(
            "cargo test --manifest-path {}",
            cargo_toml.to_string_lossy()
        )?;
    }

    Ok(())
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
