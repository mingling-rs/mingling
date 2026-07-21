use crate::linter::mlint_report::{MlintReport, StateLintReports};
use cargo_metadata::Metadata;
use mingling::LazyRes;
use mingling::consts::REMAINS;
use mingling::macros::{arg, chain, dispatcher, pack};
use mingling::picker::EntryPicker;

dispatcher!("lint", CMDMinglingLinter => EntryMinglingLinter);

/// Main linting function that processes all packages in the metadata.
///
/// Iterates through all packages and their targets (e.g., binaries, libraries, tests),
/// reads Rust source files (`.rs`), parses them into ASTs, runs lint checks,
/// and enriches each report with metadata information.
async fn linter_main(metadata: &Metadata) -> Vec<MlintReport> {
    // Vector to accumulate all lint reports
    let mut all_reports = Vec::new();

    // Iterate over all packages in the metadata
    for package in &metadata.packages {
        // Iterate over all targets within a package (e.g., bin, lib, test, etc.)
        for target in &package.targets {
            let path = &target.src_path;
            // Only process Rust source files (with `.rs` extension)
            if !path.as_str().ends_with(".rs") {
                continue;
            }
            // Read the source file content
            let source = match std::fs::read_to_string(path.as_str()) {
                Ok(s) => s,
                Err(_) => continue,
            };
            // Parse the source file into an AST (Abstract Syntax Tree)
            let ast = match syn::parse_file(&source) {
                Ok(f) => f,
                Err(_) => continue,
            };

            // Run all lint checks and collect reports
            let reports = crate::lints::run_all_lints(&ast, &source);
            //
            for mut r in reports {
                // Enrich report with metadata information
                r.file_name = path.as_str().to_string();
                r.source_code = source.clone();
                r.package_id = Some(package.id.to_string());
                r.target_name = Some(target.name.clone());
                r.target_kind = target.kind.first().map(|k| k.to_string());
                r.target_src_path = Some(path.as_str().to_string());
                all_reports.push(r);
            }
        }
    }

    all_reports
}

pack!(StateBeginLinter = ());

#[chain]
pub fn handle_lint(args: EntryMinglingLinter) -> StateBeginLinter {
    let (with_checker, checker_args) = args
        .pick_or(&arg![with_checker: Option<String>], || {
            Some("cargo,check".to_string())
        })
        .pick(&REMAINS)
        .unwrap();

    // If with_checker is not set, proceed directly to the mingling lint phase
    let Some(with_checker) = with_checker else {
        return StateBeginLinter::new(());
    };

    let with_checker: Vec<&str> = with_checker.split(',').collect();
    let checker_args: Vec<String> = checker_args.into();

    // Run the outer checker (e.g. cargo check) with output passed through directly
    execute_checker(&with_checker, checker_args.as_slice());

    StateBeginLinter::new(())
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
    StateLintReports::new(reports)
}
