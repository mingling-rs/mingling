//! Structural comparison of markdown docs (reference vs translation).
//!
//! For each file pair the comparison uses a *structural signature*: one token
//! per line, classifying headings (both Markdown `#` and HTML `<hN>`), fenced
//! code blocks (including their language tag), `@@@` hidden-compilation lines,
//! blank lines, blockquotes, lists and plain text. Translated text is allowed
//! to differ; the structure is not.

use std::path::{Path, PathBuf};

/// Collects all `.md` files under `dir`, returned relative to it.
pub(crate) fn collect_md_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&current) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "md") {
                out.push(path.strip_prefix(dir).unwrap_or(&path).to_path_buf());
            }
        }
    }
    out.sort();
    out
}

/// Compares the structural signatures of two markdown files.
///
/// Returns the human-readable diff lines (up to a small window) on the first
/// structural difference.
pub(crate) fn compare_signature(ref_path: &Path, lang_path: &Path) -> Result<(), Vec<String>> {
    let ref_content = std::fs::read_to_string(ref_path).unwrap_or_default();
    let lang_content = std::fs::read_to_string(lang_path).unwrap_or_default();

    let ref_sig = signature_of(&ref_content);
    let lang_sig = signature_of(&lang_content);

    if ref_sig == lang_sig {
        return Ok(());
    }

    let ref_lines: Vec<&str> = ref_content.lines().collect();
    let lang_lines: Vec<&str> = lang_content.lines().collect();

    let mut diffs = Vec::new();
    let mut window = 0;
    let max = ref_sig.len().max(lang_sig.len());
    for i in 0..max {
        let ref_tok = ref_sig.get(i);
        let lang_tok = lang_sig.get(i);
        if ref_tok == lang_tok {
            continue;
        }
        if window >= 5 {
            diffs.push(format!("... ({}-line window truncated)", max - i));
            break;
        }
        window += 1;
        let ref_line = ref_lines.get(i).copied().unwrap_or("<missing>");
        let lang_line = lang_lines.get(i).copied().unwrap_or("<missing>");
        diffs.push(format!("line {}", i + 1));
        diffs.push(format!(
            "expect `{}` {}",
            token_label(ref_tok.map_or("<eof>", String::as_str)),
            display_line(ref_line)
        ));
        diffs.push(format!(
            "found  `{}` {}",
            token_label(lang_tok.map_or("<eof>", String::as_str)),
            display_line(lang_line)
        ));
        if ref_sig.len() != lang_sig.len() && window >= 5 {
            diffs.push(format!(
                "note: reference has {} lines, translation has {} lines",
                ref_sig.len(),
                lang_sig.len()
            ));
            break;
        }
    }
    if diffs.is_empty() {
        diffs.push("signatures differ in length (see line count note)".to_string());
    }
    Err(diffs)
}

/// Builds the structural signature of a markdown file.
fn signature_of(content: &str) -> Vec<String> {
    let mut sig = Vec::new();
    let mut in_fence = false;
    let mut fence_lang = String::new();

    for raw_line in content.lines() {
        let line = raw_line.trim();

        if in_fence {
            if line.starts_with("```") {
                in_fence = false;
                sig.push(format!("F:{fence_lang}"));
            } else if line.starts_with("@@@") {
                sig.push("A".to_string());
            } else if line.is_empty() {
                sig.push("B".to_string());
            } else {
                sig.push("P".to_string());
            }
            continue;
        }

        if line.starts_with("```") {
            in_fence = true;
            fence_lang = line.trim_start_matches("```").trim().to_string();
            sig.push(format!("F:{fence_lang}"));
        } else if line.starts_with('#') {
            let level = line.chars().take_while(|c| *c == '#').count();
            sig.push(format!("H{level}"));
        } else if line.starts_with("<h") || line.starts_with("</h") {
            // HTML headings (e.g. `<h1 align="center">` / `</h1>`)
            let level = line
                .trim_start_matches(['<', '/'])
                .chars()
                .next()
                .and_then(|c| c.to_digit(10))
                .unwrap_or(1);
            sig.push(format!("H{level}"));
        } else if line.starts_with("@@@") {
            sig.push("A".to_string());
        } else if line.is_empty() {
            sig.push("B".to_string());
        } else if line.starts_with('>') {
            sig.push("Q".to_string());
        } else if is_list_line(line) {
            sig.push("L".to_string());
        } else {
            sig.push("P".to_string());
        }
    }
    sig
}

/// Human-readable label for a structural token.
fn token_label(token: &str) -> String {
    match token {
        "B" => "blank".to_string(),
        "A" => "@@@".to_string(),
        "Q" => "quote".to_string(),
        "L" => "list".to_string(),
        "P" => "text".to_string(),
        t if t.starts_with('H') => format!("heading-{}", &t[1..]),
        t if t.starts_with("F:") => {
            let lang = &t[2..];
            if lang.is_empty() {
                "fence".to_string()
            } else {
                format!("fence:{lang}")
            }
        }
        _ => token.to_string(),
    }
}

/// Renders a source line for display: blank lines become `<blank>`.
fn display_line(line: &str) -> String {
    if line.trim().is_empty() {
        "<blank>".to_string()
    } else {
        truncate(line)
    }
}

fn truncate(line: &str) -> String {
    const MAX: usize = 60;
    if line.chars().count() <= MAX {
        line.to_string()
    } else {
        let cut: String = line.chars().take(MAX).collect();
        format!("{cut}...")
    }
}

fn is_list_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("- ")
        || trimmed.starts_with("* ")
        || trimmed.starts_with("+ ")
        || is_numbered_list(trimmed)
}

/// A numbered list item: `1. text`, `1) text`, `10. text`, ...
fn is_numbered_list(line: &str) -> bool {
    let digit_count = line.chars().take_while(char::is_ascii_digit).count();
    if digit_count == 0 {
        return false;
    }
    let rest = &line[digit_count..];
    (rest.starts_with(". ") || rest.starts_with(") "))
        && rest.chars().nth(1).is_some_and(|c| c == ' ' || c == '\t')
}
