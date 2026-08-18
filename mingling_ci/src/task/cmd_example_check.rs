use colored::Colorize;
use mingling::{
    Grouped, Routable,
    macros::{buffer, command, renderer},
    res::ResExitCode,
};

use crate::Next;
use crate::examples::{check_example, load_test_configs};
use crate::progress::task_progress_bar;
use crate::reporter::{self, ReportResult};

#[command(node = "example-check")]
pub async fn example_check() -> Next {
    reporter::set_task("Example-Check");

    let configs = load_test_configs();
    let total = configs.len();
    let pb = task_progress_bar(total, "Testing");
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

    ResultExampleCheck { fail_count }.to_chain()
}

/// Number of examples that failed to build or pass their tests.
#[derive(Grouped)]
pub struct ResultExampleCheck {
    pub fail_count: usize,
}

/// Silently sets a non-zero exit code when any example failed.
#[renderer(buffer)]
pub fn render_example_check(r: ResultExampleCheck, exit_code: &mut ResExitCode) {
    if r.fail_count > 0 {
        exit_code.exit_code = 1;
    }
}
