use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use colored::Colorize;
use just_fmt::snake_case;
use mingling::{
    Grouped, Routable,
    macros::{buffer, command, renderer},
    res::ResExitCode,
};

use crate::Next;
use crate::markdown::compare::{collect_md_files, compare_signature};
use crate::reporter::{self, ReportResult};
use crate::task::cmd_markdown_check::{ErrorMarkdownArgs, ErrorMarkdownConfig, stem_of};

const DOCS_DIR: &str = "./docs";
const LANG_CONFIG: &str = ".config/docs-lang.txt";

/// One file-pair outcome of a structure comparison.
struct CompareOutcome {
    item: String,
    location: String,
    ok: bool,
    output: String,
}

#[command(node = "markdown-compare")]
// `#[command]` rewrites an owned first param into the entry type, so the args
// must be passed by value even though the body only reads them.
#[allow(clippy::needless_pass_by_value)]
pub fn markdown_compare(args: Vec<String>) -> Next {
    let [ref_arg, trans_arg] = args.as_slice() else {
        return ErrorMarkdownArgs("missing <reference> and <translation> arguments".to_string())
            .to_chain();
    };
    let ref_path = cwd().join(ref_arg);
    let trans_path = cwd().join(trans_arg);

    reporter::set_task("Markdown-Compare");
    let outcomes = if ref_path.is_dir() && trans_path.is_dir() {
        compare_dirs(&ref_path, &trans_path, "doc")
    } else if ref_path.is_file() && trans_path.is_file() {
        compare_files(&ref_path, &trans_path, "doc")
    } else {
        return ErrorMarkdownArgs(
            "both arguments must be files or both must be directories".to_string(),
        )
        .to_chain();
    };
    let fail_count = export_outcomes(&outcomes);
    reporter::flush();

    ResultMarkdownCompare { fail_count }.to_chain()
}

#[command(node = "markdown-compare-all")]
pub fn markdown_compare_all() -> Next {
    let Some(langs) = lang_config() else {
        return ErrorMarkdownConfig.to_chain();
    };
    let Some(reference) = langs.first() else {
        return ErrorMarkdownConfig.to_chain();
    };
    let ref_dir = PathBuf::from(DOCS_DIR).join(reference);
    if !ref_dir.is_dir() {
        return ErrorMarkdownArgs(format!(
            "reference docs directory `{}` does not exist",
            ref_dir.display()
        ))
        .to_chain();
    }

    reporter::set_task("Markdown-Compare-All");
    let mut fail_count = 0;
    for lang in &langs[1..] {
        let lang_dir = PathBuf::from(DOCS_DIR).join(lang);
        if !lang_dir.is_dir() {
            eprintln!(
                "  {}: `{}` does not exist",
                "ERROR".bright_red(),
                lang_dir.display()
            );
            fail_count += 1;
            continue;
        }
        let outcomes = compare_dirs(&ref_dir, &lang_dir, &lang_key(lang));
        fail_count += export_outcomes(&outcomes);
    }
    reporter::flush();

    ResultMarkdownCompare { fail_count }.to_chain()
}

/// Compares one file pair (reference vs translation).
fn compare_files(ref_path: &Path, trans_path: &Path, prefix: &str) -> Vec<CompareOutcome> {
    let item = format!("{prefix}-{}", snake_case!(&stem_of(ref_path)));
    let location = trans_path.to_string_lossy().into_owned();
    match compare_signature(ref_path, trans_path) {
        Ok(()) => vec![CompareOutcome {
            item,
            location,
            ok: true,
            output: String::new(),
        }],
        Err(diffs) => vec![CompareOutcome {
            item,
            location,
            ok: false,
            output: diffs.join("\n"),
        }],
    }
}

/// Compares two directories: every `.md` file in the reference must exist in
/// the translation with the same structural signature; extra files are errors.
fn compare_dirs(ref_dir: &Path, trans_dir: &Path, prefix: &str) -> Vec<CompareOutcome> {
    let ref_files = collect_md_files(ref_dir);
    let ref_set: BTreeSet<PathBuf> = ref_files.iter().cloned().collect();
    let trans_set: BTreeSet<PathBuf> = collect_md_files(trans_dir).into_iter().collect();

    let mut outcomes = Vec::new();
    for file in ref_files {
        let item = format!("{prefix}-{}", snake_case!(&stem_of(&file)));
        let trans_path = trans_dir.join(&file);
        let location = trans_path.to_string_lossy().into_owned();
        if !trans_set.contains(&file) {
            outcomes.push(CompareOutcome {
                item,
                location,
                ok: false,
                output: "missing in translation".to_string(),
            });
            continue;
        }
        outcomes.push(match compare_signature(&ref_dir.join(&file), &trans_path) {
            Ok(()) => CompareOutcome {
                item,
                location,
                ok: true,
                output: String::new(),
            },
            Err(diffs) => CompareOutcome {
                item,
                location,
                ok: false,
                output: diffs.join("\n"),
            },
        });
    }

    for file in trans_set.difference(&ref_set) {
        let item = format!("{prefix}-{}", snake_case!(&stem_of(file)));
        let trans_path = trans_dir.join(file);
        outcomes.push(CompareOutcome {
            item,
            location: trans_path.to_string_lossy().into_owned(),
            ok: false,
            output: "extra file, not in reference".to_string(),
        });
    }
    outcomes
}

/// Exports the outcomes via `reporter`; failures also print to stderr.
fn export_outcomes(outcomes: &[CompareOutcome]) -> usize {
    let mut fail_count = 0;
    for outcome in outcomes {
        if outcome.ok {
            reporter::export(&outcome.item, &outcome.location, ReportResult::Ok);
        } else {
            fail_count += 1;
            eprintln!("  {} {}", "failed".bright_red(), outcome.item);
            eprintln!("  {}\n{}", outcome.location, outcome.output);
            reporter::export(
                &outcome.item,
                &outcome.location,
                ReportResult::Error(outcome.output.clone()),
            );
        }
    }
    fail_count
}

/// Reads `.config/docs-lang.txt`: the first line is the reference directory
/// (relative to `./docs/`), the rest are translations that must mirror it.
fn lang_config() -> Option<Vec<String>> {
    let content = std::fs::read_to_string(LANG_CONFIG).ok()?;
    Some(
        content
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(|l| l.trim_start_matches("./").to_string())
            .collect(),
    )
}

/// Turns a lang directory path into a report-item key, e.g.
/// `./_zh_CN/pages/` → `_zh_CN_pages`.
fn lang_key(lang: &str) -> String {
    lang.trim_matches('/').replace('/', "_")
}

fn cwd() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// Number of files that failed the structure comparison.
#[derive(Grouped)]
pub struct ResultMarkdownCompare {
    pub fail_count: usize,
}

/// Silently sets a non-zero exit code when any comparison failed.
#[renderer(buffer)]
pub fn render_markdown_compare(r: ResultMarkdownCompare, exit_code: &mut ResExitCode) {
    if r.fail_count > 0 {
        exit_code.exit_code = 1;
    }
}
