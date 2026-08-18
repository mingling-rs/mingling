pub mod dependency_order;
pub mod verify;

use colored::Colorize;

use std::io::IsTerminal as _;

#[macro_export]
macro_rules! run_cmd {
    ($fmt:literal, $($arg:tt)*) => {
        $crate::run_cmd(format!($fmt, $($arg)*))
    };
    ($cmd:expr) => {
        $crate::run_cmd($cmd)
    };
}

/// Run a shell command and capture its combined stdout+stderr output.
/// Returns `Ok(output)` on success, `Err((exit_code, stderr))` on failure.
#[macro_export]
macro_rules! run_cmd_and_capture_stderr {
    ($fmt:literal, $($arg:tt)*) => {
        $crate::run_cmd_capture(format!($fmt, $($arg)*))
    };
    ($cmd:expr) => {
        $crate::run_cmd_capture($cmd)
    };
}

#[macro_export]
macro_rules! println_cargo_style {
    ($fmt:literal, $($arg:tt)*) => {
        $crate::println_cargo_style(format!($fmt, $($arg)*))
    };
    ($cmd:expr) => {
        $crate::println_cargo_style($cmd)
    };
}

#[macro_export]
macro_rules! eprintln_cargo_style {
    ($fmt:literal, $($arg:tt)*) => {
        $crate::eprintln_cargo_style(format!($fmt, $($arg)*))
    };
    ($cmd:expr) => {
        $crate::eprintln_cargo_style($cmd)
    };
}

#[macro_export]
macro_rules! wprintln_cargo_style {
    ($fmt:literal, $($arg:tt)*) => {
        $crate::wprintln_cargo_style(format!($fmt, $($arg)*))
    };
    ($cmd:expr) => {
        $crate::wprintln_cargo_style($cmd)
    };
}

/// Print a message in cargo style format, with bold green prefix.
///
/// # Panics
///
/// Panics if the prefix (text before the first `:`) exceeds 12 characters.
pub fn println_cargo_style(str: impl Into<String>) {
    let s = str.into();
    let (prefix, content) = if let Some(pos) = s.find(':') {
        (
            s[..pos].trim().to_string(),
            s[pos + 1..].trim_start().to_string(),
        )
    } else {
        (String::new(), s.trim().to_string())
    };

    assert!(
        prefix.len() <= 12,
        "prefix length exceeds 12: '{}' has length {}",
        prefix,
        prefix.len()
    );

    let padding = " ".repeat(12 - prefix.len());

    println!(
        "{}{} {}",
        padding,
        prefix.bold().bright_green(),
        content.trim()
    );
}

pub fn eprintln_cargo_style(str: impl Into<String>) {
    println!("{}: {}", "error".bold().bright_red(), str.into());
}

/// Print a message in cargo style format, with bold yellow prefix (warning style).
///
/// # Panics
///
/// Panics if the prefix (text before the first `:`) exceeds 12 characters.
pub fn wprintln_cargo_style(str: impl Into<String>) {
    let s = str.into();
    let (prefix, content) = if let Some(pos) = s.find(':') {
        (
            s[..pos].trim().to_string(),
            s[pos + 1..].trim_start().to_string(),
        )
    } else {
        (String::new(), s.trim().to_string())
    };

    assert!(
        prefix.len() <= 12,
        "prefix length exceeds 12: '{}' has length {}",
        prefix,
        prefix.len()
    );

    let padding = " ".repeat(12 - prefix.len());

    println!(
        "{}{} {}",
        padding,
        prefix.bold().bright_yellow(),
        content.trim()
    );
}

/// Run a shell command in the current directory and return its exit status.
///
/// # Panics
///
/// Panics if the shell command cannot be spawned (e.g. the shell binary is not found).
///
/// # Errors
///
/// Returns `Err` with the exit code if the command finishes with a non-zero exit code.
pub fn run_cmd(cmd: impl Into<String>) -> Result<(), i32> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    run_cmd_with_dir(cmd.into(), &cwd)
}

/// Run a shell command in the specified directory and return its exit status.
///
/// # Panics
///
/// Panics if the shell command cannot be spawned (e.g. the shell binary is not found).
///
/// # Errors
///
/// Returns `Err` with the exit code if the command finishes with a non-zero exit code.
pub fn run_cmd_with_dir(cmd: impl Into<String>, dir: &std::path::Path) -> Result<(), i32> {
    let shell = if cfg!(target_os = "windows") {
        "powershell"
    } else {
        "sh"
    };
    let status = std::process::Command::new(shell)
        .arg("-c")
        .arg(cmd.into())
        .current_dir(dir)
        .status()
        .expect("failed to execute command");

    let exit_code = status.code().unwrap_or(1);
    if exit_code == 0 {
        Ok(())
    } else {
        Err(exit_code)
    }
}

/// Run a shell command and capture its combined stdout+stderr output.
///
/// On success returns `Ok(combined_output)`. On failure returns `Err((exit_code, stderr))`.
/// Stderr falls back to stdout if stderr is empty.
pub fn run_cmd_capture(cmd: impl Into<String>) -> Result<String, (i32, String)> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    run_cmd_capture_with_dir(cmd.into(), &cwd)
}

/// Run a shell command in the specified directory and capture its combined stdout+stderr output.
///
/// On success returns `Ok(combined_output)`. On failure returns `Err((exit_code, stderr))`.
/// Stderr falls back to stdout if stderr is empty.
pub fn run_cmd_capture_with_dir(
    cmd: impl Into<String>,
    dir: &std::path::Path,
) -> Result<String, (i32, String)> {
    let shell = if cfg!(target_os = "windows") {
        "powershell"
    } else {
        "sh"
    };
    let output = std::process::Command::new(shell)
        .arg("-c")
        .arg(cmd.into())
        .current_dir(dir)
        .output()
        .expect("failed to execute command");

    let exit_code = output.status.code().unwrap_or(1);
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    // Keep both streams so a failure is never hidden: when stderr carries
    // warnings, the real failure details (e.g. the failing test name and
    // assertion diff) usually live on stdout and must not be dropped.
    let combined = match (stdout.trim().is_empty(), stderr.trim().is_empty()) {
        (false, false) => format!("{stdout}\n{stderr}"),
        (false, true) => stdout,
        (true, false) => stderr,
        (true, true) => stdout,
    };

    if exit_code == 0 {
        Ok(combined)
    } else {
        Err((exit_code, combined))
    }
}

/// Extract a crate-style name from a `Cargo.toml` path.
///
/// Examples:
/// - `mingling_core/Cargo.toml` → `mingling_core`
/// - `.` → `(root)`
pub fn crate_name_from(path: &std::path::Path) -> String {
    path.parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("(root)")
        .to_string()
}

/// Run a list of `(label_for_errors, crate_name_for_bar, shell_command)` tuples
/// in parallel with a progress bar.
///
/// - Success: silent, the bar tracks progress:
///   `  Building [============================] 32/32: mingling_core`
/// - Failure: `pb.println()` prints the error immediately above the bar.
pub fn run_parallel(phase: &str, tasks: Vec<(String, String, String)>) -> Result<(), i32> {
    let n = tasks.len();
    if n == 0 {
        return Ok(());
    }

    // Cargo-style prefix: right-aligned to 12 chars, bold bright cyan
    let padding = " ".repeat(12 - phase.len());
    let styled_prefix = format!("{}{}", padding, phase.bold().bright_cyan());

    let pb = indicatif::ProgressBar::new(n as u64);
    pb.set_style(
        indicatif::ProgressStyle::default_bar()
            .template(&format!(
                "{} [{{bar:28}}] {{pos}}/{{len}}: {{msg}}",
                styled_prefix
            ))
            .unwrap()
            .progress_chars("=> "),
    );
    pb.set_position(0);

    // Pre-extract labels for error messages
    let labels: Vec<String> = tasks.iter().map(|(l, _, _)| l.clone()).collect();

    let (tx, rx) = std::sync::mpsc::channel::<(usize, String, Result<String, (i32, String)>)>();

    for (i, (_label, crate_name, cmd)) in tasks.into_iter().enumerate() {
        let tx = tx.clone();
        std::thread::spawn(move || {
            let result = run_cmd_capture(&cmd);
            let _ = tx.send((i, crate_name, result));
        });
    }
    drop(tx);

    let mut first_exit_code = 0;

    while let Ok((i, crate_name, result)) = rx.recv() {
        pb.inc(1);
        pb.set_message(crate_name);

        if let Err((code, output)) = result {
            if first_exit_code == 0 {
                first_exit_code = code;
            }
            let msg = format!(
                "{}: {} failed (exit code {})",
                "error".bright_red().bold(),
                labels[i],
                code,
            );
            let mut lines = Vec::new();
            if !output.is_empty() {
                lines.extend(output.lines().map(|l| format!("  {l}")));
            }
            if std::io::stdout().is_terminal() {
                // On a TTY, render errors through the progress bar so they
                // appear above it.
                pb.println(&msg);
                for line in &lines {
                    pb.println(line);
                }
            } else {
                // On a non-TTY (CI, piped output), `ProgressBar::println` can
                // be swallowed, hiding the failure. Emit to plain stdout so the
                // failure is always visible.
                println!("{msg}");
                for line in &lines {
                    println!("{line}");
                }
            }
        }
    }

    pb.finish_and_clear();

    if first_exit_code != 0 {
        Err(first_exit_code)
    } else {
        Ok(())
    }
}

/// Run a single shell command with a progress bar, capturing its output.
///
/// - Success: bar clears silently.
/// - Failure: error is printed above the bar, then the bar clears.
pub fn run_cmd_with_progress(phase: &str, label: &str, cmd: String) -> Result<(), i32> {
    let padding = " ".repeat(12 - phase.len());
    let styled_prefix = format!("{}{}", padding, phase.bold().bright_cyan());

    let pb = indicatif::ProgressBar::new(1);
    pb.set_style(
        indicatif::ProgressStyle::default_bar()
            .template(&format!(
                "{} [{{bar:28}}] {{pos}}/{{len}}: {{msg}}",
                styled_prefix
            ))
            .unwrap()
            .progress_chars("=> "),
    );
    pb.set_message(label.to_owned());

    let result = run_cmd_capture(&cmd);
    pb.inc(1);
    pb.finish_and_clear();

    match result {
        Ok(_) => Ok(()),
        Err((code, output)) => {
            eprintln_cargo_style(format!("{} failed (exit code {})", label, code));
            if !output.is_empty() {
                println!("{}", output.trim_end());
            }
            Err(code)
        }
    }
}

/// Read `[package.metadata.docs.rs].features` from `mingling/Cargo.toml`.
///
/// Finds the git repository root, reads `mingling/Cargo.toml`, parses it as TOML,
/// and extracts the feature list under `[package.metadata.docs.rs].features`.
///
/// # Errors
///
/// Returns `std::io::Error` if:
/// - The git repository root cannot be found.
/// - The manifest file cannot be read.
/// - The TOML cannot be parsed.
/// - The `[package.metadata.docs.rs].features` key is missing or empty.
pub fn read_features() -> Result<Vec<String>, std::io::Error> {
    // Find git repo root
    let mut current_dir = std::env::current_dir()?;
    let repo_root = loop {
        let git_dir = current_dir.join(".git");
        if git_dir.exists() && git_dir.is_dir() {
            break Some(current_dir);
        }
        if !current_dir.pop() {
            break None;
        }
    };
    let repo_root = repo_root.ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Failed to find git repository root",
        )
    })?;

    let manifest_path = repo_root.join("mingling/Cargo.toml");
    if !manifest_path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Manifest not found at {}", manifest_path.display()),
        ));
    }

    let manifest_content = std::fs::read_to_string(&manifest_path)?;
    let cargo_toml: toml::Value = manifest_content.parse().map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Failed to parse Cargo.toml: {}", e),
        )
    })?;

    let doc_features = cargo_toml
        .get("package")
        .and_then(|p| p.get("metadata"))
        .and_then(|m| m.get("docs"))
        .and_then(|d| d.get("rs"))
        .and_then(|rs| rs.get("features"))
        .and_then(|f| f.as_array())
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "[package.metadata.docs.rs] or its 'features' key not found in mingling/Cargo.toml",
            )
        })?;

    let features: Vec<String> = doc_features
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();

    if features.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "No features defined in [package.metadata.docs.rs]",
        ));
    }

    Ok(features)
}

#[must_use]
pub fn cargo_tomls() -> Vec<std::path::PathBuf> {
    let mut cargo_tomls = Vec::new();
    let mut dirs = vec![std::path::PathBuf::from(".")];
    while let Some(dir) = dirs.pop() {
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    // Skip the .run directory
                    if path.file_name().and_then(|n| n.to_str()) == Some(".run") {
                        continue;
                    }
                    dirs.push(path);
                } else if path.file_name().and_then(|n| n.to_str()) == Some("Cargo.toml") {
                    cargo_tomls.push(path);
                }
            }
        }
    }
    cargo_tomls
}
