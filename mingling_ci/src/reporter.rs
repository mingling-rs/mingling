//! Minimal log exporter for CI reports.
//!
//! Writes per-package results into `collect/{task}/{platform}/{package}.{ok|err}`
//! so that the [`crate::cmd::collect_results`] command can assemble the final
//! report. The task name is set once per CI phase via [`set_task`].

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{LazyLock, Mutex};

/// Root of the collected CI logs (relative to the repo root).
pub const COLLECT_DIR: &str = "./.temp/reports/collect";

/// Generated report output (relative to the repo root).
pub const REPORT_PATH: &str = "./.temp/reports/result.md";

/// The platform a package check ran on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
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

/// Successful package names pending a [`flush`], grouped by platform.
static OK_BUFFER: LazyLock<Mutex<HashMap<ReportPlatform, Vec<String>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Sets the task that subsequent [`export`] calls write under.
///
/// # Panics
///
/// Panics if the internal mutex is poisoned.
pub fn set_task(task: &str) {
    *CURRENT_TASK.lock().unwrap() = Some(task.to_string());
}

/// Exports one package result to `collect/{task}/{platform}/`.
///
/// Successes are buffered and written to the `ok` file by [`flush`]; failures
/// write `{package}.err` (with the output) immediately. Errors are reported to
/// stderr and otherwise ignored.
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

/// Exports one package result for a specific platform.
///
/// Successes are buffered and written to the `ok` file by [`flush`]; failures
/// write `{package}.err` (with the output) immediately. Errors are reported to
/// stderr and otherwise ignored.
///
/// # Panics
///
/// Panics if the internal task mutex is poisoned.
pub fn export_on(package: &str, platform: ReportPlatform, result: ReportResult) {
    match result {
        ReportResult::Ok => OK_BUFFER
            .lock()
            .unwrap()
            .entry(platform)
            .or_default()
            .push(package.to_string()),
        ReportResult::Error(output) => write_err(package, platform, output),
    }
}

/// Writes buffered successes to `collect/{task}.{platform}.ok`, one package per
/// line.
///
/// # Panics
///
/// Panics if the internal task mutex is poisoned.
pub fn flush() {
    let Some(task) = CURRENT_TASK.lock().unwrap().clone() else {
        eprintln!("reporter: no current task; call reporter::set_task first");
        return;
    };

    let buffered = std::mem::take(&mut *OK_BUFFER.lock().unwrap());
    if buffered.is_empty() {
        return;
    }

    if let Err(e) = fs::create_dir_all(COLLECT_DIR) {
        eprintln!("reporter: failed to create {COLLECT_DIR}: {e}");
        return;
    }

    for (platform, packages) in buffered {
        let content = if packages.is_empty() {
            String::new()
        } else {
            packages.join("\n") + "\n"
        };
        let platform_name = platform.dir_name();
        let path = Path::new(COLLECT_DIR).join(format!("{task}.{platform_name}.ok"));
        if let Err(e) = fs::write(&path, content) {
            eprintln!("reporter: failed to write {}: {e}", path.display());
        }
    }
}

/// Writes a failure entry to `collect/{task}.{platform}.{package}.err`.
fn write_err(package: &str, platform: ReportPlatform, output: String) {
    let Some(task) = CURRENT_TASK.lock().unwrap().clone() else {
        eprintln!("reporter: no current task; call reporter::set_task first");
        return;
    };

    if let Err(e) = fs::create_dir_all(COLLECT_DIR) {
        eprintln!("reporter: failed to create {COLLECT_DIR}: {e}");
        return;
    }

    let platform_name = platform.dir_name();
    let path = Path::new(COLLECT_DIR).join(format!("{task}.{platform_name}.{package}.err"));
    if let Err(e) = fs::write(&path, output) {
        eprintln!("reporter: failed to write {}: {e}", path.display());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_writes_ok_and_err_files() {
        set_task("reporter-test");
        let platform_name = current_platform().dir_name();
        let ok_path = Path::new(COLLECT_DIR).join(format!("reporter-test.{platform_name}.ok"));
        let err_path =
            Path::new(COLLECT_DIR).join(format!("reporter-test.{platform_name}.pkg-b.err"));
        fs::remove_file(&ok_path).ok();
        fs::remove_file(&err_path).ok();

        export("pkg-a", ReportResult::Ok);
        export("pkg-b", ReportResult::Error("boom".to_string()));
        flush();

        assert!(ok_path.is_file());
        assert_eq!(fs::read_to_string(&ok_path).unwrap(), "pkg-a\n");
        assert!(err_path.is_file());
        assert_eq!(fs::read_to_string(&err_path).unwrap(), "boom");

        fs::remove_file(ok_path).ok();
        fs::remove_file(err_path).ok();
    }
}
