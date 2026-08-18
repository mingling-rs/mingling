use std::path::Path;

use arg_picker::{Picker, macros::arg};
use tools::{println_cargo_style, run_cmd};

const OUTPUT_DIR: &str = "docs/api-docs";

fn main() {
    let using_docsrs = Picker::from_args()
        .pick_or_default(&arg![docsrs: bool])
        .unwrap();

    let repo_root = find_git_repo().expect("Failed to find git repository root");

    // Read features from [package.metadata.docs.rs]
    let features = tools::read_features().unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    });

    let features_arg = features.join(",");

    // Ensure output directory exists
    let output_path = repo_root.join(OUTPUT_DIR);
    std::fs::create_dir_all(&output_path).expect("Failed to create output directory");

    // Build cargo doc command
    let cmd = if using_docsrs {
        format!(
            "cargo +nightly rustdoc --features \"{}\" -p mingling --target-dir \"{}\" --color always -- --cfg docsrs",
            features_arg,
            output_path.join("target").to_string_lossy()
        )
    } else {
        format!(
            "cargo doc --no-deps --features \"{}\" -p mingling --target-dir \"{}\" --color always",
            features_arg,
            output_path.join("target").to_string_lossy()
        )
    };

    println_cargo_style!("Features: {}", features_arg);
    println_cargo_style!("Output: {}", output_path.display());

    // Run cargo doc, then copy generated docs to output directory
    println_cargo_style!("Building: docs (cargo doc --no-deps)");
    run_cmd!(&cmd).unwrap_or_else(|code| {
        eprintln!("Error: cargo doc failed with exit code {}", code);
        std::process::exit(code);
    });

    // Copy generated docs from target/doc to OUTPUT_DIR (top level)
    let doc_source = output_path.join("target").join("doc");
    let doc_dest = &output_path;

    if doc_source.exists() {
        println_cargo_style!("Copying: docs to output directory");
        // Remove old docs in destination (except target/)
        if let Ok(entries) = std::fs::read_dir(doc_dest) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.file_name().and_then(|n| n.to_str()) == Some("target") {
                    continue;
                }
                if path.is_dir() {
                    std::fs::remove_dir_all(&path).ok();
                } else {
                    std::fs::remove_file(&path).ok();
                }
            }
        }
        copy_dir_recursively(&doc_source, doc_dest).expect("Failed to copy documentation");
    }

    // Clean up the intermediate target directory to save space
    std::fs::remove_dir_all(output_path.join("target")).ok();

    println_cargo_style!("Done: API docs deployed to {}", output_path.display());
}

fn copy_dir_recursively(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !dst.exists() {
        std::fs::create_dir_all(dst)?;
    }

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let file_name = src_path.file_name().expect("Failed to get file name");
        let dst_path = dst.join(file_name);

        if file_type.is_dir() {
            copy_dir_recursively(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
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
