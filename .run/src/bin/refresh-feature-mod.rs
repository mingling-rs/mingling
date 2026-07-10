use std::collections::BTreeSet;
use std::path::Path;

use just_fmt::snake_case;
use just_template::{tmpl, Template};
use tools::println_cargo_style;

const CARGO_TOML_PATH: &str = "./mingling/Cargo.toml";
const OUTPUT_PATH: &str = "./mingling/src/features.rs";

const TEMPLATE_CONTENT: &str = include_str!("../../../mingling/src/features.rs.tmpl");

fn main() {
    gen_feature_module();
}

fn gen_feature_module() {
    let repo_root = find_git_repo().unwrap();

    let cargo_toml_path = repo_root.join(CARGO_TOML_PATH);
    let output_path = repo_root.join(OUTPUT_PATH);

    let features = parse_features(&cargo_toml_path);

    let mut template = Template::from(TEMPLATE_CONTENT);

    for feat_name in &features {
        let feat_const_name = snake_case!(feat_name).to_uppercase();

        tmpl!(template += {
            features {
                (
                    feat_name = feat_name,
                    feat_const_name = feat_const_name
                )
            }
        });
        println_cargo_style!("Refresh: feature `{}`", feat_name);
    }

    let template_str = template.to_string();
    let template_str = template_str
        .lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    std::fs::write(&output_path, template_str).unwrap();

    println_cargo_style!("Written: features module to {}", OUTPUT_PATH);
}

/// Parse all feature names from the `[features]` section of a Cargo.toml.
fn parse_features(cargo_toml_path: &Path) -> Vec<String> {
    let content = std::fs::read_to_string(cargo_toml_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", cargo_toml_path.display(), e));

    let cargo_toml: toml::Value = content
        .parse()
        .unwrap_or_else(|e| panic!("Failed to parse {}: {}", cargo_toml_path.display(), e));

    let features_table = cargo_toml
        .get("features")
        .and_then(|v| v.as_table())
        .unwrap_or_else(|| {
            panic!(
                "No [features] section found in {}",
                cargo_toml_path.display()
            )
        });

    let mut feature_names: BTreeSet<String> = BTreeSet::new();
    for key in features_table.keys() {
        feature_names.insert(key.clone());
    }

    let mut result: Vec<String> = feature_names.into_iter().collect();
    result.sort();
    result
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
