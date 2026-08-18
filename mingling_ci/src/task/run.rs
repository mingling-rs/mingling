use std::ffi::OsString;
use std::path::Path;

use just_progress::progress::{self, ProgressInfo};

use crate::reporter::{self, ReportResult};
use crate::res::Manifests;

/// Runs one `cargo` subcommand per manifest in parallel, reporting each
/// outcome via `reporter` after the whole round finishes.
///
/// Only output is the progress bar; returns the number of failing packages.
#[allow(clippy::cast_precision_loss)] // counts are small
pub(crate) async fn run_parallel_checks(
    task: &str,
    status: &'static str,
    args_for: fn(&Path) -> Vec<OsString>,
    manifests: &Manifests,
) -> usize {
    reporter::set_task(task);
    let total = manifests.package_dirs.len();
    progress::update(task, 0.0, ProgressInfo::Info(status));

    // Run one cargo invocation per manifest in parallel.
    let mut set = tokio::task::JoinSet::new();
    for (name, path) in &manifests.package_dirs {
        let (name, path) = (name.clone(), path.clone());
        let args = args_for(&path);
        set.spawn(async move { (name, run_cargo(args).await) });
    }

    // Collect all outcomes first, then dump the report files in one round.
    let mut results: Vec<(String, ReportResult)> = Vec::new();
    let mut done = 0;
    while let Some(joined) = set.join_next().await {
        done += 1;
        let Ok((name, (ok, output))) = joined else {
            continue;
        };
        progress::update(task, done as f32 / total as f32, ProgressInfo::Info(status));
        results.push((
            name,
            if ok {
                ReportResult::Ok
            } else {
                ReportResult::Error(output)
            },
        ));
    }

    let fail_count = results
        .iter()
        .filter(|(_, r)| matches!(r, ReportResult::Error(_)))
        .count();
    for (name, result) in results {
        reporter::export(&name, result);
    }
    fail_count
}

/// Runs a `cargo` subcommand, returning success and captured output.
async fn run_cargo(args: Vec<OsString>) -> (bool, String) {
    let output = tokio::process::Command::new("cargo")
        .args(args)
        .output()
        .await;
    match output {
        Ok(output) => {
            let mut log = String::from_utf8_lossy(&output.stdout).into_owned();
            log.push_str(&String::from_utf8_lossy(&output.stderr));
            (output.status.success(), log)
        }
        Err(e) => (false, format!("failed to run cargo: {e}")),
    }
}
