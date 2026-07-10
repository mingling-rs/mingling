use std::io::Write as _;
use std::path::Path;

use serde::Deserialize;
use tools::println_cargo_style;

#[derive(Deserialize)]
struct VersionFile {
    file: String,
    pattern: String,
}

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "file")]
    files: Vec<VersionFile>,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Get new version
    let new_ver = if args.len() > 1 {
        args[1].clone()
    } else {
        print!("Update version to: ");
        std::io::stdout().flush().unwrap();
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        input.trim().to_string()
    };

    if new_ver.is_empty() {
        eprintln!("Error: Version cannot be empty.");
        std::process::exit(1);
    }

    // Read current version from root Cargo.toml's workspace.package.version
    let root_cargo_path = "Cargo.toml";
    let root_cargo_content =
        std::fs::read_to_string(root_cargo_path).expect("Failed to read Cargo.toml");
    let cargo_value: toml::Value = root_cargo_content.parse().expect("Failed to parse Cargo.toml");

    let current_ver = cargo_value["workspace"]["package"]["version"]
        .as_str()
        .expect("workspace.package.version not found in Cargo.toml")
        .to_string();

    if new_ver == current_ver {
        println!("Version is already {}. Nothing to do.", current_ver);
        return;
    }

    println_cargo_style!("Version: {} -> {}", current_ver, new_ver);

    // Read version-files.toml
    let config_path = Path::new("dev_tools").join("version-files.toml");
    let config_str =
        std::fs::read_to_string(&config_path).expect("Failed to read dev_tools/version-files.toml");
    let config: Config = toml::from_str(&config_str)
        .expect("Failed to parse dev_tools/version-files.toml");

    let mut updated_count = 0;
    let mut skipped_count = 0;

    for vf in &config.files {
        let file_path = &vf.file;
        let old_pattern = vf.pattern.replace("{VER}", &current_ver);
        let new_pattern = vf.pattern.replace("{VER}", &new_ver);

        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Warning: Could not read {}: {}", file_path, e);
                skipped_count += 1;
                continue;
            }
        };

        let new_content = content.replace(&old_pattern, &new_pattern);

        if new_content == content {
            eprintln!(
                "Warning: Pattern '{}' not found in {}",
                old_pattern, file_path
            );
            skipped_count += 1;
            continue;
        }

        std::fs::write(file_path, &new_content)
            .unwrap_or_else(|e| panic!("Failed to write {}: {}", file_path, e));
        println_cargo_style!("Updated: {}", file_path);
        updated_count += 1;
    }

    println_cargo_style!("Done: {} file(s) updated, {} file(s) skipped", updated_count, skipped_count);
}
