//! Example binary testing: build each example and run its `test.toml` cases.

use std::process::Output;

/// A single `[[runs]]` entry of an example's `test.toml`.
pub(crate) struct TestCase {
    input: Vec<String>,
    expect: Expect,
}

struct Expect {
    exit_code: i32,
    result: String,
}

/// One example and its test cases.
pub(crate) struct ExampleCase {
    name: String,
    cases: Vec<TestCase>,
}

/// Outcome of checking one example.
pub(crate) struct ExampleOutcome {
    pub name: String,
    pub location: String,
    pub ok: bool,
    pub output: String,
}

/// Loads `examples/<name>/test.toml` for every example that has one, in
/// alphabetical order of the example directory name.
pub(crate) fn load_test_configs() -> Vec<ExampleCase> {
    let mut configs = Vec::new();
    if let Ok(entries) = std::fs::read_dir("examples") {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let test_toml = path.join("test.toml");
            if !test_toml.is_file() {
                continue;
            }
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default()
                .to_string();
            let Ok(content) = std::fs::read_to_string(&test_toml) else {
                continue;
            };
            let Ok(table) = content.parse::<toml::Value>() else {
                continue;
            };
            let Some(cases) = parse_cases(&table) else {
                continue;
            };
            configs.push(ExampleCase { name, cases });
        }
    }
    configs.sort_by(|a, b| a.name.cmp(&b.name));
    configs
}

fn parse_cases(table: &toml::Value) -> Option<Vec<TestCase>> {
    let runs = table.get("runs")?.as_array()?;
    let mut cases = Vec::new();
    for run in runs {
        let input: Vec<String> = run
            .get("input")?
            .as_array()?
            .iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect();
        let expect = run.get("expect")?;
        let exit_code = expect
            .get("exit-code")?
            .as_integer()
            .and_then(|e| i32::try_from(e).ok())
            .unwrap_or(-1);
        let result = expect
            .get("result")
            .and_then(|r| r.as_str())
            .unwrap_or_default()
            .to_string();
        cases.push(TestCase {
            input,
            expect: Expect { exit_code, result },
        });
    }
    Some(cases)
}

/// Builds the example, then runs all of its test cases.
pub(crate) fn check_example(example: ExampleCase) -> ExampleOutcome {
    let location = format!("./examples/{}", example.name);

    // Phase 1: build.
    let manifest = format!("examples/{}/Cargo.toml", example.name);
    let build = std::process::Command::new("cargo")
        .args(["build", "--manifest-path", &manifest])
        .output();
    match build {
        Ok(output) if !output.status.success() => ExampleOutcome {
            name: example.name,
            location,
            ok: false,
            output: build_error(&output),
        },
        Err(e) => ExampleOutcome {
            name: example.name,
            location,
            ok: false,
            output: format!("failed to run cargo: {e}"),
        },
        Ok(_) => {
            // Phase 2: run the test cases against the built binary.
            let mut failures = Vec::new();
            for case in &example.cases {
                if let Err(detail) = run_case(&example.name, case) {
                    failures.push(detail);
                }
            }
            ExampleOutcome {
                name: example.name,
                location,
                ok: failures.is_empty(),
                output: failures.join("\n\n"),
            }
        }
    }
}

/// Runs a single test case against the built binary.
fn run_case(name: &str, case: &TestCase) -> Result<(), String> {
    let exe = if cfg!(target_os = "windows") {
        ".exe"
    } else {
        ""
    };
    let binary = format!(".temp/target/debug/{name}{exe}");

    let output = std::process::Command::new(&binary)
        .args(&case.input)
        .output();
    let Ok(output) = output else {
        return Err(format!("failed to run {binary}"));
    };

    let actual_exit_code = output.status.code().unwrap_or(-1);
    let actual_stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let actual_stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    let exit_ok = actual_exit_code == case.expect.exit_code;
    let result_ok =
        actual_stdout == case.expect.result || actual_stdout.contains(&case.expect.result);

    if exit_ok && result_ok {
        return Ok(());
    }

    let mut details = vec![format!("input: {}", case.input.join(" "))];
    if !exit_ok {
        details.push(format!(
            "expected exit code {}, actual {actual_exit_code}",
            case.expect.exit_code
        ));
    }
    if !result_ok {
        details.push(format!("expected output {:?}", case.expect.result));
        details.push(format!("actual stdout {actual_stdout:?}"));
        if !actual_stderr.is_empty() {
            details.push(format!("actual stderr {actual_stderr:?}"));
        }
    }
    Err(details.join("\n"))
}

/// Tail of a failed build's combined output.
fn build_error(output: &Output) -> String {
    let mut log = String::from_utf8_lossy(&output.stdout).into_owned();
    log.push_str(&String::from_utf8_lossy(&output.stderr));
    let lines: Vec<&str> = log.lines().collect();
    let tail = &lines[lines.len().saturating_sub(20)..];
    format!("build failed\n{}", tail.join("\n"))
}
