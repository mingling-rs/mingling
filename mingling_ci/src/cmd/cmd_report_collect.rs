use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

use just_template::Template;
use mingling::{
    Grouped, RenderResult, Routable,
    macros::{buffer, command, r_println, renderer},
};

use crate::Next;
use crate::reporter::{COLLECT_DIR, REPORT_PATH};
use crate::res::{CargoError, Manifests, MessagePrinter, ResCollectLogs};

const REPORT_TEMPLATE: &str = include_str!("../../tmpls/report.md");
const TASK_SECTION_TEMPLATE: &str = include_str!("../../tmpls/task_section.md");

/// Maps a package to its per-OS pass/fail status.
type OsStatuses = BTreeMap<String, bool>;

/// A row in a task section: package name and its per-OS statuses.
type TaskRow<'a> = (&'a String, &'a OsStatuses);

/// Rows grouped by task name.
type RowsByTask<'a> = BTreeMap<&'a String, Vec<TaskRow<'a>>>;

#[command(node = "report-collect")]
pub fn report_collect(manifests: &Manifests, logs: &ResCollectLogs) -> Next {
    if !PathBuf::from(COLLECT_DIR).is_dir() {
        return ErrorNoCollectDir.to_chain();
    }

    // Group rows by task: task -> [(package, os_statuses)].
    let by_task: RowsByTask = logs.statuses.iter().fold(
        BTreeMap::new(),
        |mut acc, ((task, package), os_statuses)| {
            acc.entry(task).or_default().push((package, os_statuses));
            acc
        },
    );

    // Render one section per task (table rows + this task's failures).
    let mut fail_count = 0;
    let mut sections: Vec<HashMap<String, String>> = Vec::new();
    for (task, rows) in by_task {
        let mut row_arms = Vec::new();
        let mut fail_arms = Vec::new();
        for (package, os_statuses) in rows {
            row_arms.push(HashMap::from([
                ("package_name".to_string(), package.clone()),
                ("package_dir".to_string(), package_dir(manifests, package)),
                (
                    "pass_win".to_string(),
                    pass_cell(os_statuses.get("Windows")),
                ),
                (
                    "pass_linux".to_string(),
                    pass_cell(os_statuses.get("Linux")),
                ),
                ("pass_mac".to_string(), pass_cell(os_statuses.get("MacOS"))),
            ]));

            for (os, ok) in os_statuses {
                if !ok {
                    let stdout = logs
                        .err_outputs
                        .get(&(task.clone(), os.clone(), package.clone()))
                        .cloned()
                        .unwrap_or_default();
                    fail_arms.push(HashMap::from([
                        ("package_name".to_string(), package.clone()),
                        ("stdout".to_string(), stdout),
                    ]));
                    fail_count += 1;
                }
            }
        }

        let mut section = Template::from(TASK_SECTION_TEMPLATE);
        section.insert_param("task_name".to_string(), task.clone());
        *section.add_impl("rows".to_string()) = row_arms;
        *section.add_impl("fails".to_string()) = fail_arms;
        sections.push(HashMap::from([(
            "section".to_string(),
            section.expand().unwrap_or_default(),
        )]));
    }

    let mut template = Template::from(REPORT_TEMPLATE);

    template.insert_param("date".to_string(), logs.git.date.clone());
    template.insert_param("commit_hash".to_string(), logs.git.commit_hash.clone());
    *template.add_impl("task_sections".to_string()) = sections;

    let expanded = template.expand().unwrap_or_default();
    let output = PathBuf::from(REPORT_PATH);
    let parent = output.parent().expect("output path has a parent");

    if let Err(e) = std::fs::create_dir_all(parent).and_then(|()| std::fs::write(&output, expanded))
    {
        return ErrorReportWrite(format!("failed to write {}: {e}", output.display())).to_chain();
    }

    ResultCollectResults { output, fail_count }.to_chain()
}

/// Maps a package name to its manifest directory (e.g. `mingling` →
/// `./mingling`), or `—` when the manifest is unknown.
fn package_dir(manifests: &Manifests, package: &str) -> String {
    manifests
        .package_dirs
        .get(package)
        .and_then(|path| path.parent())
        .map_or_else(|| "—".to_string(), |dir| dir.to_string_lossy().into_owned())
}

fn pass_cell(status: Option<&bool>) -> String {
    match status {
        Some(true) => "✅".to_string(),
        Some(false) => "❌".to_string(),
        None => "—".to_string(),
    }
}

/// The generated report.
#[derive(Grouped)]
pub struct ResultCollectResults {
    pub output: PathBuf,
    pub fail_count: usize,
}

#[derive(Grouped, Default)]
pub struct ErrorNoCollectDir;

#[derive(Grouped, Default)]
pub struct ErrorReportWrite(pub String);

#[renderer(buffer)]
pub fn render_collect_results(r: ResultCollectResults) {
    r_println!("Collected {} failing logs", r.fail_count);
    r_println!("Report generated at {}", r.output.display());
}

#[renderer]
pub fn render_error_no_collect_dir(_: ErrorNoCollectDir, error: &CargoError) -> RenderResult {
    let render_result = RenderResult::new();
    error.println(vec![format!("No collect directory: {COLLECT_DIR}")]);
    render_result
}

#[renderer]
pub fn render_error_report_write(e: ErrorReportWrite, error: &CargoError) -> RenderResult {
    let render_result = RenderResult::new();
    error.println(vec![format!("Report: {}", e.0)]);
    render_result
}
