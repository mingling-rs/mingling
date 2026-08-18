//! Shared task progress bar.

use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};

/// Creates a task progress bar with the CI's standard style.
///
/// `prefix` is the phase label shown before the bar, right-aligned to 12
/// columns (e.g. `Building`, `Clippy`, `Testing`). The caller sets the
/// initial message and drives the position.
pub(crate) fn task_progress_bar(len: usize, prefix: &str) -> ProgressBar {
    let padding = " ".repeat(12usize.saturating_sub(prefix.len()));
    let styled_prefix = format!("{padding}{}", prefix.bold().bright_cyan());
    let pb = ProgressBar::new(len as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(&format!(
                "{styled_prefix} [{{bar:28}}] {{pos}}/{{len}}: {{msg}}"
            ))
            .unwrap()
            .progress_chars("=> "),
    );
    pb
}
