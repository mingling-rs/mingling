//! Non-Mingling Naming Style
//!
//! ## Summary
//!
//! Checks that Mingling functions follow naming conventions:
//!
//! | Prefix          | 1st param must be |
//! |-----------------|-------------------|
//! | `handle_`       | `Entry*`          |
//! | `handle_state_` | `State*`          |
//! | `handle_error_` | `Error*`          |
//! | `help_`         | `Entry*`          |
//! | `render_`       | `Result*`         |
//! | `render_error_` | `Error*`          |
//!
//! The name after prefix (snake_case) must match the type after prefix (PascalCase).
//!
//! ## Metadata
//!
//! Author: `Weicao-CatilGrass`
//! Default: `warn`

use crate::linter::mlint_report::{LintSuggestion, MlintLevel, MlintReport};
use quote::ToTokens;
use syn::spanned::Spanned;

/// File-level entry (placeholder).
pub fn check_file(_file: &syn::File, _source: &str) -> Vec<MlintReport> {
    vec![]
}

/// ItemFn entry.
pub fn linter(ast: syn::ItemFn, source: &str) -> Vec<MlintReport> {
    check_fn_name(&ast, source)
}

fn check_fn_name(func: &syn::ItemFn, source: &str) -> Vec<MlintReport> {
    // Only check functions with Mingling attributes
    let has_mingling_attr = func.attrs.iter().any(|a| {
        let name = a.path().to_token_stream().to_string();
        name.ends_with("renderer")
            || name.ends_with("chain")
            || name.ends_with("help")
            || name.ends_with("completion")
    });
    if !has_mingling_attr {
        return vec![];
    }

    let name = func.sig.ident.to_string();
    let first_param = func.sig.inputs.first();

    let rule: Option<(&str, &str)> = {
        if name.starts_with("handle_state_") {
            Some(("handle_state_", "State"))
        } else if name.starts_with("handle_error_") {
            Some(("handle_error_", "Error"))
        } else if name.starts_with("handle_") {
            Some(("handle_", "Entry"))
        } else if name.starts_with("render_error_") {
            Some(("render_error_", "Error"))
        } else if name.starts_with("render_") {
            Some(("render_", "Result"))
        } else if name.starts_with("help_") {
            Some(("help_", "Entry"))
        } else {
            None
        }
    };

    let Some((prefix, expected_prefix)) = rule else {
        return vec![];
    };
    let fn_rest = &name[prefix.len()..];

    let Some(first) = first_param else {
        return vec![MlintReport {
            level: MlintLevel::Warning,
            lint_code: "non_mingling_naming_style".into(),
            message: format!(
                "`{name}` should take `{expected_prefix}*` as its first parameter, but it has no parameters"
            ),
            ..Default::default()
        }];
    };

    let type_name = param_type_name(first);

    if !type_name.starts_with(expected_prefix) || type_name.len() <= expected_prefix.len() {
        let expected_type = format!("{expected_prefix}{}", snake_to_pascal(fn_rest));
        return vec![MlintReport {
            level: MlintLevel::Warning,
            lint_code: "non_mingling_naming_style".into(),
            message: format!(
                "`{name}` should take `{expected_prefix}*`, but got `{type_name}` — rename it to `{expected_type}`"
            ),
            spans: vec![MlintReport::span_from_syn(&func.sig.ident, source)],
            ..Default::default()
        }];
    }

    let type_rest = &type_name[expected_prefix.len()..];
    let fn_rest_normalized = snake_to_pascal(fn_rest);

    if type_rest != fn_rest_normalized {
        let expected_type = format!("{expected_prefix}{fn_rest_normalized}");
        let expected_fn = format!("{prefix}{}", pascal_to_snake(type_rest));

        // Heuristic: if the type's base name is a substring of fn_rest, the type is clean → rename fn
        let rename_fn = fn_rest.to_lowercase().contains(&type_rest.to_lowercase())
            && fn_rest.to_lowercase() != type_rest.to_lowercase();

        let (msg, span) = if rename_fn {
            (
                format!(
                    "naming mismatch: rename `{name}` to `{expected_fn}` to match type `{type_name}`"
                ),
                MlintReport::span_from_syn(&func.sig.ident, source),
            )
        } else {
            (
                format!(
                    "naming mismatch: rename type `{type_name}` to `{expected_type}` to match function `{name}`"
                ),
                first_param_type_span(first, source),
            )
        };

        // Build diff suggestion
        let source_line = source
            .lines()
            .nth(func.sig.span().start().line.saturating_sub(1))
            .unwrap_or("");
        let (byte_range_start, suggestion_target) = if rename_fn {
            // Find the old function name in the source line
            let pos = source_line.find(&name).unwrap_or(0);
            (pos, expected_fn.clone())
        } else {
            // Find the old type name in the source line
            let pos = source_line.find(&type_name).unwrap_or(0);
            (pos, expected_type.clone())
        };
        let byte_range = byte_range_start
            ..byte_range_start
                + if rename_fn {
                    name.len()
                } else {
                    type_name.len()
                };

        return vec![MlintReport {
            level: MlintLevel::Warning,
            lint_code: "non_mingling_naming_style".into(),
            message: msg,
            spans: vec![span],
            suggestions: vec![LintSuggestion {
                source: source_line.to_string(),
                line_start: func.sig.span().start().line,
                byte_range,
                replacement: suggestion_target,
            }],
            attached_reports: vec![MlintReport {
                level: MlintLevel::Help,
                message: format!("expected `{expected_fn}` ↔ `{expected_type}`"),
                ..Default::default()
            }],
            ..Default::default()
        }];
    }

    vec![]
}

fn param_type_name(arg: &syn::FnArg) -> String {
    if let syn::FnArg::Typed(pat) = arg
        && let syn::Type::Path(ref tp) = *pat.ty.clone()
    {
        return tp
            .path
            .segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect::<Vec<_>>()
            .join("::");
    }
    String::new()
}

fn first_param_type_span(arg: &syn::FnArg, source: &str) -> crate::linter::mlint_report::LintSpan {
    if let syn::FnArg::Typed(pat) = arg {
        return MlintReport::span_from_syn(&*pat.ty, source);
    }
    MlintReport::span_from_syn(arg, source)
}

fn snake_to_pascal(s: &str) -> String {
    let mut r = String::new();
    let mut cap = true;
    for ch in s.chars() {
        if ch == '_' {
            cap = true;
        } else if cap {
            r.extend(ch.to_uppercase());
            cap = false;
        } else {
            r.push(ch);
        }
    }
    r
}

fn pascal_to_snake(s: &str) -> String {
    let mut r = String::new();
    for (i, ch) in s.char_indices() {
        if ch.is_uppercase() && i != 0 {
            r.push('_');
        }
        for lower in ch.to_lowercase() {
            r.push(lower);
        }
    }
    r
}

#[cfg(test)]
mod lint_test {
    use crate::{assert_detected, assert_not_detected};

    #[test]
    fn handle_entry() {
        assert_not_detected!(super::linter, syn::ItemFn => {
            #[::mingling::macros::chain]
            fn handle_greet(args: EntryGreet) {}
        });
    }

    #[test]
    fn handle_state() {
        assert_not_detected!(super::linter, syn::ItemFn => {
            #[::mingling::macros::chain]
            fn handle_state_processing(prev: StateProcessing) {}
        });
    }

    #[test]
    fn handle_error() {
        assert_not_detected!(super::linter, syn::ItemFn => {
            #[::mingling::macros::chain]
            fn handle_error_not_found(err: ErrorNotFound) {}
        });
    }

    #[test]
    fn render_result() {
        assert_not_detected!(super::linter, syn::ItemFn => {
            #[::mingling::macros::renderer]
            fn render_greeting(result: ResultGreeting) {}
        });
    }

    #[test]
    fn render_error() {
        assert_not_detected!(super::linter, syn::ItemFn => {
            #[::mingling::macros::renderer]
            fn render_error_not_found(err: ErrorNotFound) {}
        });
    }

    #[test]
    fn help_entry() {
        assert_not_detected!(super::linter, syn::ItemFn => {
            #[::mingling::macros::help]
            fn help_greet(args: EntryGreet) {}
        });
    }

    #[test]
    fn handle_should_be_entry() {
        assert_detected!(super::linter, syn::ItemFn => {
            #[::mingling::macros::chain]
            fn handle_greet(x: String) {}
        });
    }

    #[test]
    fn handle_state_should_be_state() {
        assert_detected!(super::linter, syn::ItemFn => {
            #[::mingling::macros::chain]
            fn handle_state_processing(x: String) {}
        });
    }

    #[test]
    fn handle_error_should_be_error() {
        assert_detected!(super::linter, syn::ItemFn => {
            #[::mingling::macros::chain]
            fn handle_error_not_found(x: String) {}
        });
    }

    #[test]
    fn render_should_be_result() {
        assert_detected!(super::linter, syn::ItemFn => {
            #[::mingling::macros::renderer]
            fn render_greeting(x: EntryGreet) {}
        });
    }

    #[test]
    fn render_error_should_be_error() {
        assert_detected!(super::linter, syn::ItemFn => {
            #[::mingling::macros::renderer]
            fn render_error_not_found(x: ResultGreeting) {}
        });
    }

    #[test]
    fn name_mismatch_rename_fn() {
        assert_detected!(super::linter, syn::ItemFn => {
            #[::mingling::macros::renderer]
            fn render_result_greeting(_greeting: ResultGreeting) {}
        });
    }

    #[test]
    fn name_mismatch_rename_type() {
        assert_detected!(super::linter, syn::ItemFn => {
            #[::mingling::macros::chain]
            fn handle_greet(args: EntryHello) {}
        });
    }

    #[test]
    fn handle_no_params() {
        assert_detected!(super::linter, syn::ItemFn => {
            #[::mingling::macros::chain]
            fn handle_greet() {}
        });
    }

    #[test]
    fn regular_fn_ok() {
        assert_not_detected!(super::linter, syn::ItemFn => {
            fn do_something(x: i32) {}
        });
    }

    #[test]
    fn handle_no_params_and_no_attrs() {
        assert_not_detected!(super::linter, syn::ItemFn => { fn handle_greet() {} });
    }
}
