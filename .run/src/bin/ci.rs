use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::exit;

use arg_picker::{Picker, macros::arg};
use tools::{
    cargo_tomls, crate_name_from, eprintln_cargo_style, println_cargo_style, run_cmd, run_parallel,
};

fn get_ignore_dirs() -> Vec<String> {
    vec![".temp".to_string()]
}

/// A single CI step, each individually toggleable via `--check-*`.
struct Checks {
    build: bool,
    clippy: bool,
    test: bool,
    arg_picker: bool,
    markdown_code: bool,
    examples: bool,
    docs_refresh: bool,
    docs_structure: bool,
    api_docs: bool,
}

impl Checks {
    fn any(&self) -> bool {
        self.build
            || self.clippy
            || self.test
            || self.arg_picker
            || self.markdown_code
            || self.examples
            || self.docs_refresh
            || self.docs_structure
            || self.api_docs
    }
}

fn print_help() {
    println!(
        r"
Usage: ci [options]
Options:
   -h, --help                 Print this help message
   -y                         Auto-confirm temporary commits
       --dirty                Run CI on dirty workspace (skip temp commit & clean check)
       --check-build          Build all crates
       --check-clippy         Run clippy on all crates (-D warnings)
       --check-test           Run unit tests for all crates
       --check-arg-picker     Test the arg-picker crate
       --check-markdown-code  Verify all *.md code blocks compile
       --check-examples       Test all examples
       --check-docs-refresh   Refresh docs and fail if the tree is contaminated
       --check-docs-structure Verify translated docs mirror the English structure
       --check-api-docs       Build API docs with docs.rs features

If no specific options are given, all checks are run.
    "
    );
}

fn main() {
    #[cfg(windows)]
    let _ = colored::control::set_virtual_terminal(true);
    println!("{}", include_str!("../../../docs/res/ci_banner.txt"));

    let (
        auto_yes,
        dirty,
        check_build,
        check_clippy,
        check_test,
        check_arg_picker,
        check_markdown_code,
        check_examples,
        check_docs_refresh,
        check_docs_structure,
        check_api_docs,
        help,
    ) = Picker::from_args()
        .pick_or_default(&arg![yes: bool, 'y'])
        .pick_or_default(&arg![dirty: bool])
        .pick_or_default(&arg![check_build: bool])
        .pick_or_default(&arg![check_clippy: bool])
        .pick_or_default(&arg![check_test: bool])
        .pick_or_default(&arg![check_arg_picker: bool])
        .pick_or_default(&arg![check_markdown_code: bool])
        .pick_or_default(&arg![check_examples: bool])
        .pick_or_default(&arg![check_docs_refresh: bool])
        .pick_or_default(&arg![check_docs_structure: bool])
        .pick_or_default(&arg![check_api_docs: bool])
        .pick_or_default(&arg![help: bool, 'h'])
        .unwrap();

    if help {
        print_help();
        return;
    }

    let checks = Checks {
        build: check_build,
        clippy: check_clippy,
        test: check_test,
        arg_picker: check_arg_picker,
        markdown_code: check_markdown_code,
        examples: check_examples,
        docs_refresh: check_docs_refresh,
        docs_structure: check_docs_structure,
        api_docs: check_api_docs,
    };
    let run_all = !checks.any();

    let needs_commit_temp = !dirty && !{ run_cmd!("git diff-index --quiet HEAD --").is_ok() };

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

    if let Err(exit_code) = ci(&checks, run_all) {
        restore_workspace(needs_commit_temp).unwrap();
        exit(exit_code)
    }

    if !dirty {
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

/// Run one CI step.
///
/// When `continue_on_error` is set (used for the documentation steps in
/// "run all" mode), a failing step is recorded and the remaining steps still
/// execute, so every problem is reported in a single run.
fn run_step(
    exit_code: &mut i32,
    phase: &str,
    step: fn() -> Result<(), i32>,
    continue_on_error: bool,
) -> Result<(), i32> {
    println_cargo_style!(phase);
    match step() {
        Ok(()) => Ok(()),
        Err(code) if continue_on_error => {
            *exit_code = (*exit_code).max(code);
            Ok(())
        }
        Err(code) => Err(code),
    }
}

fn ci(checks: &Checks, run_all: bool) -> Result<(), i32> {
    let mut exit_code = 0;

    if run_all || checks.build {
        run_step(
            &mut exit_code,
            "Phase: Scan and build all crates",
            build_all,
            false,
        )?;
    }
    if run_all || checks.clippy {
        run_step(
            &mut exit_code,
            "Phase: Run clippy for all crates",
            clippy_all,
            false,
        )?;
    }
    if run_all || checks.test {
        run_step(&mut exit_code, "Phase: Test all crates", test_all, false)?;
    }
    if run_all || checks.arg_picker {
        run_step(
            &mut exit_code,
            "Phase: Test arg picker",
            test_arg_picker,
            false,
        )?;
    }

    if run_all || checks.markdown_code {
        run_step(
            &mut exit_code,
            "Phase: Verify all *.md document code blocks are compilable",
            test_docs_code_blocks,
            run_all,
        )?;
    }
    if run_all || checks.examples {
        run_step(
            &mut exit_code,
            "Phase: Test all examples",
            test_examples,
            run_all,
        )?;
    }
    if run_all || checks.docs_refresh {
        run_step(
            &mut exit_code,
            "Phase: Check all documentation is up to date",
            docs_refresh,
            run_all,
        )?;
    }
    if run_all || checks.docs_structure {
        run_step(
            &mut exit_code,
            "Phase: Check translated docs structure consistency",
            docs_structure,
            run_all,
        )?;
    }
    if run_all || checks.api_docs {
        run_step(
            &mut exit_code,
            "Phase: Try Build API docs",
            deploy_api_docs,
            run_all,
        )?;
    }

    if exit_code != 0 {
        return Err(exit_code);
    }

    run_cmd!("git add --renormalize .")?;

    Ok(())
}

fn test_examples() -> Result<(), i32> {
    run_cmd!("cargo run --manifest-path .run/Cargo.toml --color always --bin test-examples")
}

fn test_docs_code_blocks() -> Result<(), i32> {
    run_cmd!(
        "cargo run --manifest-path .run/Cargo.toml --color always --bin test-all-markdown-code"
    )
}

/// Returns the manifest paths of all workspace members (via `cargo metadata --no-deps`).
///
/// These crates are tested/built/clipped together with `--workspace` so that
/// feature-gated code is covered, instead of relying on each crate's default features.
fn workspace_manifests() -> Vec<PathBuf> {
    let Ok(output) = tools::run_cmd_capture("cargo metadata --no-deps --format-version 1") else {
        return Vec::new();
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&output) else {
        return Vec::new();
    };
    json["packages"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|p| p["manifest_path"].as_str().map(PathBuf::from))
        .collect()
}

fn same_path(a: &Path, b: &Path) -> bool {
    let norm = |p: &Path| std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
    norm(a) == norm(b)
}

fn build_all() -> Result<(), i32> {
    let ignore_dirs = get_ignore_dirs();
    let cargo_tomls = cargo_tomls();
    let workspace_manifests = workspace_manifests();
    let mut tasks = Vec::new();

    // Workspace members: build with all documented features (same set used by cov-test)
    let features_arg = doc_features_arg();
    tasks.push((
        "Build: workspace".to_string(),
        "workspace".to_string(),
        format!("cargo build --workspace{features_arg} --color always"),
    ));

    for cargo_toml in cargo_tomls {
        let path = cargo_toml.parent().unwrap_or(Path::new(""));
        let path_str = path.to_string_lossy();
        if ignore_dirs.iter().any(|d| path_str.contains(d.as_str())) {
            continue;
        }
        if workspace_manifests
            .iter()
            .any(|m| same_path(m, &cargo_toml))
        {
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
    let workspace_manifests = workspace_manifests();
    let mut tasks = Vec::new();

    // Workspace members: clippy with all documented features
    let features_arg = doc_features_arg();
    tasks.push((
        "Clippy: workspace".to_string(),
        "workspace".to_string(),
        format!("cargo clippy --workspace{features_arg} --color always -- -D warnings"),
    ));

    for cargo_toml in cargo_tomls {
        let path = cargo_toml.parent().unwrap_or(Path::new(""));
        let path_str = path.to_string_lossy();
        if ignore_dirs.iter().any(|d| path_str.contains(d.as_str())) {
            continue;
        }
        if workspace_manifests
            .iter()
            .any(|m| same_path(m, &cargo_toml))
        {
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

/// ` --features "<docs.rs features>"` (empty string when unavailable)
fn doc_features_arg() -> String {
    match tools::read_features() {
        Ok(features) if !features.is_empty() => format!(" --features \"{}\"", features.join(",")),
        _ => String::new(),
    }
}

fn test_all() -> Result<(), i32> {
    let ignore_dirs = get_ignore_dirs();
    let cargo_tomls = cargo_tomls();
    let workspace_manifests = workspace_manifests();
    let mut tasks = Vec::new();

    // Workspace members: test with all documented features so that feature-gated
    // tests (comp/repl/picker/structural_renderer/...) are actually executed.
    // `arg-picker` is excluded here and tested separately via [`test_arg_picker`].
    let features_arg = doc_features_arg();
    tasks.push((
        "Test: workspace".to_string(),
        "workspace".to_string(),
        format!("cargo test --workspace{features_arg} --exclude arg-picker --color always"),
    ));

    for cargo_toml in cargo_tomls {
        let path = cargo_toml.parent().unwrap_or(Path::new(""));
        let path_str = path.to_string_lossy();
        if ignore_dirs.iter().any(|d| path_str.contains(d.as_str())) {
            continue;
        }
        if workspace_manifests
            .iter()
            .any(|m| same_path(m, &cargo_toml))
        {
            continue;
        }
        let label = format!("Test: {}", cargo_toml.to_string_lossy());
        let crate_name = crate_name_from(&cargo_toml);
        let cmd = format!(
            "cargo test --manifest-path {} --color always",
            cargo_toml.to_string_lossy()
        );
        tasks.push((label, crate_name, cmd));
    }
    run_parallel("Testing", tasks)
}

/// `arg-picker` is excluded from the workspace test command: when built with
/// `mingling_support` (enabled via `mingling/picker`), its README doctests
/// expand `arg!` to `::mingling::picker::PickerArg`, which is not available
/// inside the arg-picker crate itself. Test it separately with its default
/// features instead.
fn test_arg_picker() -> Result<(), i32> {
    run_cmd!("cargo test -p arg-picker --color always")
}

fn deploy_api_docs() -> Result<(), i32> {
    run_cmd!(
        "cargo run --manifest-path .run/Cargo.toml --color always --bin deploy-api-docs -- --docsrs"
    )
}

fn docs_refresh() -> Result<(), i32> {
    println_cargo_style!("Refresh: document at `./docs/`");

    run_cmd!("cargo run --manifest-path .run/Cargo.toml --bin docs-code-box-fix")?;
    run_cmd!("cargo run --manifest-path .run/Cargo.toml --bin docsify-sidebar-gen")?;
    run_cmd!("cargo run --manifest-path .run/Cargo.toml --bin refresh-docs")?;
    run_cmd!("cargo run --manifest-path .run/Cargo.toml --bin refresh-feature-mod")?;
    run_cmd!("cargo run --manifest-path .run/Cargo.toml --bin sync-examples")?;
    run_cmd!("cargo fmt")?;

    Ok(())
}

fn docs_structure() -> Result<(), i32> {
    println_cargo_style!("Check: docs structure consistency across languages");

    run_cmd!("cargo run --manifest-path .run/Cargo.toml --bin check-docs-structure")
}
