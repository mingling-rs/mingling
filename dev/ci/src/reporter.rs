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

/// Pending success entries: `(item, location)`.
type PendingOk = (String, String);

/// Successful items pending a [`flush`], grouped by platform.
static OK_BUFFER: LazyLock<Mutex<HashMap<ReportPlatform, Vec<PendingOk>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Sets the task that subsequent [`export`] calls write under.
///
/// # Panics
///
/// Panics if the internal mutex is poisoned.
pub fn set_task(task: &str) {
    *CURRENT_TASK.lock().unwrap() = Some(task.to_string());
}

/// Exports one item result.
///
/// `item` and `location` are free-form strings chosen by the generator.
/// Successes are buffered and written to the `ok` file by [`flush`]; failures
/// write `{task}.{platform}.{item}.err` immediately (first line is the
/// location). Errors are reported to stderr and otherwise ignored.
///
/// # Panics
///
/// Panics if the internal task mutex is poisoned.
pub fn export(item: &str, location: &str, result: ReportResult) {
    export_on(item, location, current_platform(), result);
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

/// Exports one item result for a specific platform.
///
/// `item` and `location` are free-form strings chosen by the generator.
/// Successes are buffered and written to the `ok` file by [`flush`]; failures
/// write `{task}.{platform}.{item}.err` immediately (first line is the
/// location). Errors are reported to stderr and otherwise ignored.
///
/// # Panics
///
/// Panics if the internal task mutex is poisoned.
pub fn export_on(item: &str, location: &str, platform: ReportPlatform, result: ReportResult) {
    match result {
        ReportResult::Ok => OK_BUFFER
            .lock()
            .unwrap()
            .entry(platform)
            .or_default()
            .push((item.to_string(), location.to_string())),
        ReportResult::Error(output) => write_err(item, location, platform, &output),
    }
}

/// Writes buffered successes to `collect/{task}.{platform}.ok`, one `item` (or
/// `item = location`) per line.
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

    for (platform, items) in buffered {
        let lines: Vec<String> = items
            .iter()
            .map(|(item, location)| {
                if location.is_empty() {
                    item.clone()
                } else {
                    format!("{item} = {location}")
                }
            })
            .collect();
        let content = if lines.is_empty() {
            String::new()
        } else {
            lines.join("\n") + "\n"
        };
        let platform_name = platform.dir_name();
        let path = Path::new(COLLECT_DIR).join(format!("{task}.{platform_name}.ok"));
        if let Err(e) = fs::write(&path, content) {
            eprintln!("reporter: failed to write {}: {e}", path.display());
        }
    }
}

/// Writes a failure entry to `collect/{task}.{platform}.{item}.err`, with the
/// location as the first line (empty when unknown).
fn write_err(item: &str, location: &str, platform: ReportPlatform, output: &str) {
    let Some(task) = CURRENT_TASK.lock().unwrap().clone() else {
        eprintln!("reporter: no current task; call reporter::set_task first");
        return;
    };

    if let Err(e) = fs::create_dir_all(COLLECT_DIR) {
        eprintln!("reporter: failed to create {COLLECT_DIR}: {e}");
        return;
    }

    let platform_name = platform.dir_name();
    let path = Path::new(COLLECT_DIR).join(format!("{task}.{platform_name}.{item}.err"));
    if let Err(e) = fs::write(&path, format!("{location}\n{output}")) {
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

        export("pkg-a", "./pkg-a", ReportResult::Ok);
        export("pkg-b", "./pkg-b", ReportResult::Error("boom".to_string()));
        export("pkg-c", "", ReportResult::Ok); // no location
        flush();

        assert!(ok_path.is_file());
        assert_eq!(
            fs::read_to_string(&ok_path).unwrap(),
            "pkg-a = ./pkg-a\npkg-c\n"
        );
        assert!(err_path.is_file());
        assert_eq!(fs::read_to_string(&err_path).unwrap(), "./pkg-b\nboom");

        fs::remove_file(ok_path).ok();
        fs::remove_file(err_path).ok();
    }
}
