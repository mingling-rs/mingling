//! Minimal log exporter for CI reports.
//!
//! Writes per-package results into `collect/{task}/{platform}/{package}.{ok|err}`
//! so that the [`crate::cmd::collect_results`] command can assemble the final
//! report. The task name is set once per CI phase via [`set_task`].

use std::fs;
use std::path::Path;
use std::sync::Mutex;

/// Root of the collected CI logs (relative to the repo root).
pub const COLLECT_DIR: &str = "./.temp/reports/collect";

/// Generated report output (relative to the repo root).
pub const REPORT_PATH: &str = "./.temp/reports/result.md";

/// The platform a package check ran on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportPlatform {
    Windows,
    Linux,
    MacOS,
}

impl ReportPlatform {
    /// Directory name used under the task folder.
    const fn dir_name(self) -> &'static str {
        match self {
            Self::Windows => "Windows",
            Self::Linux => "Linux",
            Self::MacOS => "MacOS",
        }
    }
}

/// The outcome of a package check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReportResult {
    /// Check passed.
    Ok,
    /// Check failed, with the captured output.
    Error(String),
}

/// Current task name (e.g. `Build-All`); set via [`set_task`].
static CURRENT_TASK: Mutex<Option<String>> = Mutex::new(None);

/// Sets the task that subsequent [`export`] calls write under.
///
/// # Panics
///
/// Panics if the internal mutex is poisoned.
pub fn set_task(task: &str) {
    *CURRENT_TASK.lock().unwrap() = Some(task.to_string());
}

/// Exports one package result to `collect/{task}/{platform}/{package}.{ok|err}`.
///
/// The platform is inferred from the current build target using `#[cfg]`
/// attributes, so callers don't need to pass it explicitly.
///
/// Writes `{package}.ok` on success and `{package}.err` (with the output) on
/// failure. Errors are reported to stderr and otherwise ignored.
///
/// # Panics
///
/// Panics if the internal task mutex is poisoned.
pub fn export(package: &str, result: ReportResult) {
    export_on(package, current_platform(), result);
}

/// The `ReportPlatform` for the currently compiling target.
fn current_platform() -> ReportPlatform {
    if cfg!(target_os = "windows") {
        ReportPlatform::Windows
    } else if cfg!(target_os = "macos") {
        ReportPlatform::MacOS
    } else {
        ReportPlatform::Linux
    }
}

/// Exports one package result to `collect/{task}/{platform}/{package}.{ok|err}`
/// for a specific platform.
///
/// Writes `{package}.ok` on success and `{package}.err` (with the output) on
/// failure. Errors are reported to stderr and otherwise ignored.
///
/// # Panics
///
/// Panics if the internal task mutex is poisoned.
pub fn export_on(package: &str, platform: ReportPlatform, result: ReportResult) {
    let Some(task) = CURRENT_TASK.lock().unwrap().clone() else {
        eprintln!("reporter: no current task; call reporter::set_task first");
        return;
    };

    let dir = Path::new(COLLECT_DIR).join(task).join(platform.dir_name());
    if let Err(e) = fs::create_dir_all(&dir) {
        eprintln!("reporter: failed to create {}: {e}", dir.display());
        return;
    }

    let (file_name, content) = match result {
        ReportResult::Ok => (format!("{package}.ok"), String::new()),
        ReportResult::Error(output) => (format!("{package}.err"), output),
    };
    let path = dir.join(file_name);
    if let Err(e) = fs::write(&path, content) {
        eprintln!("reporter: failed to write {}: {e}", path.display());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_writes_ok_and_err_files() {
        set_task("reporter-test");
        let task_root = Path::new(COLLECT_DIR).join("reporter-test");
        let dir = task_root.join("Linux");
        fs::remove_dir_all(&task_root).ok();

        export("pkg-a", ReportResult::Ok);
        export("pkg-b", ReportResult::Error("boom".to_string()));

        assert!(dir.join("pkg-a.ok").is_file());
        assert!(dir.join("pkg-b.err").is_file());
        assert_eq!(fs::read_to_string(dir.join("pkg-b.err")).unwrap(), "boom");

        fs::remove_dir_all(&task_root).ok();
    }
}
