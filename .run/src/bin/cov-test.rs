use std::fs;
use std::path::PathBuf;
use tools::{println_cargo_style, run_cmd};

const OUTPUT_DIR: &str = "docs/cov-test";

fn main() {
    let repo_root = find_git_repo().expect("Failed to find git repository root");
    let output_path = repo_root.join(OUTPUT_DIR);

    // Read features from [package.metadata.docs.rs]
    let features = tools::read_features().unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    });

    let features_arg = features.join(",");

    // Ensure output directory exists
    std::fs::create_dir_all(&output_path).expect("Failed to create output directory");

    let cmd = format!(
        "cargo llvm-cov --html --output-dir \"{}\" --workspace --features \"{}\" --color always",
        output_path.to_string_lossy(),
        features_arg,
    );

    println_cargo_style!("Features: {}", features_arg);
    println_cargo_style!("Coverage: {}", output_path.display());

    println_cargo_style!("Running: cargo llvm-cov --html");
    run_cmd!(&cmd).unwrap_or_else(|code| {
        eprintln!("Error: cargo llvm-cov failed with exit code {}", code);
        std::process::exit(code);
    });

    // Move files from <output_path>/html/ to <output_path>
    let html_dir = output_path.join("html");
    if html_dir.exists() && html_dir.is_dir() {
        println_cargo_style!("Moving files from {}/html/ to {}/", OUTPUT_DIR, OUTPUT_DIR);

        // Move each entry in html_dir up one level
        for entry in fs::read_dir(&html_dir).expect("Failed to read html directory") {
            let entry = entry.expect("Failed to read entry");
            let entry_path = entry.path();
            let file_name = entry
                .file_name()
                .to_str()
                .expect("Invalid filename")
                .to_owned();

            let dest_path = output_path.join(&file_name);
            // Remove existing file/directory at destination if any
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

        // Remove the now-empty html directory
        fs::remove_dir(&html_dir).unwrap_or_else(|e| {
            eprintln!("Warning: could not remove html directory: {}", e);
        });

        println_cargo_style!("Files moved successfully.");
    }

    println_cargo_style!(
        "Done: coverage report generated at {}/index.html",
        OUTPUT_DIR
    );
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
