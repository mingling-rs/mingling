use crate::Next;
use crate::linter::mlint_report::{
    LintSpan, LintSpanLine, LintSuggestion, MlintLevel, MlintReport, StateLintReports,
};
use mingling::Routable;
use mingling::macros::{buffer, chain, dispatcher, pack, r_eprintln, renderer, routeify};
use mingling::res::{ResCurrentDir, ResExitCode};
use std::ops::Range;
use std::path::PathBuf;

const OVERRIDE_KEY: &str = "check.overrideCommand";
const EXPECTED_LINE: &str = r#"check.overrideCommand = ["mling", "ra-lint-check"]"#;
const VALID_FIRST: &[&str] = &["mling", "mingling-cli"];
const RA_CONFIG_TEMPLATE: &str = include_str!("../../tmpls/rust-analyzer.toml");

// Key names
const KEY_CHECK_ON_SAVE: &str = "checkOnSave";
const VALUE_CHECK_ON_SAVE_TRUE: &str = "true";
const DISPLAY_CHECK_ON_SAVE_TRUE: &str = "checkOnSave = true";

// File names
const SOURCE_FILE_NAME: &str = "rust-analyzer.toml";

const MSG_ALREADY_CORRECT: &str = "`.rust-analyzer.toml` already has the correct mling settings";
const MSG_NON_EMPTY_ARRAY: &str = "`check.overrideCommand`: expected a non-empty array";
const MSG_FIRST_ARG_INVALID: &str =
    "`check.overrideCommand`: first argument should be `mling` or `mingling-cli`";
const MSG_MISSING_SECOND: &str = "`check.overrideCommand`: missing second argument";
const MSG_SECOND_ARG_INVALID: &str =
    "`check.overrideCommand`: second argument should be a `ra-lint-*` subcommand or `lint`";
const MSG_MESSAGE_FORMAT_REQUIRED: &str =
    "`check.overrideCommand`: `lint` subcommand needs `--message-format=json`";

// Suggestions / replacements
const SUGGEST_RA_LINT_CHECK_ARRAY: &str = r#"["mling", "ra-lint-check"]"#;
const SUGGEST_MLING_QUOTED: &str = r#""mling""#;
const SUGGEST_RA_LINT_CHECK_QUOTED: &str = r#""ra-lint-check""#;
const SUGGEST_MESSAGE_FORMAT_JSON: &str = ", \"--message-format=json\"]";
const SUGGEST_OVERRIDE_LINE: &str = "overrideCommand = [\"mling\", \"ra-lint-check\"]\n";
const SUGGEST_OVERRIDE_FULL_LINE: &str = r#"check.overrideCommand = ["mling", "ra-lint-check"]"#;

// Subcommand constants
const SUB_CMD_LINT: &str = "lint";
const MESSAGE_FORMAT_FLAG: &str = "--message-format=json";

dispatcher!("lint-install", CMDLintInstall => EntryLintInstall);

pack!(StateWriteMlingLinterConfig = PathBuf);
pack!(StateSuggestMlingLinterSetup = ());
pack!(ResultMlingLinterConfigInstalled = PathBuf);

#[chain]
pub fn handle_lint_install(_: EntryLintInstall, current_dir: &ResCurrentDir) -> Next {
    let cfg_file_path = current_dir.join(SOURCE_FILE_NAME);

    if !cfg_file_path.exists() {
        return StateWriteMlingLinterConfig::new(cfg_file_path).to_chain();
    }

    StateSuggestMlingLinterSetup::new(()).to_chain()
}

#[chain(routeify)]
pub fn handle_state_write_mling_linter_config(prev: StateWriteMlingLinterConfig) -> Next {
    let cfg_file_path = prev.inner;
    std::fs::write(&cfg_file_path, RA_CONFIG_TEMPLATE)?;
    ResultMlingLinterConfigInstalled::new(cfg_file_path).into()
}

#[renderer(buffer)]
pub fn render_mling_linter_config_installed(result: ResultMlingLinterConfigInstalled) {
    let cfg_file_path = result.inner;
    r_eprintln!(
        "info: created `{}` with mling lint-integrated settings",
        cfg_file_path.display()
    );
}

#[chain]
pub fn handle_state_suggest_mling_linter_setup(
    _: StateSuggestMlingLinterSetup,
    current_dir: &ResCurrentDir,
    ec: &mut ResExitCode,
) -> StateLintReports {
    ec.exit_code = 1;

    let cfg_file_path = current_dir.join(SOURCE_FILE_NAME);
    let file_name = cfg_file_path.to_string_lossy().to_string();

    let content = match std::fs::read_to_string(&cfg_file_path) {
        Ok(c) => c,
        Err(e) => {
            return StateLintReports::new(vec![MlintReport {
                level: MlintLevel::Error,
                message: format!("failed to read `{file_name}`: {e}"),
                ..Default::default()
            }]);
        }
    };

    let mut reports: Vec<MlintReport> = vec![];
    reports.extend(check_simple_key(
        &content,
        KEY_CHECK_ON_SAVE,
        VALUE_CHECK_ON_SAVE_TRUE,
        DISPLAY_CHECK_ON_SAVE_TRUE,
        SOURCE_FILE_NAME,
    ));
    reports.extend(check_override_command(&content, SOURCE_FILE_NAME));

    if reports.is_empty() {
        reports.push(MlintReport {
            level: MlintLevel::Note,
            message: MSG_ALREADY_CORRECT.to_string(),
            ..Default::default()
        });
    }

    StateLintReports::new(reports)
}

/// A `MlintReport` at `Help` level with the given message, file, and source.
fn report_help(file_name: &str, source_code: &str, message: String) -> MlintReport {
    MlintReport {
        file_name: file_name.to_string(),
        source_code: source_code.to_string(),
        level: MlintLevel::Help,
        message,
        ..Default::default()
    }
}

/// Attach a single-line span + replace suggestion to a report.
fn with_replace_suggestion(
    report: MlintReport,
    line: usize,
    line_text: &str,
    byte_range: Range<usize>,
    replacement: String,
    label: Option<String>,
) -> MlintReport {
    let span = LintSpan {
        line_start: line,
        line_end: line,
        column_start: byte_range.start + 1,
        column_end: byte_range.end + 1,
        text: vec![LintSpanLine {
            text: line_text.to_string(),
            highlight_start: byte_range.start + 1,
            highlight_end: byte_range.end + 1,
        }],
        label,
    };
    let suggestion = LintSuggestion {
        source: line_text.to_string(),
        line_start: line,
        byte_range,
        replacement,
    };
    MlintReport {
        spans: vec![span],
        suggestions: vec![suggestion],
        ..report
    }
}

/// Attach an "insert new content" suggestion (byte_range 0..0) to a report.
fn with_insert_suggestion(report: MlintReport, line: usize, new_content: String) -> MlintReport {
    let suggestion = LintSuggestion {
        source: new_content.clone(),
        line_start: line,
        byte_range: 0..0,
        replacement: new_content,
    };
    MlintReport {
        suggestions: vec![suggestion],
        ..report
    }
}

fn check_simple_key(
    content: &str,
    key: &str,
    expected_val: &str,
    display_line: &str,
    source_file: &str,
) -> Vec<MlintReport> {
    let found = find_key_value(content, key);

    let matches = found
        .as_ref()
        .is_some_and(|(_, v)| collapse_whitespace(v) == collapse_whitespace(expected_val));

    if matches {
        return vec![];
    }

    let msg = format!("expected `{display_line}` in `rust-analyzer.toml`");
    let report = report_help(source_file, content, msg);

    match found {
        Some((ln, val)) => {
            // Key exists but value is wrong → suggest replacing the value
            let line_text = nth_line(content, ln);
            let byte_start = line_text.find(&val).unwrap_or(0);
            let byte_end = byte_start + val.len();
            vec![with_replace_suggestion(
                report,
                ln,
                &line_text,
                byte_start..byte_end,
                expected_val.to_string(),
                Some(format!("expected {expected_val}")),
            )]
        }
        None => {
            // Key missing entirely → suggest inserting line at end
            let insert_line = content.lines().count().max(1) + 1;
            let new_content = format!("{display_line}\n");
            vec![with_insert_suggestion(report, insert_line, new_content)]
        }
    }
}

fn check_override_command(content: &str, source_file: &str) -> Vec<MlintReport> {
    let mut reports = Vec::new();

    let Some((ln, val)) = find_key_value(content, OVERRIDE_KEY) else {
        // Setting entirely missing
        let report = report_help(
            source_file,
            content,
            format!("expected `{EXPECTED_LINE}` in `rust-analyzer.toml`"),
        );
        let (insert_line, new_content) = match find_table_header(content, "check") {
            Some(header_line) => (header_line + 1, SUGGEST_OVERRIDE_LINE.to_string()),
            None => (
                content.lines().count().max(1) + 1,
                format!("{SUGGEST_OVERRIDE_FULL_LINE}\n"),
            ),
        };
        reports.push(with_insert_suggestion(report, insert_line, new_content));
        return reports;
    };

    let line_text = nth_line(content, ln);
    let args = parse_array_items(&val);

    // First: must be `mling` or `mingling-cli`
    if !args
        .first()
        .is_some_and(|a| VALID_FIRST.contains(&a.as_str()))
    {
        let Some(first) = args.first() else {
            let report = report_help(source_file, content, MSG_NON_EMPTY_ARRAY.into());
            reports.push(with_replace_suggestion(
                report,
                ln,
                &line_text,
                0..val.len(),
                SUGGEST_RA_LINT_CHECK_ARRAY.into(),
                None,
            ));
            return reports;
        };
        let quoted = format!("\"{first}\"");
        let byte_start = line_text.find(&quoted).unwrap_or(0);
        let byte_end = byte_start + quoted.len();
        let report = report_help(source_file, content, MSG_FIRST_ARG_INVALID.into());
        reports.push(with_replace_suggestion(
            report,
            ln,
            &line_text,
            byte_start..byte_end,
            SUGGEST_MLING_QUOTED.into(),
            None,
        ));
        return reports;
    }

    // Second: must be `ra-lint-*` or `lint`
    let Some(second) = args.get(1) else {
        let report = report_help(source_file, content, MSG_MISSING_SECOND.into());
        reports.push(with_replace_suggestion(
            report,
            ln,
            &line_text,
            0..val.len(),
            SUGGEST_RA_LINT_CHECK_ARRAY.into(),
            None,
        ));
        return reports;
    };

    if !second.starts_with("ra-lint-") && second != SUB_CMD_LINT {
        let quoted = format!("\"{second}\"");
        let byte_start = line_text.find(&quoted).unwrap_or(0);
        let byte_end = byte_start + quoted.len();
        let report = report_help(source_file, content, MSG_SECOND_ARG_INVALID.into());
        reports.push(with_replace_suggestion(
            report,
            ln,
            &line_text,
            byte_start..byte_end,
            SUGGEST_RA_LINT_CHECK_QUOTED.into(),
            None,
        ));
        return reports;
    }

    // If second arg is `lint`, it must be followed by --message-format=json
    if second == SUB_CMD_LINT && !has_message_format_json(&args[2..]) {
        let byte_start = line_text
            .rfind(']')
            .unwrap_or(line_text.len().saturating_sub(1));
        let byte_end = byte_start + 1;
        let report = report_help(source_file, content, MSG_MESSAGE_FORMAT_REQUIRED.into());
        reports.push(with_replace_suggestion(
            report,
            ln,
            &line_text,
            byte_start..byte_end,
            SUGGEST_MESSAGE_FORMAT_JSON.into(),
            None,
        ));
    }

    reports
}

fn has_message_format_json(rest: &[String]) -> bool {
    rest.contains(&MESSAGE_FORMAT_FLAG.to_string())
        || rest
            .windows(2)
            .any(|w| w[0] == "--message-format" && w[1] == "json")
}

fn parse_array_items(s: &str) -> Vec<String> {
    let s = s.trim();
    if !s.starts_with('[') || !s.ends_with(']') {
        return vec![];
    }
    let inner = s[1..s.len() - 1].trim();
    if inner.is_empty() {
        return vec![];
    }
    let mut items = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;
    for ch in inner.chars() {
        match ch {
            '"' => in_quote = !in_quote,
            ',' if !in_quote => {
                let trimmed = current.trim().trim_matches('"').to_string();
                if !trimmed.is_empty() {
                    items.push(trimmed);
                }
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    let trimmed = current.trim().trim_matches('"').to_string();
    if !trimmed.is_empty() {
        items.push(trimmed);
    }
    items
}

/// Result of looking up a key=value pair in TOML content.
///
/// - `Some((line, value))` — found on that 1-based line with that value string.
/// - `None` — key not found.
type FoundKey = Option<(usize, String)>;

fn find_key_value(content: &str, dotted_key: &str) -> FoundKey {
    let parts: Vec<&str> = dotted_key.split('.').collect();
    let field = parts.last().copied().unwrap_or(dotted_key);
    let table_path = if parts.len() > 1 {
        &parts[..parts.len() - 1]
    } else {
        &[]
    };
    let table_path_str = if parts.len() > 1 {
        Some(parts[..parts.len() - 1].join("."))
    } else {
        None
    };

    let mut in_correct_table = table_path.is_empty();

    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        // Track table headers like [check]
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            let header = &trimmed[1..trimmed.len() - 1];
            in_correct_table = header.split('.').collect::<Vec<_>>() == table_path;
            continue;
        }

        let without_comment = trimmed.split('#').next().unwrap_or("").trim();
        if without_comment.is_empty() {
            continue;
        }

        if let Some(eq_pos) = without_comment.find('=') {
            let k = without_comment[..eq_pos].trim();
            let v = without_comment[eq_pos + 1..].trim();

            // Inside explicit [table] header
            if in_correct_table && k == field {
                return Some((i + 1, v.to_string()));
            }
            // Inline dotted key at root (e.g. `check.overrideCommand = ...`)
            if table_path_str.is_some() && k == dotted_key {
                return Some((i + 1, v.to_string()));
            }
        }
    }

    None
}

fn find_table_header(content: &str, table_name: &str) -> Option<usize> {
    let target = format!("[{table_name}]");
    content
        .lines()
        .position(|line| line.trim() == target)
        .map(|i| i + 1)
}

fn nth_line(content: &str, n: usize) -> String {
    content
        .lines()
        .nth(n.saturating_sub(1))
        .unwrap_or("")
        .to_string()
}

fn collapse_whitespace(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_space = false;
    for ch in s.chars() {
        if ch.is_whitespace() {
            if !in_space {
                out.push(' ');
                in_space = true;
            }
        } else {
            out.push(ch);
            in_space = false;
        }
    }
    out.trim().to_string()
}
