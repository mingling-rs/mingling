use crate::linter::mlint_report::{MlintReport, StateLintReports};
use cargo_metadata::Metadata;
use mingling::consts::REMAINS;
use mingling::macros::{arg, chain, completion, dispatcher, metadata, suggest};
use mingling::metadata::Description;
use mingling::picker::parselib::ParserStyle;
use mingling::picker::{EntryPicker, PickerArg};
use mingling::{Grouped, Wrap};
use mingling::{LazyRes, ShellContext, Suggest};
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
/// reads Rust source files (`.rs`), parses them into ASTs, runs lint checks,
/// and enriches each report with metadata information.
async fn linter_main(metadata: &Metadata) -> Vec<MlintReport> {
    let mut join_set = JoinSet::new();

    for package in &metadata.packages {
        for target in &package.targets {
            let path = &target.src_path;
            // Only process Rust source files (with `.rs` extension)
            if !path.as_str().ends_with(".rs") {
                continue;
            }

            // Clone/move all data needed inside the blocking closure
            let path_str = path.as_str().to_string();
            let package_id = package.id.to_string();
            let target_name = target.name.clone();
            let target_kind = target.kind.first().map(|k| k.to_string());

            join_set.spawn_blocking(move || {
                // Read the source file content
                let source = std::fs::read_to_string(&path_str).ok()?;
                // Parse the source file into an AST
                let ast = syn::parse_file(&source).ok()?;
                // Run all lint checks and collect reports
                let reports = crate::lints::run_all_lints(&ast, &source);

                // Enrich each report with metadata information
                let enriched: Vec<MlintReport> = reports
                    .into_iter()
                    .map(|mut r| {
                        r.file_name = path_str.clone();
                        r.source_code = source.clone();
                        r.package_id = Some(package_id.clone());
                        r.target_name = Some(target_name.clone());
                        r.target_kind = target_kind.clone();
                        r.target_src_path = Some(path_str.clone());
                        r
                    })
                    .collect();

                Some(enriched)
            });
        }
    }

    let mut all_reports = Vec::new();
    while let Some(res) = join_set.join_next().await {
        // `spawn_blocking` panics are propagated, `None` means task skipped (read/parse failure)
        if let Ok(Some(reports)) = res {
            all_reports.extend(reports);
        }
    }

    all_reports
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
pub fn complete_lint(ctx: &ShellContext) -> Suggest {
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
