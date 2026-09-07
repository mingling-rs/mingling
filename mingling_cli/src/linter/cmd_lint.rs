use crate::linter::mlint_report::{MlintReport, StateLintReports};
use cargo_metadata::Metadata;
use mingling::consts::REMAINS;
use mingling::macros::{arg, chain, completion, dispatcher, metadata, suggest};
use mingling::metadata::Description;
use mingling::picker::parselib::ParserStyle;
use mingling::picker::{EntryPicker, PickerArg};
use mingling::{Grouped, Wrap};
use mingling::{LazyRes, ShellContext, Suggest};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tokio::task::JoinSet;

dispatcher!("lint", EntryLint);

const ARG_WITH_CHECKER: PickerArg<Option<String>> = arg![with_checker: Option<String>];

#[metadata(EntryLint)]
pub fn desc_lint() -> Description {
    "Mingling Linter".to_string().into()
}

/// Main linting function that processes all packages in the metadata.
///
/// Iterates through all packages and their targets (e.g., binaries, libraries, tests),
/// recursively expands each target's module tree into its Rust source files (`.rs`),
/// parses them into ASTs, runs lint checks, and enriches each report with metadata
/// information.
async fn linter_main(metadata: &Metadata) -> Vec<MlintReport> {
    let mut join_set = JoinSet::new();

    // A single source file to lint, attributed to one compilation target.
    struct FileTask {
        path: String,
        package_id: String,
        target_name: String,
        target_kind: Option<String>,
        target_src_path: String,
    }

    // Resolve every reachable module file for each target up front, deduplicating
    // files that belong to more than one target (e.g. a lib shared by two bins).
    let mut tasks: Vec<FileTask> = Vec::new();
    let mut seen = HashSet::new();
    for package in &metadata.packages {
        for target in &package.targets {
            let path = &target.src_path;
            // Only process Rust source files (with `.rs` extension).
            if !path.as_str().ends_with(".rs") {
                continue;
            }
            let path_str = path.as_str().to_string();
            let package_id = package.id.to_string();
            let target_name = target.name.clone();
            let target_kind = target.kind.first().map(|k| k.to_string());
            let target_src_path = path_str.clone();

            // Walk the module tree declared from this crate root so that module files
            // (e.g. a `mod foo` living in `foo.rs`) are linted too, not just the root.
            for file in collect_module_files(PathBuf::from(path)) {
                let file_str = file.to_string_lossy().into_owned();
                if !seen.insert(file) {
                    continue;
                }
                tasks.push(FileTask {
                    path: file_str,
                    package_id: package_id.clone(),
                    target_name: target_name.clone(),
                    target_kind: target_kind.clone(),
                    target_src_path: target_src_path.clone(),
                });
            }
        }
    }

    for task in tasks {
        join_set.spawn_blocking(move || {
            // Read the source file content.
            let source = std::fs::read_to_string(&task.path).ok()?;
            // Parse the source file into an AST.
            let ast = syn::parse_file(&source).ok()?;
            // Run all lint checks and collect reports.
            let reports = crate::lints::run_all_lints(&ast, &source);

            // Enrich each report with metadata information.
            let enriched: Vec<MlintReport> = reports
                .into_iter()
                .map(|mut r| {
                    r.file_name = task.path.clone();
                    r.source_code = source.clone();
                    r.package_id = Some(task.package_id.clone());
                    r.target_name = Some(task.target_name.clone());
                    r.target_kind = task.target_kind.clone();
                    r.target_src_path = Some(task.target_src_path.clone());
                    r
                })
                .collect();

            Some(enriched)
        });
    }

    let mut all_reports = Vec::new();
    while let Some(res) = join_set.join_next().await {
        // `spawn_blocking` panics are propagated, `None` means task skipped (read/parse failure).
        if let Ok(Some(reports)) = res {
            all_reports.extend(reports);
        }
    }

    all_reports
}

/// Recursively collect `file` and every module file reachable from it via
/// `mod name;` declarations (non-inline modules).
///
/// Mirrors the module-resolution rules used by `mingling_pathf`:
/// - `main.rs` / `lib.rs` / `mod.rs` keep the module base in their own directory;
/// - any other `xxx.rs` resolves its children from the sibling directory `xxx/`;
/// - a child `foo` is looked up as `{base}/foo.rs`, then `{base}/foo/mod.rs`.
fn collect_module_files(root: PathBuf) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut visited = HashSet::new();
    collect_from_file(&root, &mut visited, &mut files);
    files
}

fn collect_from_file(file: &Path, visited: &mut HashSet<PathBuf>, files: &mut Vec<PathBuf>) {
    if !visited.insert(file.to_path_buf()) {
        return;
    }
    files.push(file.to_path_buf());

    // Parse to discover child `mod` declarations; a file that cannot be read or
    // parsed (e.g. a `#[cfg(...)]`-gated module whose file is absent) is skipped
    // rather than aborting the whole traversal.
    let Ok(content) = std::fs::read_to_string(file) else {
        return;
    };
    let Ok(ast) = syn::parse_file(&content) else {
        return;
    };

    for item in &ast.items {
        // `mod foo;` declares an external module file; inline `mod foo { }` does not.
        if let syn::Item::Mod(item_mod) = item
            && item_mod.semi.is_some()
            && let Some(child) = resolve_module_file(file, &item_mod.ident.to_string())
        {
            collect_from_file(&child, visited, files);
        }
    }
}

/// Resolve the file backing the module `name` declared inside `parent_file`,
/// applying standard Rust module path rules. Returns `None` when no candidate exists.
fn resolve_module_file(parent_file: &Path, module_name: &str) -> Option<PathBuf> {
    let parent_dir = parent_file.parent()?;
    let file_stem = parent_file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    // Rust module resolution:
    // - `mod.rs`, `main.rs`, `lib.rs` -> base is their own directory;
    // - `aaa.rs` -> base is the sibling directory `aaa/`.
    let module_base = if matches!(file_stem, "mod" | "main" | "lib") {
        parent_dir.to_path_buf()
    } else {
        parent_dir.join(file_stem)
    };

    [
        module_base.join(format!("{module_name}.rs")),
        module_base.join(module_name).join("mod.rs"),
    ]
    .into_iter()
    .find(|path| path.is_file())
}

#[derive(Grouped, Wrap)]
pub struct StateBeginLinter(());

#[chain]
pub fn handle_lint(args: EntryLint) -> StateBeginLinter {
    let (with_checker, checker_args) = args
        .pick_or(&ARG_WITH_CHECKER, || Some("cargo,check".to_string()))
        .pick(&REMAINS)
        .unwrap();

    // If with_checker is not set, proceed directly to the mingling lint phase
    let Some(with_checker) = with_checker else {
        return StateBeginLinter(());
    };

    let with_checker: Vec<&str> = with_checker.split(',').collect();
    let checker_args: Vec<String> = checker_args.into();

    // Run the outer checker (e.g. cargo check) with output passed through directly
    execute_checker(&with_checker, checker_args.as_slice());

    StateBeginLinter(())
}

/// Run the outer checker (e.g. cargo check) with output passed through directly.
fn execute_checker(with_checker: &[&str], checker_args: &[String]) {
    if with_checker.is_empty() {
        return;
    }

    let checker_str = with_checker.join(" ");
    let args_str = checker_args.join(" ");
    let full_cmd = if args_str.is_empty() {
        checker_str
    } else {
        format!("{} {}", checker_str, args_str)
    };

    let mut cmd = if cfg!(target_os = "windows") {
        let mut c = std::process::Command::new("cmd");
        c.args(["/C", &full_cmd]);
        c
    } else {
        let mut c = std::process::Command::new("sh");
        c.args(["-c", &full_cmd]);
        c
    };

    // Pass through stdin/stdout/stderr so the user sees everything
    let _ = cmd.status();
}

#[chain]
pub async fn handle_state_begin_linter(
    _: StateBeginLinter,
    metadata: &mut LazyRes<crate::metadata::setup::ResMetadata>,
) -> StateLintReports {
    let metadata = metadata.get_ref().data();
    let reports = linter_main(metadata).await;
    StateLintReports(reports)
}

#[completion(EntryLint)]
pub fn complete_lint(ctx: ShellContext) -> Suggest {
    if mingling::picker::parselib::build_possible_flags(
        ParserStyle::global_style(),
        &ARG_WITH_CHECKER.into_info(),
    )
    .contains(&ctx.previous_word)
    {
        suggest! {
            "cargo,check": "Also run `cargo check` for checking",
            "cargo,clippy": "Also run `cargo clippy` for checking",
        }
    } else {
        suggest! {
            ARG_WITH_CHECKER: "Comma-separated Rust Analyzer-compatible checkers to also run, e.g. `cargo,check`"
        }
    }
}
