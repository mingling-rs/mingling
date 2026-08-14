//! Coverage test generator for mingling.
//!
//! This script requires the **fork** of cargo-llvm-cov:
//! <https://github.com/Weicao-CatilGrass/cargo-llvm-cov>
//!
//! The upstream `report` command cannot include binaries of non-workspace
//! crates (examples and test crates) and unconditionally filters
//! `tests`/`examples` source files. The fork adds two flags to fix this:
//!
//! - `--object <PATH>`: include arbitrary binaries in the report
//!   (upstream issue taiki-e/cargo-llvm-cov#367)
//! - `--include-tests-examples-benches`: stop filtering those source dirs
//!   (upstream issue taiki-e/cargo-llvm-cov#503)
//!
//! Install it with:
//!
//! ```bash
//! cargo install --git https://github.com/Weicao-CatilGrass/cargo-llvm-cov cargo-llvm-cov
//! ```

use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use tools::{eprintln_cargo_style, println_cargo_style, run_cmd};

const OUTPUT_DIR: &str = "docs/cov-test";

/// Shared target directory for all `cargo llvm-cov` runs.
///
/// Pointing every run at the same target dir makes all of them share the
/// instrumented build cache and, more importantly, accumulate profraw files
/// in one place so the final `report` can merge everything.
const COV_TARGET_DIR: &str = ".temp/cov-llvm";

/// An example's `test.toml` (`[[runs]]` entries).
#[derive(Deserialize)]
struct TestConfig {
    runs: Vec<TestCase>,
}

/// One `[[runs]]` entry of an example's `test.toml`.
#[derive(Deserialize)]
struct TestCase {
    input: Vec<String>,
}

fn main() {
    let repo_root = find_git_repo().expect("Failed to find git repository root");
    let output_path = repo_root.join(OUTPUT_DIR);
    let cov_target = repo_root.join(COV_TARGET_DIR);

    // Read features from [package.metadata.docs.rs]
    let features = tools::read_features().unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    });
    let features_arg = features.join(",");

    // Ensure output directory exists
    std::fs::create_dir_all(&output_path).expect("Failed to create output directory");
    std::fs::create_dir_all(&cov_target).expect("Failed to create cov target directory");

    // All `cargo llvm-cov` invocations below share one target dir, so profraw
    // files accumulate and are merged by the final `report` command.
    // SAFETY: set before any thread is spawned; this process only shells out
    // to subcommands via std::process.
    unsafe {
        std::env::set_var("CARGO_LLVM_COV_TARGET_DIR", &cov_target);
    }

    // Drop stale profraw from previous runs (keep the instrumented build cache).
    clean_old_profraw(&cov_target);

    println_cargo_style!("Features: {}", features_arg);
    println_cargo_style!("Target: {}", cov_target.display());

    // 1. Workspace tests
    println_cargo_style!("Running: cargo llvm-cov test --workspace");
    run_cmd!(format!(
        "cargo llvm-cov test --no-report --workspace --features \"{}\" --color always",
        features_arg
    ))
    .unwrap_or_else(|code| {
        eprintln_cargo_style!("workspace tests failed with exit code {}", code);
        std::process::exit(code);
    });

    // 2. Integration test crates under mingling_core/tests (excluded from the
    //    workspace, so they need their own `--manifest-path` runs)
    for manifest in find_test_crate_manifests(&repo_root) {
        println_cargo_style!(
            "Running: cargo llvm-cov test {}",
            manifest.file_name().unwrap_or_default().to_string_lossy()
        );
        run_cmd!(format!(
            "cargo llvm-cov test --no-report --manifest-path \"{}\" --color always",
            manifest.display()
        ))
        .unwrap_or_else(|code| {
            eprintln_cargo_style!(
                "test crate {} failed with exit code {}",
                manifest.display(),
                code
            );
            std::process::exit(code);
        });
    }

    // 3. Examples: build each example with explicit RUSTFLAGS, then execute
    //    every command declared in the example's test.toml directly.
    //
    //    NOTE: `cargo llvm-cov run` cannot be used here. Its rustc wrapper
    //    only instruments the crates of the *current* cargo project (with
    //    `--manifest-path` that is the example itself), so the mingling
    //    libraries — being dependencies — would not be instrumented and their
    //    coverage would silently be lost (once_exec.rs showed 0%). Building
    //    with plain RUSTFLAGS instruments the whole dependency graph.
    //
    //    RUSTFLAGS/CARGO_TARGET_DIR are set process-wide here because only the
    //    `report` step (which does not compile) follows. Non-zero exit codes
    //    are expected for some examples (e.g. `--help` exits with 2); profraw
    //    is still written.
    unsafe {
        std::env::set_var("RUSTFLAGS", "-Cinstrument-coverage");
        std::env::set_var("CARGO_TARGET_DIR", &cov_target);
    }
    let examples = load_example_commands(&repo_root);
    let mut built = std::collections::HashSet::new();
    for (example, input) in &examples {
        if built.insert(example.clone()) {
            println_cargo_style!("Building: {}", example);
            run_cmd!(format!(
                "cargo build --manifest-path examples/{}/Cargo.toml --color always",
                example
            ))
            .unwrap_or_else(|code| {
                eprintln_cargo_style!(
                    "build of example {} failed with exit code {}",
                    example,
                    code
                );
                std::process::exit(code);
            });
        }
        let binary = cov_target.join("debug").join(get_binary_name(example));
        let profraw = format!(
            "{}/example-{}.%p.profraw",
            cov_target.to_string_lossy(),
            example
        );
        match std::process::Command::new(&binary)
            .args(input)
            .env("LLVM_PROFILE_FILE", &profraw)
            .status()
        {
            Ok(status) if status.success() => {}
            Ok(status) => println_cargo_style!(
                "Warning: example {} exited with {:?}, profraw still recorded",
                example,
                status.code()
            ),
            Err(e) => eprintln_cargo_style!("Failed to run example {}: {}", example, e),
        }
    }

    // 4. Collect the binaries of non-workspace crates (examples + test crates).
    //    The automatic object-file detection only knows workspace members, so
    //    these must be passed explicitly via --object.
    let member_names = workspace_member_names(&repo_root);
    let object_args = collect_object_args(&cov_target, &member_names);

    // 5. Generate the merged HTML report.
    //
    //    --no-default-ignore-filename-regex: the default regex unconditionally
    //    excludes `examples`/`tests` directories, which is exactly what we want
    //    to include here, so we take over the filter ourselves.
    let ignore_re = build_ignore_regex(&cov_target);
    println_cargo_style!("Running: cargo llvm-cov report --html");
    run_cmd!(format!(
        "cargo llvm-cov report --html --output-dir \"{}\" --no-default-ignore-filename-regex --ignore-filename-regex \"{}\" {} --color always",
        output_path.to_string_lossy(),
        ignore_re,
        object_args
    ))
    .unwrap_or_else(|code| {
        eprintln_cargo_style!("cargo llvm-cov report failed with exit code {}", code);
        std::process::exit(code);
    });

    // Move files from <output_path>/html/ to <output_path>
    let html_dir = output_path.join("html");
    if html_dir.exists() && html_dir.is_dir() {
        println_cargo_style!("Moving files from {}/html/ to {}/", OUTPUT_DIR, OUTPUT_DIR);

        for entry in fs::read_dir(&html_dir).expect("Failed to read html directory") {
            let entry = entry.expect("Failed to read entry");
            let entry_path = entry.path();
            let file_name = entry
                .file_name()
                .to_str()
                .expect("Invalid filename")
                .to_owned();

            let dest_path = output_path.join(&file_name);
            if dest_path.exists() {
                if dest_path.is_dir() {
                    fs::remove_dir_all(&dest_path).unwrap_or_else(|e| {
                        eprintln!(
                            "Warning: could not remove directory {}: {}",
                            dest_path.display(),
                            e
                        );
                    });
                } else {
                    fs::remove_file(&dest_path).unwrap_or_else(|e| {
                        eprintln!(
                            "Warning: could not remove file {}: {}",
                            dest_path.display(),
                            e
                        );
                    });
                }
            }
            fs::rename(&entry_path, &dest_path).unwrap_or_else(|e| {
                eprintln!("Warning: could not move {}: {}", entry_path.display(), e);
            });
        }

        fs::remove_dir(&html_dir).unwrap_or_else(|e| {
            eprintln!("Warning: could not remove html directory: {}", e);
        });

        println_cargo_style!("Files moved successfully.");
    }

    // 6. Recolor the per-file coverage summary with project-specific
    //    thresholds: 0-50% red, 51-80% yellow, 81-100% green. llvm-cov's
    //    built-in thresholds differ, and the color is assigned when the HTML
    //    is generated, so the summary table is rewritten here.
    let index_path = output_path.join("index.html");
    if let Err(e) = recolor_report_index(&index_path) {
        eprintln_cargo_style!("Warning: failed to recolor {}: {}", index_path.display(), e);
    }

    println_cargo_style!(
        "Done: coverage report generated at {}/index.html",
        OUTPUT_DIR
    );
}

/// Remove `*.profraw` from the shared target dir so stale data from previous
/// runs does not pollute the merged report. The instrumented build cache
/// (everything else) is kept.
fn clean_old_profraw(cov_target: &Path) {
    if let Ok(entries) = fs::read_dir(cov_target) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "profraw") {
                let _ = fs::remove_file(&path);
            }
        }
    }
}

/// All `mingling_core/tests/<crate>/Cargo.toml` manifests.
fn find_test_crate_manifests(repo_root: &Path) -> Vec<PathBuf> {
    let tests_dir = repo_root.join("mingling_core/tests");
    let mut manifests = Vec::new();
    if let Ok(entries) = fs::read_dir(&tests_dir) {
        for entry in entries.flatten() {
            let manifest = entry.path().join("Cargo.toml");
            if manifest.is_file() {
                manifests.push(manifest);
            }
        }
    }
    manifests.sort();
    manifests
}

/// Parse every `examples/<name>/test.toml` into `(example_name, input)` pairs.
fn load_example_commands(repo_root: &Path) -> Vec<(String, Vec<String>)> {
    let examples_dir = repo_root.join("examples");
    let mut entries: Vec<_> = std::fs::read_dir(&examples_dir)
        .unwrap_or_else(|e| {
            eprintln_cargo_style!("Failed to read {}: {}", examples_dir.display(), e);
            std::process::exit(1);
        })
        .flatten()
        .collect();
    entries.sort_by_key(|e| e.file_name());

    let mut pairs = Vec::new();
    for entry in entries {
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
        let content = fs::read_to_string(&test_toml).unwrap_or_else(|e| {
            eprintln_cargo_style!("Failed to read {}: {}", test_toml.display(), e);
            std::process::exit(1);
        });
        let config: TestConfig = toml::from_str(&content).unwrap_or_else(|e| {
            eprintln_cargo_style!("Failed to parse {}: {}", test_toml.display(), e);
            std::process::exit(1);
        });
        for case in config.runs {
            pairs.push((name.clone(), case.input));
        }
    }
    pairs
}

/// Names of all workspace members, from `cargo metadata --no-deps`.
fn workspace_member_names(repo_root: &Path) -> Vec<String> {
    let Ok(output) = tools::run_cmd_capture_with_dir(
        "cargo metadata --no-deps --format-version 1".to_string(),
        repo_root,
    ) else {
        return Vec::new();
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&output) else {
        return Vec::new();
    };
    json["packages"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|p| p["name"].as_str().map(str::to_owned))
        .collect()
}

/// Collect the binaries of non-workspace crates (examples and test crates)
/// from the shared target dir, as `--object <path>` arguments.
///
/// - `debug/` root: example binaries (built via `cargo llvm-cov run`).
/// - `debug/deps/`: test crate binaries (e.g. `integration-<hash>`); their
///   names do not follow a single pattern, so anything that is not a
///   workspace-member binary and not a proc-macro `.so` is collected.
///
/// Workspace member binaries are detected automatically by `report` and must
/// NOT be passed again (duplicate `-object` entries produce duplicated
/// output). Hard links to the same file are deduplicated by inode.
fn collect_object_args(cov_target: &Path, member_names: &[String]) -> String {
    let debug_dir = cov_target.join("debug");
    let mut objects = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for dir in [debug_dir.clone(), debug_dir.join("deps")] {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() || !is_executable(&path) {
                continue;
            }
            if !seen.insert(file_id(&path)) {
                continue;
            }
            let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
                continue;
            };
            // Proc-macro shared objects are either workspace members (picked
            // up automatically) or external deps (excluded from the report
            // by the ignore regex), so never pass them explicitly.
            if name.starts_with("lib") && name.ends_with(".so") {
                continue;
            }
            if is_workspace_member_binary(name, member_names) {
                continue;
            }
            objects.push(path);
        }
    }

    objects.sort();
    objects
        .iter()
        .map(|p| format!("--object \"{}\"", p.to_string_lossy()))
        .collect::<Vec<_>>()
        .join(" ")
}

/// True if the binary name (e.g. `mingling_core-fea14a01b88afcaa`) belongs to
/// a workspace member.
fn is_workspace_member_binary(name: &str, member_names: &[String]) -> bool {
    let stem = strip_cargo_hash(name);
    member_names.iter().any(|m| stem == m)
}

/// Strip the cargo-generated hash suffix: `mingling_core-fea14a01b88afcaa` ->
/// `mingling_core`. Returns the input unchanged if there is no such suffix.
fn strip_cargo_hash(name: &str) -> &str {
    let Some(idx) = name.rfind('-') else {
        return name;
    };
    let (head, tail) = name.split_at(idx);
    let hash = &tail[1..];
    if hash.len() == 16 && hash.chars().all(|c| c.is_ascii_hexdigit()) {
        head
    } else {
        name
    }
}

/// A stable identity for deduplicating hard links: device+inode on Unix,
/// canonicalized path elsewhere.
fn file_id(path: &Path) -> String {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt as _;
        if let Ok(metadata) = fs::metadata(path) {
            return format!("{}:{}", metadata.dev(), metadata.ino());
        }
    }
    fs::canonicalize(path)
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .into_owned()
}

/// Resolve binary filename for the given example.
///
/// The binary name matches the package name. On Windows, the `.exe` suffix is
/// required.
fn get_binary_name(example_name: &str) -> String {
    let base = example_name;
    if cfg!(target_os = "windows") {
        format!("{base}.exe")
    } else {
        base.to_string()
    }
}

/// Rewrite the per-file coverage colors in `index.html` with project-specific
/// thresholds: 0-50% red, 51-80% yellow, 81-100% green.
fn recolor_report_index(index_path: &Path) -> std::io::Result<()> {
    let content = fs::read_to_string(index_path)?;
    fs::write(index_path, recolor_coverage_table(&content))
}

/// Recolor every `<td class='column-entry-...'><pre>XX% ...</pre></td>` cell
/// in the coverage summary table according to the new thresholds. Cells with
/// no data (e.g. branch coverage `- (0/0)`, class `gray`) are left as-is.
fn recolor_coverage_table(input: &str) -> String {
    const TD: &str = "<td class='column-entry-";
    let mut out = String::with_capacity(input.len());
    let mut rest = input;
    while let Some(pos) = rest.find(TD) {
        out.push_str(&rest[..pos + TD.len()]);
        rest = &rest[pos + TD.len()..];
        let Some(pre_end) = rest.find("'><pre>") else {
            out.push_str(rest);
            return out;
        };
        let color = &rest[..pre_end];
        let tail = &rest[pre_end + "'><pre>".len()..];
        let pct: String = tail
            .trim_start()
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        let new_color = match pct.parse::<f64>() {
            Ok(v) if v <= 50.0 => "red",
            Ok(v) if v <= 80.0 => "yellow",
            Ok(_) => "green",
            Err(_) => color, // no data (e.g. gray branch column)
        };
        out.push_str(new_color);
        out.push_str("'><pre>");
        rest = tail;
    }
    out.push_str(rest);
    out
}

/// True if the file is executable: mode bits on Unix, `.exe` on Windows.
fn is_executable(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let Ok(metadata) = std::fs::metadata(path) else {
            return false;
        };
        metadata.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        path.extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("exe"))
    }
}

/// Regex that keeps only the project's own sources in the report:
/// excludes the shared llvm-cov target dir, the standard library, and
/// external dependencies.
fn build_ignore_regex(cov_target: &Path) -> String {
    let target = regex_escape_path(cov_target);
    format!(
        "^{target}($|/)|/rustc/([0-9a-f]+|[0-9]+\\.[0-9]+\\.[0-9]+)/|/\\.cargo/(registry|git)/|/\\.rustup/toolchains($|/)"
    )
}

/// Escape a path for use inside a regular expression (as a literal prefix).
fn regex_escape_path(path: &Path) -> String {
    let s = path.to_string_lossy().replace('\\', "/");
    let mut escaped = String::with_capacity(s.len());
    for ch in s.chars() {
        if ch == '.' || ch == '-' {
            escaped.push('\\');
        }
        escaped.push(ch);
    }
    escaped
}

fn find_git_repo() -> Option<std::path::PathBuf> {
    let mut current_dir = std::env::current_dir().ok()?;

    loop {
        let git_dir = current_dir.join(".git");
        if git_dir.exists() && git_dir.is_dir() {
            return Some(current_dir);
        }

        if !current_dir.pop() {
            break;
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::recolor_coverage_table;

    #[test]
    fn recolor_thresholds() {
        let input = concat!(
            "<td class='column-entry-red'><pre>  50.00% (2/4)</pre></td>",
            "<td class='column-entry-yellow'><pre>  51.23% (32/52)</pre></td>",
            "<td class='column-entry-red'><pre>  80.00% (48/89)</pre></td>",
            "<td class='column-entry-green'><pre>  81.00% (1/1)</pre></td>",
            "<td class='column-entry-yellow'><pre>  90.00% (6/7)</pre></td>",
            "<td class='column-entry-gray'><pre>- (0/0)</pre></td>",
        );
        let out = recolor_coverage_table(input);
        assert!(out.contains("class='column-entry-red'><pre>  50.00%"));
        assert!(out.contains("class='column-entry-yellow'><pre>  51.23%"));
        assert!(out.contains("class='column-entry-yellow'><pre>  80.00%"));
        assert!(out.contains("class='column-entry-green'><pre>  81.00%"));
        assert!(out.contains("class='column-entry-green'><pre>  90.00%"));
        assert!(out.contains("class='column-entry-gray'><pre>- (0/0)"));
    }
}
