//! Regenerates `mingling/src/features.rs` from the `[features]` section of
//! `mingling/Cargo.toml`.

use std::collections::HashMap;
use std::fs;

use just_fmt::snake_case;
use just_template::Template;
use mingling::{
    Grouped, RenderResult, Routable,
    macros::{buffer, command, r_println, renderer},
};

use crate::Next;
use crate::res::{CargoError, MessagePrinter};

const CARGO_TOML_PATH: &str = "./mingling/Cargo.toml";
const OUTPUT_PATH: &str = "./mingling/src/features.rs";
const TEMPLATE_CONTENT: &str = include_str!("../../../mingling/src/features.rs.tmpl");

#[command(node = "features-refresh")]
pub fn features_refresh() -> Next {
    match gen_feature_module() {
        Ok(written) => ResultFeaturesRefresh { written }.to_chain(),
        Err(e) => ErrorFeaturesRefresh(e).to_chain(),
    }
}

fn gen_feature_module() -> Result<Vec<String>, String> {
    let features = parse_features()?;

    let mut template = Template::from(TEMPLATE_CONTENT);
    let mut written = Vec::new();
    for feat_name in &features {
        let feat_const_name = snake_case!(feat_name).to_uppercase();
        template
            .add_impl("features".to_string())
            .push(HashMap::from([
                ("feat_name".to_string(), feat_name.clone()),
                ("feat_const_name".to_string(), feat_const_name),
            ]));
        written.push(format!("feature: {feat_name}"));
    }

    let template_str = template.to_string();
    let template_str = template_str
        .lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    fs::write(OUTPUT_PATH, template_str)
        .map_err(|e| format!("failed to write {OUTPUT_PATH}: {e}"))?;
    written.push(format!("written: {OUTPUT_PATH}"));
    Ok(written)
}

/// All feature names from the `[features]` section, sorted.
fn parse_features() -> Result<Vec<String>, String> {
    let content = fs::read_to_string(CARGO_TOML_PATH)
        .map_err(|e| format!("failed to read {CARGO_TOML_PATH}: {e}"))?;
    let table: toml::Value = content
        .parse()
        .map_err(|e| format!("failed to parse {CARGO_TOML_PATH}: {e}"))?;
    let features = table
        .get("features")
        .and_then(|v| v.as_table())
        .ok_or_else(|| format!("no [features] section in {CARGO_TOML_PATH}"))?;

    let mut names: Vec<String> = features.keys().cloned().collect();
    names.sort();
    Ok(names)
}

/// Feature names written by `features-refresh`.
#[derive(Grouped)]
pub struct ResultFeaturesRefresh {
    pub written: Vec<String>,
}

#[derive(Grouped, Default)]
pub struct ErrorFeaturesRefresh(pub String);

#[renderer(buffer)]
pub fn render_features_refresh(r: ResultFeaturesRefresh) {
    for item in r.written {
        r_println!("{item}");
    }
}

#[renderer]
pub fn render_error_features_refresh(e: ErrorFeaturesRefresh, error: &CargoError) -> RenderResult {
    let render_result = RenderResult::new();
    error.println(vec![e.0]);
    render_result
}
