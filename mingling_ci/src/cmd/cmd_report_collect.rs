use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

use just_template::Template;
use mingling::{
    Grouped, RenderResult, Routable,
    macros::{buffer, command, r_println, renderer},
};

use crate::Next;
use crate::reporter::COLLECT_DIR;
use crate::res::{CargoError, Manifests, MessagePrinter, package_name};

const OUTPUT_PATH: &str = "./.temp/reports/result.md";
const REPORT_TEMPLATE: &str = include_str!("../../tmpls/report.md");
const TASK_SECTION_TEMPLATE: &str = include_str!("../../tmpls/task_section.md");

/// Maps a package to its per-OS pass/fail status.
type OsStatuses = BTreeMap<String, bool>;

/// A row in a task section: package name and its per-OS statuses.
type TaskRow<'a> = (&'a String, &'a OsStatuses);

/// Rows grouped by task name.
type RowsByTask<'a> = BTreeMap<&'a String, Vec<TaskRow<'a>>>;

/// Git commit date and short hash for the report.
struct GitInfo {
    date: String,
    commit_hash: String,
}

#[command(node = "report-collect")]
pub fn report_collect(manifests: &Manifests) -> Next {
    let Ok(task_entries) = std::fs::read_dir(COLLECT_DIR) else {
        return ErrorNoCollectDir.to_chain();
    };

    // Parse `collect/{task}/{os}/{package}.{ok|err}` and group them by
    // (task, package) -> os -> ok.
    let mut statuses: BTreeMap<(String, String), OsStatuses> = BTreeMap::new();
    for task_entry in task_entries.flatten() {
        if !task_entry.file_type().is_ok_and(|t| t.is_dir()) {
            continue;
        }
        let task = task_entry.file_name().to_string_lossy().into_owned();

        let Ok(os_entries) = std::fs::read_dir(task_entry.path()) else {
            continue;
        };
        for os_entry in os_entries.flatten() {
            if !os_entry.file_type().is_ok_and(|t| t.is_dir()) {
                continue;
            }
            let os = os_entry.file_name().to_string_lossy().into_owned();

            let Ok(files) = std::fs::read_dir(os_entry.path()) else {
                continue;
            };
            for file in files.flatten() {
                let file_name = file.file_name().to_string_lossy().into_owned();
                if let Some((package, ok)) = parse_log_name(&file_name) {
                    statuses
                        .entry((task.clone(), package))
                        .or_default()
                        .insert(os.clone(), ok);
                }
            }
        }
    }

    // Group rows by task: task -> [(package, os_statuses)].
    let by_task: RowsByTask = statuses.iter().fold(
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
                    let path = format!("{COLLECT_DIR}/{task}/{os}/{package}.err");
                    let stdout = strip_ansi(&std::fs::read_to_string(path).unwrap_or_default());
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

    let git = git_info();
    let mut template = Template::from(REPORT_TEMPLATE);
    template.insert_param("date".to_string(), git.date);
    template.insert_param("commit_hash".to_string(), git.commit_hash);
    *template.add_impl("task_sections".to_string()) = sections;

    let expanded = template.expand().unwrap_or_default();
    std::fs::create_dir_all(std::path::Path::new(OUTPUT_PATH).parent().unwrap()).ok();
    std::fs::write(OUTPUT_PATH, expanded).ok();

    ResultCollectResults {
        output: OUTPUT_PATH.into(),
        fail_count,
    }
    .to_chain()
}

/// Parses a `{package}.{ok|err}` file name into `(package, ok)`.
fn parse_log_name(file_name: &str) -> Option<(String, bool)> {
    file_name
        .strip_suffix(".ok")
        .map(|n| (n.to_string(), true))
        .or_else(|| {
            file_name
                .strip_suffix(".err")
                .map(|n| (n.to_string(), false))
        })
}

/// Strips ANSI escape sequences from `input`.
///
/// Handles CSI (`ESC [ ...`), OSC (`ESC ] ...` terminated by BEL or `ESC \`)
/// and other single-character escapes, while preserving UTF-8 text. Literal
/// `^[` (caret-bracket, produced by some terminal captures) is normalized to
/// `ESC` first.
fn strip_ansi(input: &str) -> String {
    // Normalize literal `^[` (0x5E 0x5B) to a real ESC byte.
    let normalized = input.replace("^[", "\u{1b}");
    let mut out = String::with_capacity(normalized.len());
    let mut rest = normalized.as_str();
    while let Some(idx) = rest.find('\u{1b}') {
        out.push_str(&rest[..idx]);
        rest = &rest[idx..];
        rest = &rest[ansi_len(rest)..];
    }
    out.push_str(rest);
    out
}

/// Byte length of the ANSI escape sequence starting at `s[0]` (`s[0]` is `ESC`).
fn ansi_len(s: &str) -> usize {
    let b = s.as_bytes();
    match b.get(1) {
        Some(b'[') => {
            // CSI: `ESC [` params/intermediates (0x20-0x3F) then a final byte (0x40-0x7E).
            let mut i = 2;
            while i < b.len() {
                let byte = b[i];
                i += 1;
                if (0x40..=0x7E).contains(&byte) {
                    break;
                }
                if !(0x20..=0x3F).contains(&byte) {
                    break;
                }
            }
            i
        }
        Some(b']') => {
            // OSC: `ESC ]` ... terminated by BEL (0x07) or `ESC \`.
            let mut i = 2;
            while i < b.len() {
                let byte = b[i];
                i += 1;
                if byte == 0x07 {
                    break;
                }
                if byte == 0x1b {
                    if b.get(i) == Some(&b'\\') {
                        i += 1;
                    }
                    break;
                }
            }
            i
        }
        Some(_) => 2.min(b.len()),
        None => 1,
    }
}

/// Maps a package name to its manifest directory (e.g. `mingling` →
/// `./mingling`), or `—` when the manifest is unknown.
fn package_dir(manifests: &Manifests, package: &str) -> String {
    manifests
        .path
        .iter()
        .find(|path| package_name(path) == package)
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

/// Commit date (`YYYY-MM-DD`) and short commit hash; empty on failure.
fn git_info() -> GitInfo {
    let run = |args: &[&str]| {
        std::process::Command::new("git")
            .args(args)
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default()
    };
    GitInfo {
        date: run(&["log", "-1", "--format=%cs"]),
        commit_hash: run(&["rev-parse", "--short", "HEAD"]),
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

#[cfg(test)]
mod tests {
    use super::strip_ansi;

    #[test]
    fn strips_csi_and_osc_and_literal_caret() {
        let input =
            "\u{1b}[1m\u{1b}[92mok\u{1b}[0m \u{1b}]8;;https://x\u{1b}\\done\u{1b}]8;;\u{1b}\\\n";
        assert_eq!(strip_ansi(input), "ok done\n");

        // Literal `^[` (caret-bracket) captured by some terminals.
        assert_eq!(strip_ansi("^[[31mred^[[0m"), "red");
    }

    #[test]
    fn preserves_utf8() {
        assert_eq!(strip_ansi("你好\u{1b}[1m世界！\u{1b}[0m"), "你好世界！");
    }
}
