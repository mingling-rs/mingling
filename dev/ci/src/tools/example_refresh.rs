//! Regenerates the example documentation module and the examples index.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use just_fmt::snake_case;
use just_template::Template;
use mingling::{
    Grouped, RenderResult, Routable,
    macros::{buffer, command, r_println, renderer},
};
use serde::Serialize;

use crate::Next;
use crate::res::{CargoError, MessagePrinter};

const EXAMPLE_ROOT: &str = "./examples";
const EXAMPLE_DOCS_OUTPUT: &str = "./mingling/src/example_docs.rs";
const EXAMPLE_DOCS_TEMPLATE: &str = include_str!("../../../../mingling/src/example_docs.rs.tmpl");
const EXAMPLES_JSON_OUTPUT: &str = "./docs/examples.json";

#[command(node = "example-refresh")]
pub fn example_refresh() -> Next {
    match refresh_all() {
        Ok(written) => ResultExampleRefresh { written }.to_chain(),
        Err(e) => ErrorExampleRefresh(e).to_chain(),
    }
}

fn refresh_all() -> Result<Vec<String>, String> {
    let mut written = Vec::new();
    written.extend(refresh_example_docs()?);
    written.extend(sync_examples()?);
    Ok(written)
}

/// Part 1: regenerate `mingling/src/example_docs.rs` from the examples'
/// `src/main.rs` (header `//!` + code) and `Cargo.toml`.
fn refresh_example_docs() -> Result<Vec<String>, String> {
    let mut template = Template::from(EXAMPLE_DOCS_TEMPLATE);

    let mut examples = Vec::new();
    let entries =
        fs::read_dir(EXAMPLE_ROOT).map_err(|e| format!("failed to read {EXAMPLE_ROOT}: {e}"))?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.starts_with("example-") {
            continue;
        }
        examples.push(ExampleContent::read(&name));
    }
    examples.sort_by(|a, b| a.name.cmp(&b.name));

    let mut written = Vec::new();
    for example in examples {
        template
            .add_impl("examples".to_string())
            .push(HashMap::from([
                ("example_header".to_string(), example.header),
                ("example_import".to_string(), example.cargo_toml),
                ("example_code".to_string(), example.code),
                ("example_name".to_string(), snake_case!(&example.name)),
            ]));
        written.push(format!("example_docs: {}", example.name));
    }

    let template_str = template.to_string();
    let template_str = template_str
        .lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    fs::write(EXAMPLE_DOCS_OUTPUT, template_str)
        .map_err(|e| format!("failed to write {EXAMPLE_DOCS_OUTPUT}: {e}"))?;
    written.push(format!("written: {EXAMPLE_DOCS_OUTPUT}"));
    Ok(written)
}

struct ExampleContent {
    name: String,
    header: String,
    code: String,
    cargo_toml: String,
}

impl ExampleContent {
    fn read(name: &str) -> Self {
        let prefix = |s: &str| {
            s.lines()
                .map(|line| format!("/// {line}"))
                .collect::<Vec<_>>()
                .join("\n")
        };

        let (header, code) = read_header_and_code(name);
        Self {
            name: name.to_string(),
            header: prefix(&header),
            code: prefix(&code),
            cargo_toml: prefix(&read_cargo_toml(name)),
        }
    }
}

/// Reads an example's `src/main.rs`, splitting `//!` doc header from code.
fn read_header_and_code(name: &str) -> (String, String) {
    let content = fs::read_to_string(Path::new(EXAMPLE_ROOT).join(name).join("src/main.rs"))
        .unwrap_or_default();
    let mut lines = content.lines();
    let mut header = String::new();
    let mut code = String::new();

    for line in lines.by_ref() {
        if line.trim_start().starts_with("//!") {
            header.push_str(line.trim_start_matches("//!"));
            header.push('\n');
        } else {
            code.push_str(line);
            code.push('\n');
            break;
        }
    }
    for line in lines {
        code.push_str(line);
        code.push('\n');
    }

    (header.trim().to_string(), code.trim().to_string())
}

fn read_cargo_toml(name: &str) -> String {
    fs::read_to_string(Path::new(EXAMPLE_ROOT).join(name).join("Cargo.toml")).unwrap_or_default()
}

/// Part 2: regenerate `docs/example-pages/examples.json` from each example's
/// `page.toml`.
fn sync_examples() -> Result<Vec<String>, String> {
    fs::create_dir_all("docs/example-pages")
        .map_err(|e| format!("failed to create docs/example-pages: {e}"))?;

    let mut examples = Vec::new();
    let entries =
        fs::read_dir(EXAMPLE_ROOT).map_err(|e| format!("failed to read {EXAMPLE_ROOT}: {e}"))?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let dir_name = entry.file_name().to_string_lossy().into_owned();
        let page_toml = path.join("page.toml");
        if !page_toml.is_file() {
            continue;
        }
        let Ok(content) = fs::read_to_string(&page_toml) else {
            continue;
        };
        let Ok(table) = content.parse::<toml::Value>() else {
            eprintln!("Warning: failed to parse {}", page_toml.display());
            continue;
        };
        let Some(example) = table.get("example") else {
            continue;
        };

        let get = |key: &str| {
            example
                .get(key)
                .and_then(|v| v.as_str())
                .unwrap_or_default()
        };
        let str_vec = |key: &str| {
            example
                .get(key)
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default()
        };

        let id = get("id");
        examples.push(ExampleMeta {
            id: if id.is_empty() {
                dir_name.clone()
            } else {
                id.to_string()
            },
            name: {
                let name = get("name");
                if name.is_empty() {
                    dir_name.clone()
                } else {
                    name.to_string()
                }
            },
            icon: {
                let icon = get("icon");
                if icon.is_empty() {
                    "📦".to_string()
                } else {
                    icon.to_string()
                }
            },
            category: get("category").to_string(),
            desc: get("desc").to_string(),
            tags: str_vec("tags"),
            files: {
                let files = str_vec("files");
                if files.is_empty() {
                    vec!["Cargo.toml".to_string(), "src/main.rs".to_string()]
                } else {
                    files
                }
            },
        });
    }

    // Basic first, then alphabetical.
    examples.sort_by(
        |a, b| match (a.id == "example-basic", b.id == "example-basic") {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.id.cmp(&b.id),
        },
    );

    let json = serde_json::to_string_pretty(&examples)
        .map_err(|e| format!("failed to serialize examples: {e}"))?;
    fs::write(EXAMPLES_JSON_OUTPUT, json)
        .map_err(|e| format!("failed to write {EXAMPLES_JSON_OUTPUT}: {e}"))?;

    Ok(vec![format!(
        "synced: {} examples -> {EXAMPLES_JSON_OUTPUT}",
        examples.len()
    )])
}

/// One entry of `docs/example-pages/examples.json`.
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

/// Files written by `example-refresh`.
#[derive(Grouped)]
pub struct ResultExampleRefresh {
    pub written: Vec<String>,
}

#[derive(Grouped, Default)]
pub struct ErrorExampleRefresh(pub String);

#[renderer(buffer)]
pub fn render_example_refresh(r: ResultExampleRefresh) {
    for item in r.written {
        r_println!("{item}");
    }
}

#[renderer]
pub fn render_error_example_refresh(e: ErrorExampleRefresh, error: &CargoError) -> RenderResult {
    let render_result = RenderResult::new();
    error.println(vec![e.0]);
    render_result
}
