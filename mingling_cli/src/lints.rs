#![allow(unused)]
use crate::linter::mlint_report::{MlintLevel, MlintReport};

mod template_linter;
pub use template_linter::linter as template_linter;
mod unnecessary_render_result_creation;
pub use unnecessary_render_result_creation::linter as unnecessary_render_result_creation;

/// Run all registered lints on a parsed file with its source text.
pub fn run_all_lints(file: &syn::File, source: &str) -> Vec<MlintReport> {
    use crate::linter::mlint_attr::{get_mlint_override, MlintLevelOverride};
    let mut reports = vec![];
    for item in &file.items {
        if let syn::Item::Fn(f) = item {
            let skip = get_mlint_override(&f.attrs, "template_linter") == Some(MlintLevelOverride::Allow);
            if !skip {
                let mut rs = template_linter::linter(f.clone(), source);
                if get_mlint_override(&f.attrs, "template_linter") == Some(MlintLevelOverride::Deny) {
                    for r in &mut rs { r.level = MlintLevel::Error; }
                }
                reports.extend(rs);
            }
            let skip = get_mlint_override(&f.attrs, "unnecessary_render_result_creation") == Some(MlintLevelOverride::Allow);
            if !skip {
                let mut rs = unnecessary_render_result_creation::linter(f.clone(), source);
                if get_mlint_override(&f.attrs, "unnecessary_render_result_creation") == Some(MlintLevelOverride::Deny) {
                    for r in &mut rs { r.level = MlintLevel::Error; }
                }
                reports.extend(rs);
            }

        }
    }
    for r in &mut reports {
        let name = &r.lint_code;
        if let Some(override_level) = get_mlint_override(&file.attrs, name) {
            match override_level {
                MlintLevelOverride::Allow => r.level = MlintLevel::Help,
                MlintLevelOverride::Deny => r.level = MlintLevel::Error,
                MlintLevelOverride::Warn => {
                    if r.level != MlintLevel::Error { r.level = MlintLevel::Warning; }
                }
            }
        }
    }
    reports.retain(|r| r.level != MlintLevel::Help);
    reports
}

#[macro_export]
macro_rules! assert_detected {
    ($linter:expr, $ast_type:ty => { $($code:tt)* }) => {
        // $($code:tt)* captures tokens INSIDE the braces, not including the braces
        // e.g. `fn foo() { ... }` — exactly what syn::ItemFn expects
        let source = stringify!($($code)*);
        let ast: $ast_type = syn::parse_str(&source).unwrap();
        assert!(!$linter(ast, &source).is_empty());
    };
}

#[macro_export]
macro_rules! assert_not_detected {
    ($linter:expr, $ast_type:ty => { $($code:tt)* }) => {
        let source = stringify!($($code)*);
        let ast: $ast_type = syn::parse_str(&source).unwrap();
        assert!($linter(ast, &source).is_empty());
    };
}