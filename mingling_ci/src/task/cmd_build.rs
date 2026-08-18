use std::path::Path;

use just_progress::progress::{self, ProgressInfo};
use mingling::{
    Grouped, Routable,
    macros::{buffer, command, r_println, renderer},
};

use crate::Next;
use crate::reporter::{self, ReportResult};
use crate::res::Manifests;

#[command(node = "build-all")]
pub async fn build_all(manifests: &Manifests) -> Next {
    const TASK: &str = "Build-All";

    reporter::set_task(TASK);

    let total = manifests.package_dirs.len();
    progress::update(TASK, 0.0, ProgressInfo::Info("Building"));

    // Run one `cargo build` per manifest in parallel.
    let mut set = tokio::task::JoinSet::new();
    for (name, path) in &manifests.package_dirs {
        let (name, path) = (name.clone(), path.clone());
        set.spawn(async move { (name, run_cargo_build(&path).await) });
    }

    // Collect all outcomes first, then dump the report files in one round.
    let mut results: Vec<(String, ReportResult)> = Vec::new();
    let mut done = 0;
    while let Some(joined) = set.join_next().await {
        done += 1;
        let Ok((name, (ok, output))) = joined else {
            continue;
        };
        // The count is small, so the `usize -> f32` cast cannot lose precision.
        #[allow(clippy::cast_precision_loss)]
        let overall = done as f32 / total as f32;
        progress::update(TASK, overall, ProgressInfo::Info("Building"));
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

    ResultBuildAll { fail_count }.to_chain()
}

/// Runs `cargo build --manifest-path <path>`, returning success and output.
async fn run_cargo_build(path: &Path) -> (bool, String) {
    let output = tokio::process::Command::new("cargo")
        .args(["build", "--manifest-path"])
        .arg(path)
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

/// Number of packages that failed to build.
#[derive(Grouped)]
pub struct ResultBuildAll {
    pub fail_count: usize,
}

#[renderer(buffer)]
pub fn render_build_all(r: ResultBuildAll) {
    r_println!("Build-All: {} package(s) failed", r.fail_count);
}
