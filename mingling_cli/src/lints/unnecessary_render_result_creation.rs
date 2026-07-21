//! Unnecessary Manual RenderResult Creation
//!
//! ## Summary
//!
//! Detects `#[renderer]` functions that manually create a `RenderResult` and
//! manage it via `r_println!(r, ...)` style calls, when they could be simplified
//! to `#[renderer(buffer)]` which handles the buffer automatically.
//!
//! This lint will not trigger if `r` is used outside of `r_println!`,
//! `r_eprintln!`, `r_print`, `r_eprint`, or `r_append`.
//!
//! ## Metadata
//!
//! Author: `Weicao-CatilGrass`
//! Default: `warn`

use crate::linter::mlint_report::{LintSuggestion, MlintLevel, MlintReport};
use quote::ToTokens;
use syn::spanned::Spanned;

pub fn linter(ast: syn::ItemFn, source: &str) -> Vec<MlintReport> {
    if !has_renderer_attr(&ast) {
        return vec![];
    }

    let stmts = &ast.block.stmts;
    if stmts.len() < 2 {
        return vec![];
    }

    let (r_ident, let_idx) = match find_render_result_new(stmts) {
        Some(pair) => pair,
        None => return vec![],
    };

    let r_name = r_ident.to_string();
    let mut only_print_and_return = true;
    let mut print_count = 0;

    for stmt in &stmts[let_idx + 1..] {
        if !check_stmt_usage(stmt, &r_name, &mut print_count) {
            only_print_and_return = false;
            break;
        }
    }

    if only_print_and_return && print_count > 0 {
        let span = MlintReport::span_from_syn(&ast.sig, source);
        let mut suggestions = Vec::new();

        // 1. Attribute change: #[renderer] → #[renderer(buffer)]
        if let Some(sugg) = make_attr_suggestion(&ast, source) {
            suggestions.push(sugg);
        }

        // 2. Remove -> RenderResult from function signature
        if let Some(sugg) = make_return_type_suggestion(&ast, source) {
            suggestions.push(sugg);
        }

        // 3. Remove let mut r = RenderResult::...
        if let Some(sugg) = make_let_removal_suggestion(stmts, let_idx, source) {
            suggestions.push(sugg);
        }

        // 4. Fix r_println!(r, ...) → r_println!(...) for all r_xxx macros
        suggestions.extend(make_macro_arg_suggestions(stmts, &r_name, source));

        // 5. Remove return 'r' expression
        if let Some(sugg) = make_return_removal_suggestion(stmts, &r_name, source) {
            suggestions.push(sugg);
        }

        vec![MlintReport {
            source_code: source.to_string(),
            level: MlintLevel::Warning,
            lint_code: "unnecessary_render_result_creation".into(),
            message: format!(
                "unnecessary manual `RenderResult` creation in `{}`: use `#[renderer(buffer)]` instead",
                ast.sig.ident,
            ),
            spans: vec![span],
            suggestions,
            attached_reports: vec![MlintReport {
                level: MlintLevel::Help,
                message: format!(
                    "change to `#[renderer(buffer)]` and use `r_println!(...)` without the `{}` parameter",
                    r_name,
                ),
                ..Default::default()
            }],
            ..Default::default()
        }]
    } else {
        vec![]
    }
}

fn has_renderer_attr(func: &syn::ItemFn) -> bool {
    func.attrs.iter().any(|a| {
        if !a.path().is_ident("renderer") {
            return false;
        }
        if let Ok(list) = a.meta.require_list() {
            let tokens = list.tokens.to_string();
            if tokens.contains("buffer") {
                return false;
            }
        }
        true
    })
}

fn find_render_result_new(stmts: &[syn::Stmt]) -> Option<(proc_macro2::Ident, usize)> {
    for (i, stmt) in stmts.iter().enumerate() {
        if let syn::Stmt::Local(local) = stmt
            && let Some(init) = &local.init
            && let syn::Pat::Ident(pat_id) = &local.pat
            && pat_id.mutability.is_some()
            && let syn::Expr::Call(call) = &*init.expr
            && let syn::Expr::Path(expr_path) = call.func.as_ref()
        {
            let segs = &expr_path.path.segments;
            let matches = match segs.len() {
                2 => {
                    segs[0].ident == "RenderResult"
                        && (segs[1].ident == "new" || segs[1].ident == "default")
                }
                3 => {
                    segs[0].ident == "mingling"
                        && segs[1].ident == "RenderResult"
                        && (segs[2].ident == "new" || segs[2].ident == "default")
                }
                _ => false,
            };
            if matches {
                return Some((pat_id.ident.clone(), i));
            }
        }

        // Also handle `RenderResult::from(...)` and `mingling::RenderResult::from(...)`
        if let syn::Stmt::Local(local) = stmt
            && let Some(init) = &local.init
            && let syn::Pat::Ident(pat_id) = &local.pat
            && pat_id.mutability.is_some()
            && let syn::Expr::Call(call) = &*init.expr
            && let syn::Expr::Path(expr_path) = call.func.as_ref()
        {
            let segs = &expr_path.path.segments;
            let matches = match segs.len() {
                2 => segs[0].ident == "RenderResult" && segs[1].ident == "from",
                3 => {
                    segs[0].ident == "mingling"
                        && segs[1].ident == "RenderResult"
                        && segs[2].ident == "from"
                }
                _ => false,
            };
            if matches {
                return Some((pat_id.ident.clone(), i));
            }
        }
    }
    None
}

fn check_stmt_usage(stmt: &syn::Stmt, r_name: &str, print_count: &mut usize) -> bool {
    // return r; → allowed
    if let syn::Stmt::Expr(expr, _) = stmt
        && let syn::Expr::Return(ret) = expr
        && let Some(ret_expr) = &ret.expr
        && let syn::Expr::Path(p) = ret_expr.as_ref()
        && p.path.is_ident(r_name)
    {
        return true;
    }

    // r_println!(r, ...) → allowed
    if let syn::Stmt::Macro(stmt_mac) = stmt {
        let macro_name = stmt_mac
            .mac
            .path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default();
        let is_r_macro = matches!(
            macro_name.as_str(),
            "r_println" | "r_eprintln" | "r_print" | "r_eprint" | "r_append"
        );
        if is_r_macro
            && let Some(first_arg) = stmt_mac.mac.tokens.clone().into_iter().next()
            && first_arg.to_string() == *r_name
        {
            *print_count += 1;
            return true;
        }
    }

    // Any other reference to r → not allowed
    // Check the token stream for the variable name
    let ts_string = stmt.to_token_stream().to_string();
    if ts_string.contains(&format!("({r_name})"))
        || ts_string.contains(&format!(" {r_name})"))
        || ts_string.contains(&format!("(&mut {r_name})"))
        || ts_string.contains(&format!("&{r_name}"))
        || ts_string.contains(&format!("move {r_name}"))
        || ts_string.contains(&format!(",{r_name},"))
    {
        return false;
    }
    true
}

/// Build suggestion: `#[renderer]` → `#[renderer(buffer)]`
fn make_attr_suggestion(ast: &syn::ItemFn, source: &str) -> Option<LintSuggestion> {
    let attr = ast.attrs.iter().find(|a| {
        let name = a.path().to_token_stream().to_string();
        name.ends_with("renderer")
    })?;

    let line_idx = attr.span().start().line.saturating_sub(1);
    let line = source.lines().nth(line_idx)?;

    // Replace `renderer]` with `renderer(buffer)]`
    // This handles both `#[renderer]` and `#[::mingling::macros::renderer]`
    let line_str = line;
    let replacement = line_str.replacen("renderer]", "renderer(buffer)]", 1);

    if replacement == line_str {
        return None;
    }

    Some(LintSuggestion {
        source: line_str.to_string(),
        line_start: line_idx + 1,
        byte_range: 0..line_str.len(),
        replacement,
    })
}

/// Build suggestion: remove ` -> RenderResult` from function signature
fn make_return_type_suggestion(ast: &syn::ItemFn, source: &str) -> Option<LintSuggestion> {
    let syn::ReturnType::Type(arrow, ret_type) = &ast.sig.output else {
        return None;
    };

    let sig_line_idx = ast.sig.span().start().line.saturating_sub(1);
    let line = source.lines().nth(sig_line_idx)?;

    // proc-macro2 column is 0-based byte offset from line start
    let arrow_byte_col = arrow.span().start().column;
    let ret_end_byte_col = ret_type.span().end().column;

    // Include the space before `->`
    let range_start = if arrow_byte_col > 0 {
        arrow_byte_col - 1
    } else {
        arrow_byte_col
    };

    Some(LintSuggestion {
        source: line.to_string(),
        line_start: sig_line_idx + 1,
        byte_range: range_start..ret_end_byte_col,
        replacement: String::new(),
    })
}

/// Build suggestion: remove `let mut r = RenderResult::...` line
fn make_let_removal_suggestion(
    stmts: &[syn::Stmt],
    let_idx: usize,
    source: &str,
) -> Option<LintSuggestion> {
    let stmt = &stmts[let_idx];
    let line_idx = stmt.span().start().line.saturating_sub(1);
    let line = source.lines().nth(line_idx)?;

    Some(LintSuggestion {
        source: line.to_string(),
        line_start: line_idx + 1,
        byte_range: 0..line.len(),
        replacement: String::new(),
    })
}

/// Build suggestions: fix `r_println!(r, ...)` → `r_println!(...)` for all r_xxx macros
fn make_macro_arg_suggestions(
    stmts: &[syn::Stmt],
    r_name: &str,
    source: &str,
) -> Vec<LintSuggestion> {
    let r_macros = ["r_println", "r_eprintln", "r_print", "r_eprint"];

    stmts
        .iter()
        .filter_map(|stmt| {
            let syn::Stmt::Macro(stmt_mac) = stmt else {
                return None;
            };

            let macro_name = stmt_mac
                .mac
                .path
                .segments
                .last()
                .map(|s| s.ident.to_string())
                .unwrap_or_default();

            if !r_macros.contains(&macro_name.as_str()) {
                return None;
            }

            // Check that the first token is the r_name
            let first_token = stmt_mac.mac.tokens.clone().into_iter().next()?;
            if first_token.to_string() != *r_name {
                return None;
            }

            let line_idx = stmt.span().start().line.saturating_sub(1);
            let line = source.lines().nth(line_idx)?;

            // Find pattern: macro_name!(r_name, ...
            let macro_str = format!("{}!(", macro_name);
            let macro_pos = line.find(&macro_str)?;
            let after_open = macro_pos + macro_str.len();

            // The first argument is `r_name` followed by `,` and possibly a space
            // We need to find and remove `r_name, ` or `r_name,`
            let first_arg = r_name;
            if line[after_open..].starts_with(first_arg) {
                // Find the end of the first argument (including `,` and any whitespace)
                let arg_end = after_open + first_arg.len();
                if arg_end < line.len() {
                    let rest = &line[arg_end..];
                    // Skip `,` and optional whitespace
                    let skip = if rest.starts_with(", ") {
                        2
                    } else if rest.starts_with(',') {
                        1
                    } else {
                        // Not followed by comma — not our pattern
                        return None;
                    };
                    let range_end = arg_end + skip;

                    Some(LintSuggestion {
                        source: line.to_string(),
                        line_start: line_idx + 1,
                        byte_range: after_open..range_end,
                        replacement: String::new(),
                    })
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect()
}

/// Build suggestion: remove the return `r` expression
fn make_return_removal_suggestion(
    stmts: &[syn::Stmt],
    r_name: &str,
    source: &str,
) -> Option<LintSuggestion> {
    let last = stmts.last()?;

    let is_r_return = match last {
        // `r` (bare expression, no semicolon) or `r;` (with semicolon)
        syn::Stmt::Expr(expr, _) => {
            if let syn::Expr::Path(p) = expr {
                p.path.is_ident(r_name)
            } else if let syn::Expr::Return(ret) = expr {
                ret.expr.as_ref().is_some_and(|e| {
                    if let syn::Expr::Path(p) = e.as_ref() {
                        p.path.is_ident(r_name)
                    } else {
                        false
                    }
                })
            } else {
                false
            }
        }
        _ => false,
    };

    if !is_r_return {
        return None;
    }

    let line_idx = last.span().start().line.saturating_sub(1);
    let line = source.lines().nth(line_idx)?;

    Some(LintSuggestion {
        source: line.to_string(),
        line_start: line_idx + 1,
        byte_range: 0..line.len(),
        replacement: String::new(),
    })
}

#[cfg(test)]
mod lint_test {
    use crate::{assert_detected, assert_not_detected};

    #[test]
    fn test_detected_render_result_new() {
        assert_detected!(super::linter, syn::ItemFn => {
            #[renderer]
            fn render_somesthing(_: Prev) -> RenderResult {
                let mut r = RenderResult::new();
                r_println!(r, "");
                r
            }
        });
    }

    #[test]
    fn test_detected_render_result_default() {
        assert_detected!(super::linter, syn::ItemFn => {
            #[renderer]
            fn render_somesthing(_: Prev) -> RenderResult {
                let mut r = RenderResult::default();
                r_println!(r, "");
                r
            }
        });
    }

    #[test]
    fn test_detected_render_result_from() {
        assert_detected!(super::linter, syn::ItemFn => {
            #[renderer]
            fn render_somesthing(_: Prev) -> RenderResult {
                let mut r = RenderResult::from("Hello".to_string());
                r_println!(r, "");
                r
            }
        });
    }

    #[test]
    fn test_not_detected_with_other_function_call() {
        assert_not_detected!(super::linter, syn::ItemFn => {
            #[renderer]
            fn render_somesthing(_: Prev) -> RenderResult {
                let mut r = RenderResult::new();
                r_println!(r, "");
                other(&mut r);
                r
            }
        });
    }

    #[test]
    fn test_not_detected_without_renderer_attr() {
        assert_not_detected!(super::linter, syn::ItemFn => {
            fn render_somesthing(_: Prev) -> RenderResult {
                let mut r = RenderResult::new();
                r_println!(r, "");
                r
            }
        });
    }

    #[test]
    fn test_not_detected_with_buffer_attr() {
        assert_not_detected!(super::linter, syn::ItemFn => {
            #[::mingling::macros::renderer(::mingling::macros::buffer)]
            fn render_somesthing(_: Prev) {
                r_println!("");
            }
        });
    }

    #[test]
    fn test_not_detected_with_buffer_attr_fully_qualified() {
        assert_not_detected!(super::linter, syn::ItemFn => {
            #[::mingling::macros::renderer(::mingling::macros::buffer)]
            fn render_somesthing(_: Prev) {
                r_println!("");
            }
        });
    }
}
