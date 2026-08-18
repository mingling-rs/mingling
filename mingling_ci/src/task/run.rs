use std::ffi::OsString;
use std::path::Path;

use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};

use crate::reporter::{self, ReportResult};

/// The manifest's parent directory, e.g. `./mingling` — the report location
/// for a crate-based item.
pub(crate) fn location(path: &Path) -> String {
    path.parent()
        .map_or_else(|| ".".to_string(), |d| d.to_string_lossy().into_owned())
}

/// Outcome of a `cargo` subcommand.
struct CargoResult {
    ok: bool,
    exit_code: Option<i32>,
    output: String,
}

/// Runs the given cargo task list in parallel.
///
/// Each task is an `(item, location, argv)` triple; progress and failures go
/// to stderr: a failing task prints its output immediately and writes its
/// report entry at the same time. Returns the number of failing tasks.
pub(crate) async fn run_parallel_checks(
    task: &str,
    phase: &str,
    tasks: Vec<(String, String, Vec<OsString>)>,
) -> usize {
    reporter::set_task(task);

    let n = tasks.len();
    let pb = ProgressBar::new(n as u64);
    let padding = " ".repeat(12usize.saturating_sub(phase.len()));
    let styled_prefix = format!("{}{}", padding, phase.bold().bright_cyan());
    pb.set_style(
        ProgressStyle::default_bar()
            .template(&format!(
                "{styled_prefix} [{{bar:28}}] {{pos}}/{{len}}: {{msg}}"
            ))
            .unwrap()
            .progress_chars("=> "),
    );

    // Run each task in parallel.
    let mut set = tokio::task::JoinSet::new();
    for (item, location, args) in tasks {
        set.spawn(async move { (item, location, run_cargo(args).await) });
    }

    let mut fail_count = 0;
    while let Some(joined) = set.join_next().await {
        let Ok((item, location, result)) = joined else {
            continue;
        };
        pb.inc(1);
        pb.set_message(item.clone());

        if result.ok {
            reporter::export(&item, &location, ReportResult::Ok);
        } else {
            fail_count += 1;
            // Failures print to stderr immediately (bar suspended to avoid
            // interleaving) and write their report entry at the same time.
            pb.suspend(|| {
                eprintln!(
                    "{}: {} failed{}",
                    phase.bold().bright_cyan(),
                    item,
                    result
                        .exit_code
                        .map_or_else(String::new, |c| format!(" (exit code {c})"))
                );
                for line in result.output.lines() {
                    eprintln!("  {line}");
                }
            });
            reporter::export(&item, &location, ReportResult::Error(result.output));
        }
    }

    pb.finish_and_clear();
    reporter::flush();
    fail_count
}

/// Runs a `cargo` subcommand, capturing its output.
/// Runs a cargo subcommand (`argv[0]` is the program), capturing its output.
async fn run_cargo(argv: Vec<OsString>) -> CargoResult {
    let mut argv = argv.into_iter();
    let Some(program) = argv.next() else {
        return CargoResult {
            ok: false,
            exit_code: None,
            output: "empty command".to_string(),
        };
    };

    let output = tokio::process::Command::new(program)
        .args(argv)
        .output()
        .await;

    match output {
        Ok(output) => {
            let mut log = String::from_utf8_lossy(&output.stdout).into_owned();
            log.push_str(&String::from_utf8_lossy(&output.stderr));
            CargoResult {
                ok: output.status.success(),
                exit_code: output.status.code(),
                output: log,
            }
        }
        Err(e) => CargoResult {
            ok: false,
            exit_code: None,
            output: format!("failed to run cargo: {e}"),
        },
    }
}
