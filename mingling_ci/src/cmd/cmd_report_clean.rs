use std::path::PathBuf;

use mingling::{
    Grouped, RenderResult, Routable,
    macros::{buffer, command, r_println, renderer},
};

use crate::Next;
use crate::reporter::{COLLECT_DIR, REPORT_PATH};
use crate::res::{CargoError, MessagePrinter};

/// Removes collected logs and the generated report.
#[command(node = "report-clean")]
pub fn report_clean() -> Next {
    let mut removed = Vec::new();
    for path in [PathBuf::from(COLLECT_DIR), PathBuf::from(REPORT_PATH)] {
        match std::fs::remove_dir_all(&path).or_else(|_| std::fs::remove_file(&path)) {
            Ok(()) => removed.push(path),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => {
                return ErrorReportClean(format!("failed to remove {}: {e}", path.display()))
                    .to_chain();
            }
        }
    }
    ResultReportClean { removed }.to_chain()
}

/// Paths removed by `report-clean`.
#[derive(Grouped)]
pub struct ResultReportClean {
    pub removed: Vec<PathBuf>,
}

#[derive(Grouped, Default)]
pub struct ErrorReportClean(pub String);

#[renderer(buffer)]
pub fn render_report_clean(r: ResultReportClean) {
    if r.removed.is_empty() {
        r_println!("Report data already clean");
    } else {
        for path in r.removed {
            r_println!("Removed {}", path.display());
        }
    }
}

#[renderer]
pub fn render_error_report_clean(e: ErrorReportClean, error: &CargoError) -> RenderResult {
    let render_result = RenderResult::new();
    error.println(vec![format!("Report: {}", e.0)]);
    render_result
}
