use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use tools::println_cargo_style;

#[derive(Serialize)]
struct ExampleMeta {
    id: String,
    name: String,
    icon: String,
    category: String,
    desc: String,
    tags: Vec<String>,
    files: Vec<String>,
}

#[derive(Deserialize)]
struct PageToml {
    example: PageTomlExample,
}

#[derive(Deserialize)]
struct PageTomlExample {
    id: String,
    #[serde(default)]
    name: String,
    #[serde(default = "default_icon")]
    icon: String,
    #[serde(default)]
    category: String,
    #[serde(default)]
    desc: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default = "default_files")]
    files: Vec<String>,
}

fn default_icon() -> String {
    "📦".to_string()
}

fn default_files() -> Vec<String> {
    vec!["Cargo.toml".to_string(), "src/main.rs".to_string()]
}

fn main() {
    #[cfg(windows)]
    let _ = colored::control::set_virtual_terminal(true);

    let examples_dir = Path::new("examples");
    let output_dir = Path::new("docs/example-pages");
    fs::create_dir_all(output_dir).expect("failed to create docs/example-pages");

    let mut examples: Vec<ExampleMeta> = Vec::new();

    let entries = fs::read_dir(examples_dir).expect("failed to read examples/");
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        let id = dir_name.to_string();
        let page_toml_path = path.join("page.toml");

        let meta = if page_toml_path.exists() {
            match fs::read_to_string(&page_toml_path)
                .map_err(|e| e.to_string())
                .and_then(|content| toml::from_str::<PageToml>(&content).map_err(|e| e.to_string()))
            {
                Ok(page) => {
                    let ex = page.example;
                    ExampleMeta {
                        id: if ex.id.is_empty() { id.clone() } else { ex.id },
                        name: if ex.name.is_empty() {
                            id.clone()
                        } else {
                            ex.name
                        },
                        icon: ex.icon,
                        category: ex.category,
                        desc: ex.desc,
                        tags: ex.tags,
                        files: if ex.files.is_empty() {
                            default_files()
                        } else {
                            ex.files
                        },
                    }
                }
                Err(e) => {
                    eprintln!(
                        "Warning: failed to parse {}: {}",
                        page_toml_path.display(),
                        e
                    );
                    continue;
                }
            }
        } else {
            continue;
        };

        examples.push(meta);
    }

    // Sort: basic first, then alphabetical
    examples.sort_by(|a, b| {
        if a.id == "example-basic" {
            return std::cmp::Ordering::Less;
        }
        if b.id == "example-basic" {
            return std::cmp::Ordering::Greater;
        }
        a.id.cmp(&b.id)
    });

    let json = serde_json::to_string_pretty(&examples).expect("failed to serialize");
    let output_path = output_dir.join("examples.json");
    fs::write(&output_path, &json).expect("failed to write examples.json");

    println_cargo_style!(
        "Sync: {} examples -> {}",
        examples.len(),
        output_path.display()
    );
}
