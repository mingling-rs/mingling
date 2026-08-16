//! Checks that every translated docs directory mirrors the structure of the
//! reference (English) docs directory exactly.
//!
//! The language directories are declared in `.config/docs-lang.txt`, one path
//! per line (relative to `./docs/`). The first line is the reference
//! directory; every other line is a translation that must match it.
//!
//! For each file pair the tool compares a *structural signature*: one token per
//! line, classifying headings (both Markdown `#` and HTML `<hN>`), fenced code
//! blocks (including their language tag), `@@@` hidden-compilation lines, blank
//! lines, blockquotes, lists and plain text. Translated text is allowed to
//! differ; the structure is not.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use colored::Colorize;
use tools::println_cargo_style;

const DOCS_DIR: &str = "./docs";
const LANG_CONFIG: &str = ".config/docs-lang.txt";

fn main() {
    println_cargo_style!("Checking: docs structure consistency across languages ...");

    let repo_root = find_git_repo().expect("Cannot find git repo root");
    let docs_dir = repo_root.join(DOCS_DIR);

    let lang_lines = read_lang_config(&repo_root);
    if lang_lines.is_empty() {
        println!("No language directories declared in {LANG_CONFIG}, nothing to check.");
        return;
    }

    let reference = docs_dir.join(&lang_lines[0]);
    if !reference.is_dir() {
        eprintln!(
            "Reference docs directory `{}` does not exist.",
            reference.display()
        );
        std::process::exit(1);
    }

    let mut failed = false;

    for lang in &lang_lines[1..] {
        let lang_dir = docs_dir.join(lang);
        println!("\nChecking `{lang}` against `{}` ...", lang_lines[0]);
        if !lang_dir.is_dir() {
            eprintln!("  ERROR: `{}` does not exist.", lang_dir.display());
            failed = true;
            continue;
        }
        if check_lang_dir(&reference, &lang_dir).is_err() {
            failed = true;
        }
    }

    if failed {
        println!();
        eprintln!(
            "{} Fix the differences above.",
            "Docs structure check FAILED.".red().bold()
        );
        std::process::exit(1);
    }

    println_cargo_style!("Done: docs structure is consistent across all languages!");
}

fn read_lang_config(repo_root: &Path) -> Vec<String> {
    let path = repo_root.join(LANG_CONFIG);
    let Ok(content) = fs::read_to_string(&path) else {
        return Vec::new();
    };
    content
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| l.trim_start_matches("./").to_string())
        .collect()
}

/// Returns `Err(())` when the translated directory does not mirror the reference.
fn check_lang_dir(reference: &Path, lang: &Path) -> Result<(), ()> {
    let mut failed = false;

    let ref_files = collect_md_files(reference);
    let lang_files = collect_md_files(lang);

    let ref_set: BTreeSet<PathBuf> = ref_files.clone().into_iter().collect();
    let lang_set: BTreeSet<PathBuf> = lang_files.clone().into_iter().collect();

    let missing: Vec<PathBuf> = ref_set.difference(&lang_set).cloned().collect();
    let extra: Vec<PathBuf> = lang_set.difference(&ref_set).cloned().collect();

    if !missing.is_empty() {
        failed = true;
        println!("  ERROR: files missing in translation:");
        for f in &missing {
            println!("    - {}", f.display());
        }
    }
    if !extra.is_empty() {
        failed = true;
        println!("  ERROR: extra files in translation:");
        for f in &extra {
            println!("    - {}", f.display());
        }
    }

    // Compare the structural signature of every file present in both sides.
    for file in &ref_files {
        if !lang_set.contains(file) {
            continue;
        }
        let ref_path = reference.join(file);
        let lang_path = lang.join(file);
        match compare_signature(&ref_path, &lang_path) {
            Ok(()) => {}
            Err(diff) => {
                failed = true;
                eprintln!(
                    "  {}: structure mismatch in `{}`",
                    "ERROR".red().bold(),
                    file.display().to_string().cyan()
                );
                for line in diff {
                    println!("    {line}");
                }
            }
        }
    }

    if failed { Err(()) } else { Ok(()) }
}

fn collect_md_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        let Ok(entries) = fs::read_dir(&current) else {
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

/// Compare the structural signatures of two markdown files.
///
/// Returns a list of human-readable diff lines on the first structural
/// difference found (all differences up to a small window are reported).
fn compare_signature(ref_path: &Path, lang_path: &Path) -> Result<(), Vec<String>> {
    let ref_content = fs::read_to_string(ref_path).unwrap_or_default();
    let lang_content = fs::read_to_string(lang_path).unwrap_or_default();

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
        diffs.push(format!(
            "    {}: {}",
            "line".yellow().bold(),
            (i + 1).to_string().yellow()
        ));
        diffs.push(format!(
            "    {} : {} {}",
            "expect".green().bold(),
            format!("`{}`", token_label(ref_tok.map_or("<eof>", String::as_str))).green(),
            display_line(ref_line).cyan()
        ));
        diffs.push(format!(
            "    {}  : {} {}",
            "found".red().bold(),
            format!(
                "`{}`",
                token_label(lang_tok.map_or("<eof>", String::as_str))
            )
            .red(),
            display_line(lang_line).cyan()
        ));
        if ref_sig.len() != lang_sig.len() && window >= 5 {
            diffs.push(format!(
                "    note: reference has {} lines, translation has {} lines",
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

/// Human-readable label for a structural token.
fn token_label(token: &str) -> String {
    match token {
        "B" => "blank".to_string(),
        "A" => "@@@".to_string(),
        "Q" => "quote".to_string(),
        "L" => "list".to_string(),
        "P" => "text".to_string(),
        t if t.starts_with("H") => format!("heading-{}", &t[1..]),
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

/// Render a source line for display: blank lines become `<blank>`.
fn display_line(line: &str) -> String {
    if line.trim().is_empty() {
        "<blank>".to_string()
    } else {
        truncate(line)
    }
}

/// Build the structural signature of a markdown file.
fn signature_of(content: &str) -> Vec<String> {
    let mut sig = Vec::new();
    let mut in_fence = false;
    let mut fence_lang = String::new();

    for raw_line in content.lines() {
        let line = raw_line.trim();

        if in_fence {
            if line.starts_with("```") {
                in_fence = false;
                sig.push(format!("F:{}", fence_lang));
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

    // An unclosed fence is still a fence line; the signature already recorded it.
    sig
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
    let digit_count = line.chars().take_while(|c| c.is_ascii_digit()).count();
    if digit_count == 0 {
        return false;
    }
    let rest = &line[digit_count..];
    (rest.starts_with(". ") || rest.starts_with(") "))
        && rest.chars().nth(1).is_some_and(|c| c == ' ' || c == '\t')
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

fn find_git_repo() -> Option<PathBuf> {
    let mut current = std::env::current_dir().ok()?;
    loop {
        if current.join(".git").is_dir() {
            return Some(current);
        }
        current = current.parent()?.to_path_buf();
    }
}
