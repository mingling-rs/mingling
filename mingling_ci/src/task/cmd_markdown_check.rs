use std::collections::HashMap;
use std::path::{Path, PathBuf};

use just_fmt::snake_case;
use mingling::{
    Grouped, RenderResult, Routable,
    macros::{buffer, command, renderer},
    res::ResExitCode,
};

use crate::Next;
use crate::markdown::project::parse_markdown;
use crate::markdown::test::{MarkdownBlockOutcome, try_test_markdown_project};
use crate::reporter::{self, ReportResult};
use crate::res::{CargoError, MessagePrinter};

const VERIFIED_DOCS: &str = ".config/verified-docs.toml";

#[command(node = "markdown-check")]
pub async fn markdown_check(args: Vec<String>) -> Next {
    let Some(path_str) = args.first() else {
        return ErrorMarkdownArgs("missing <path> argument".to_string()).to_chain();
    };
    let path =
        std::env::current_dir().map_or_else(|_| PathBuf::from(path_str), |cwd| cwd.join(path_str));
    if !path.is_file() {
        return ErrorMarkdownArgs(format!("{} is not a file", path.display())).to_chain();
    }
    let Ok(content) = std::fs::read_to_string(&path) else {
        return ErrorMarkdownArgs(format!("failed to read {}", path.display())).to_chain();
    };

    let location = path.to_string_lossy().into_owned();
    let item = format!("doc-{}", snake_case!(&stem_of(&path)));
    reporter::set_task("Markdown-Check");

    let projects = parse_markdown(&content, &location);
    let outcomes = try_test_markdown_project(projects).await;
    let file_info = HashMap::from([(location.clone(), (item, location))]);
    let fail_count = report_files(&outcomes, &file_info);
    reporter::flush();

    ResultMarkdownCheck { fail_count }.to_chain()
}

#[command(node = "markdown-check-all")]
pub async fn markdown_check_all() -> Next {
    let Some(files) = verified_md_files() else {
        return ErrorMarkdownConfig.to_chain();
    };
    reporter::set_task("Markdown-Check-All");

    // Collect all projects; remember each file's report identity
    // (`{key}-{snake_case(file_stem)}` -> location).
    let mut projects = Vec::new();
    let mut file_info: HashMap<String, (String, String)> = HashMap::new();
    for (label, path) in files {
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        let file_name = path.file_name().unwrap().to_string_lossy();
        let source_file = format!("{label}/{file_name}");
        let item = format!("{label}-{}", snake_case!(&stem_of(&path)));
        let location = path.to_string_lossy().into_owned();
        file_info.insert(source_file.clone(), (item, location));
        projects.extend(parse_markdown(&content, &source_file));
    }

    let outcomes = try_test_markdown_project(projects).await;
    let fail_count = report_files(&outcomes, &file_info);
    reporter::flush();

    ResultMarkdownCheck { fail_count }.to_chain()
}

/// The file name without extension, e.g. `README.md` → `README`.
pub(crate) fn stem_of(path: &Path) -> String {
    path.file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}

/// Exports one report entry per source file: `ok` when every block passed,
/// otherwise an error carrying the failed blocks' details.
fn report_files(
    outcomes: &[MarkdownBlockOutcome],
    file_info: &HashMap<String, (String, String)>,
) -> usize {
    let mut by_file: HashMap<&str, (bool, Vec<String>)> = HashMap::new();
    for outcome in outcomes {
        let (ok, outputs) = by_file
            .entry(outcome.source_file.as_str())
            .or_insert((true, Vec::new()));
        if !outcome.ok {
            *ok = false;
            outputs.push(format!(
                "{}:{}:\n{}",
                outcome.source_file, outcome.line, outcome.output
            ));
        }
    }

    let mut fail_count = 0;
    for (source_file, (ok, outputs)) in by_file {
        let Some((item, location)) = file_info.get(source_file) else {
            continue;
        };
        if ok {
            reporter::export(item, location, ReportResult::Ok);
        } else {
            fail_count += outputs.len();
            reporter::export(item, location, ReportResult::Error(outputs.join("\n\n")));
        }
    }
    fail_count
}

/// Reads `verified-docs.toml` and collects all `.md` files: single files,
/// directories, or `**` globs (walked from the base directory).
fn verified_md_files() -> Option<Vec<(String, PathBuf)>> {
    let content = std::fs::read_to_string(VERIFIED_DOCS).ok()?;
    let table: toml::Table = content.parse().ok()?;

    let mut files: Vec<(String, PathBuf)> = Vec::new();
    for (label, value) in table.get("verified")?.as_table()? {
        let value_str = value.as_str()?;
        let candidate = PathBuf::from(value_str);
        if candidate.is_dir() {
            collect_md_files(&candidate, &mut files, label);
        } else if candidate.is_file() {
            files.push((label.clone(), candidate));
        } else if candidate.extension().is_none() {
            // Glob like "docs/pages/**": walk the base directory.
            let base = PathBuf::from(value_str.trim_end_matches("/**").trim_end_matches('*'));
            if base.is_dir() {
                collect_md_files(&base, &mut files, label);
            }
        }
    }

    files.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
    Some(files)
}

/// Recursively collects all `.md` files under a directory.
fn collect_md_files(dir: &Path, files: &mut Vec<(String, PathBuf)>, label: &str) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_md_files(&path, files, label);
            } else if path.extension().is_some_and(|ext| ext == "md") {
                files.push((label.to_string(), path));
            }
        }
    }
}

/// Number of code blocks that failed to build.
#[derive(Grouped)]
pub struct ResultMarkdownCheck {
    pub fail_count: usize,
}

#[derive(Grouped, Default)]
pub struct ErrorMarkdownArgs(pub String);

#[derive(Grouped, Default)]
pub struct ErrorMarkdownConfig;

/// Silently sets a non-zero exit code when any block failed.
#[renderer(buffer)]
pub fn render_markdown_check(r: ResultMarkdownCheck, exit_code: &mut ResExitCode) {
    if r.fail_count > 0 {
        exit_code.exit_code = 1;
    }
}

#[renderer]
pub fn render_error_markdown_args(e: ErrorMarkdownArgs, error: &CargoError) -> RenderResult {
    let render_result = RenderResult::new();
    error.println(vec![e.0]);
    render_result
}

#[renderer]
pub fn render_error_markdown_config(_: ErrorMarkdownConfig, error: &CargoError) -> RenderResult {
    let render_result = RenderResult::new();
    error.println(vec![format!("failed to read {VERIFIED_DOCS}")]);
    render_result
}
