use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use mingling::{
    Grouped, Routable,
    macros::{buffer, command, renderer},
    res::ResExitCode,
};

use crate::Next;
use crate::examples::{check_example, load_test_configs};
use crate::reporter::{self, ReportResult};

#[command(node = "test-examples")]
pub async fn test_examples() -> Next {
    reporter::set_task("Test-Examples");

    let configs = load_test_configs();
    let total = configs.len();
    let pb = ProgressBar::new(total as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(&format!(
                "{} [{{bar:28}}] {{pos}}/{{len}}: {{msg}}",
                "     Testing".bold().bright_cyan()
            ))
            .unwrap()
            .progress_chars("=> "),
    );
    pb.set_message("examples");

    // One blocking task per example: build + run its test cases.
    let mut handles = Vec::new();
    for example in configs {
        handles.push(tokio::task::spawn_blocking(move || check_example(example)));
    }

    let mut fail_count = 0;
    for handle in handles {
        let Ok(outcome) = handle.await else {
            continue;
        };
        pb.set_message(outcome.name.clone());
        pb.inc(1);

        if outcome.ok {
            reporter::export(&outcome.name, &outcome.location, ReportResult::Ok);
        } else {
            fail_count += 1;
            // Plain stderr: `pb.println` is swallowed on non-TTY (CI).
            eprintln!("  {} {}", "failed".bright_red(), outcome.name);
            eprintln!("  {}", outcome.output);
            reporter::export(
                &outcome.name,
                &outcome.location,
                ReportResult::Error(outcome.output),
            );
        }
    }

    pb.finish_and_clear();
    reporter::flush();

    ResultTestExamples { fail_count }.to_chain()
}

/// Number of examples that failed to build or pass their tests.
#[derive(Grouped)]
pub struct ResultTestExamples {
    pub fail_count: usize,
}

/// Silently sets a non-zero exit code when any example failed.
#[renderer(buffer)]
pub fn render_test_examples(r: ResultTestExamples, exit_code: &mut ResExitCode) {
    if r.fail_count > 0 {
        exit_code.exit_code = 1;
    }
}
