//! IO side of the report command: reads the collect directory once and keeps
//! the parsed data in a resource, so chains only do computation.

use std::collections::BTreeMap;

use mingling::{Program, macros::program_setup};

use crate::ThisProgram;
use crate::reporter::COLLECT_DIR;

/// Git commit date and short hash for the report.
#[derive(Default, Clone, Debug)]
pub struct GitInfo {
    pub date: String,
    pub commit_hash: String,
}

/// Parsed contents of the collect directory.
#[derive(Default, Clone)]
pub struct ResCollectLogs {
    /// `(task, package) -> os -> ok`
    pub statuses: BTreeMap<(String, String), BTreeMap<String, bool>>,
    /// `(task, os, package) -> stripped error output`
    pub err_outputs: BTreeMap<(String, String, String), String>,
    pub git: GitInfo,
}

impl ResCollectLogs {
    /// Reads `collect/{task}/{os}/{package}.{ok|err}` and the git info.
    #[must_use]
    pub fn read() -> Self {
        let mut logs = Self::default();

        if let Ok(task_entries) = std::fs::read_dir(COLLECT_DIR) {
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
                            logs.statuses
                                .entry((task.clone(), package.clone()))
                                .or_default()
                                .insert(os.clone(), ok);
                            if !ok {
                                let err = std::fs::read_to_string(file.path()).unwrap_or_default();
                                logs.err_outputs
                                    .insert((task.clone(), os.clone(), package), strip_ansi(&err));
                            }
                        }
                    }
                }
            }
        }

        logs.git = git_info();
        logs
    }
}

#[program_setup]
pub fn report_setup(p: &mut Program<ThisProgram>) {
    p.with_resource(ResCollectLogs::read());
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
