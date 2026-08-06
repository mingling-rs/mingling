use mingling::{LazyInit, Program, macros::program_setup};
use serde::Deserialize;

use crate::ThisProgram;

#[derive(Debug, Default, Clone, Deserialize)]
pub struct ResLintRegistry {
    pub lints: Vec<LintEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LintEntry {
    pub name: String,
    pub title: String,
    pub summary: String,
    pub metadata: LintMetadata,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LintMetadata {
    pub active_on: String,
    pub author: String,
    pub default: String,
}

#[program_setup]
pub fn lint_registry_setup(p: &mut Program<ThisProgram>) {
    p.with_resource(ResLintRegistry::lazy_init(|| {
        let registry: ResLintRegistry = serde_json::from_str(include_str!("../../registry.json"))
            .expect("failed to parse embedded registry.json");
        registry
    }));
}
