//! Shared terminal progress helpers, in a cargo-like style.
//!
//! `download_bar` is a determinate byte/percent bar (falling back to a spinner
//! when the total is unknown) driven by the caller as bytes arrive. Git pulls
//! are not given a synthetic bar — `git` already renders its own progress to
//! stderr, so those paths simply inherit git's raw output.

use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};

/// Right-align `prefix` to 12 columns and style it like `dev/ci` does.
fn styled_prefix(prefix: &str) -> String {
    let padding = " ".repeat(12usize.saturating_sub(prefix.len()));
    format!("{padding}{}", prefix.bold().bright_cyan())
}

/// A determinate download bar, or a spinner when `total` is unknown.
///
/// The caller drives it with `set_position` (or `set_message`/`tick` for the
/// spinner) and must call `finish_and_clear()` when done. Drawn to stderr so
/// stdout output stays clean.
pub fn download_bar(total: Option<u64>, prefix: &str) -> ProgressBar {
    let styled = styled_prefix(prefix);
    match total {
        Some(len) => {
            let pb = ProgressBar::new(len);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template(&format!(
                        "{styled} [{{bar:28}}] {{bytes}}/{{total_bytes}} ({{percent}}%)"
                    ))
                    .unwrap()
                    .progress_chars("=> "),
            );
            pb
        }
        None => {
            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template(&format!("{styled} [{{spinner:.cyan}}] {{msg}}"))
                    .unwrap(),
            );
            pb
        }
    }
}
