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

use crate::linter::mlint_report::{MlintLevel, MlintReport};
use quote::ToTokens;

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
        vec![MlintReport {
            source_code: source.to_string(),
            level: MlintLevel::Warning,
            lint_code: "unnecessary_render_result_creation".into(),
            message: format!(
                "unnecessary manual `RenderResult` creation in `{}`: use `#[renderer(buffer)]` instead",
                ast.sig.ident,
            ),
            spans: vec![span],
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
