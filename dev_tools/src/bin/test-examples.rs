use std::collections::HashMap;

use colored::Colorize;
use indicatif::ProgressBar;
use serde::Deserialize;
use tools::{eprintln_cargo_style, println_cargo_style};

#[derive(Deserialize)]
struct TestConfig {
    test: HashMap<String, Vec<TestCase>>,
}

#[derive(Deserialize)]
struct TestCase {
    command: String,
    expect: Expect,
}

#[derive(Deserialize)]
struct Expect {
    #[serde(rename = "exit-code")]
    exit_code: i32,
    result: String,
}

fn main() {
    #[cfg(windows)]
    let _ = colored::control::set_virtual_terminal(true);

    let config = load_config();

    // Count total test cases upfront
    let total: usize = config.test.values().map(|cases| cases.len()).sum();
    let bar = ProgressBar::new(total as u64);
    bar.set_style(
        indicatif::ProgressStyle::default_bar()
            .template(&format!(
                "{} [{{bar:28}}] {{pos}}/{{len}}: {{msg}}",
                "     Testing".bold().bright_cyan()
            ))
            .unwrap()
            .progress_chars("=> "),
    );
    bar.set_message("examples");

    let passed = run_all_tests(&config, &bar);

    bar.finish_and_clear();

    println_cargo_style!("Result: {}/{} tests passed", passed, total);

    if passed != total {
        eprintln_cargo_style!("{} test(s) failed", total - passed);
        std::process::exit(1);
    }
}

/// Parse test config from TOML file
fn load_config() -> TestConfig {
    let content = std::fs::read_to_string("examples/test-examples.toml").unwrap_or_else(|e| {
        eprintln_cargo_style!("Failed to read TOML config file: {}", e);
        std::process::exit(1);
    });

    toml::from_str(&content).unwrap_or_else(|e| {
        eprintln_cargo_style!("Failed to parse TOML config: {}", e);
        std::process::exit(1);
    })
}

/// Run all example test groups, return number passed
fn run_all_tests(config: &TestConfig, bar: &ProgressBar) -> usize {
    let mut passed = 0;

    for (example_name, test_cases) in &config.test {
        bar.set_message(example_name.clone());

        if !build_example(example_name) {
            bar.inc(test_cases.len() as u64);
            continue;
        }

        for test_case in test_cases {
            if run_single_test(example_name, test_case, bar) {
                passed += 1;
            }
            bar.inc(1);
        }
    }

    passed
}

/// Build the example binary, return true on success
fn build_example(example_name: &str) -> bool {
    let manifest = format!("examples/{example_name}/Cargo.toml");
    tools::run_cmd_capture(format!(
        "cargo build --manifest-path {manifest} --color always",
    ))
    .is_ok()
}

/// Run a single test case, return true on pass
fn run_single_test(example_name: &str, test_case: &TestCase, bar: &ProgressBar) -> bool {
    let binary_path = format!(".temp/target/debug/{}", get_binary_name(example_name));
    let args: Vec<&str> = test_case.command.split_whitespace().collect();

    let output = match std::process::Command::new(&binary_path)
        .args(&args)
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            bar.println(format!("'{}' - failed to run: {}", test_case.command, e));
            return false;
        }
    };

    let actual_exit_code = output.status.code().unwrap_or(-1);
    let actual_stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let actual_stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    let exit_ok = actual_exit_code == test_case.expect.exit_code;
    let result_ok = actual_stdout == test_case.expect.result
        || actual_stdout.contains(&test_case.expect.result);

    if exit_ok && result_ok {
        true
    } else {
        bar.println(format!("failed: '{}'", test_case.command));
        if !exit_ok {
            bar.println(format!(
                "  Expected exit code: {}, actual: {}",
                test_case.expect.exit_code, actual_exit_code
            ));
        }
        if !result_ok {
            bar.println(format!("  Expected output: {:?}", test_case.expect.result));
            bar.println(format!("  Actual stdout: {:?}", actual_stdout));
            if !actual_stderr.is_empty() {
                bar.println(format!("  Actual stderr: {:?}", actual_stderr));
            }
        }
        false
    }
}

/// Resolve binary filename for the given example
///
/// The binary name matches the package name. On Windows, the `.exe` suffix is required.
fn get_binary_name(example_name: &str) -> String {
    let base = example_name;
    if cfg!(target_os = "windows") {
        format!("{base}.exe")
    } else {
        base.to_string()
    }
}
