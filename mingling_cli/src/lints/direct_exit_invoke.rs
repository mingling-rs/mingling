//! Direct Exit Invocation
//!
//! ## Summary
//!
//! Detects direct `exit(...)` calls (i.e. `std::process::exit`) inside
//! functions annotated with `#[chain]`, `#[renderer]`, `#[command]`,
//! `#[completion]`, or `#[help]`.
//!
//! `exit()` terminates the whole process immediately and bypasses mingling's
//! `ExitCodeSetup` hook. Instead, declare an `&mut ResExitCode` resource
//! parameter and assign its `exit_code` field:
//!
//! ```text
//! exit(2);          // Before
//! ec.exit_code = 2; // After (with `ec: &mut ResExitCode` as a parameter)
//! ```
//!
//! The lint reuses an existing `&mut ResExitCode` parameter if the signature
//! already has one, and suggests changing `&ResExitCode` to `&mut ResExitCode`
//! if only an immutable borrow is present.
//!
//! ## Metadata
//!
//! Author: `Weicao-CatilGrass`
//! Default: `warn`

use crate::linter::mlint_report::{LintSuggestion, MlintLevel, MlintReport};
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::visit::Visit;

/// Resolved way of writing the exit code, based on the function signature.
struct ExitCodeParam {
    /// Identifier to use for `{name}.exit_code = ...`.
    name: String,
    /// Signature edits (`&` → `&mut`, or insert a new `ec: &mut ResExitCode`
    /// parameter). Consumed by the first report so it is only shown once.
    signature_suggestions: Option<Vec<LintSuggestion>>,
}

impl ExitCodeParam {
    /// Inspect the function signature and pick the parameter to use.
    fn resolve(sig: &syn::Signature, source: &str) -> Self {
        for arg in &sig.inputs {
            let syn::FnArg::Typed(pat_type) = arg else {
                continue;
            };
            let Some((name, is_mut)) = exit_code_param_info(pat_type) else {
                continue;
            };
            if is_mut {
                // Reuse the user's existing `&mut ResExitCode` parameter as-is.
                return Self {
                    name,
                    signature_suggestions: None,
                };
            }
            // Only an immutable `&ResExitCode` borrow exists — require `&mut`.
            return Self {
                name,
                signature_suggestions: Some(vec![make_amp_mut_suggestion(pat_type, source)]),
            };
        }
        // No `ResExitCode` borrow at all — suggest declaring a new parameter.
        Self {
            name: "ec".into(),
            signature_suggestions: Some(vec![make_add_param_suggestion(sig, source)]),
        }
    }
}

/// Returns `(param_name, is_mut)` when the parameter is `&ResExitCode` or
/// `&mut ResExitCode` (any leading path is accepted, e.g. `mingling::ResExitCode`).
fn exit_code_param_info(pat_type: &syn::PatType) -> Option<(String, bool)> {
    let syn::Type::Reference(ref_type) = &*pat_type.ty else {
        return None;
    };
    if !matches!(
        &*ref_type.elem,
        syn::Type::Path(type_path)
            if type_path.path.segments.last().is_some_and(|s| s.ident == "ResExitCode")
    ) {
        return None;
    }
    let syn::Pat::Ident(pat_ident) = &*pat_type.pat else {
        return None;
    };
    Some((pat_ident.ident.to_string(), ref_type.mutability.is_some()))
}

/// Suggestion: change `&ResExitCode` to `&mut ResExitCode` by replacing the `&` token.
fn make_amp_mut_suggestion(pat_type: &syn::PatType, source: &str) -> LintSuggestion {
    let syn::Type::Reference(ref_type) = &*pat_type.ty else {
        unreachable!("caller checked the reference type");
    };
    let span = ref_type.and_token.span();
    let start = span.start();
    let end = span.end();
    let line = source
        .lines()
        .nth(start.line.saturating_sub(1))
        .unwrap_or_default();
    LintSuggestion {
        source: line.to_string(),
        line_start: start.line,
        byte_range: start.column..end.column,
        replacement: "&mut ".into(),
    }
}

/// Suggestion: append `ec: &mut ResExitCode` to the parameter list.
///
/// The new parameter is inserted right after the last existing parameter (or
/// before the closing `)` when the list is empty) so the mandatory first
/// "previous type" parameter of `#[chain]`-style macros is left untouched.
fn make_add_param_suggestion(sig: &syn::Signature, source: &str) -> LintSuggestion {
    if let Some(last_arg) = sig.inputs.last() {
        let end = last_arg.span().end();
        let line = source
            .lines()
            .nth(end.line.saturating_sub(1))
            .unwrap_or_default();
        LintSuggestion {
            source: line.to_string(),
            line_start: end.line,
            byte_range: end.column..end.column,
            replacement: ", ec: &mut ResExitCode".into(),
        }
    } else {
        let close = sig.paren_token.span.close().start();
        let line = source
            .lines()
            .nth(close.line.saturating_sub(1))
            .unwrap_or_default();
        LintSuggestion {
            source: line.to_string(),
            line_start: close.line,
            byte_range: close.column..close.column,
            replacement: "ec: &mut ResExitCode".into(),
        }
    }
}

pub fn linter(ast: syn::ItemFn, source: &str) -> Vec<MlintReport> {
    let Some(attr_name) = find_mingling_attr(&ast) else {
        return vec![];
    };

    let mut finder = ExitCallFinder {
        source,
        attr_name,
        fn_name: ast.sig.ident.to_string(),
        param: ExitCodeParam::resolve(&ast.sig, source),
        reports: Vec::new(),
    };
    finder.visit_block(&ast.block);
    finder.reports
}

/// Returns the name of the first mingling attribute that applies to this lint.
///
/// Handles both bare (`#[renderer]`) and fully-qualified
/// (`#[::mingling::macros::renderer]`) paths by matching the last segment.
fn find_mingling_attr(ast: &syn::ItemFn) -> Option<String> {
    ast.attrs.iter().find_map(|attr| {
        let name = attr.path().segments.last()?.ident.to_string();
        matches!(
            name.as_str(),
            "renderer" | "chain" | "command" | "completion" | "help"
        )
        .then_some(name)
    })
}

struct ExitCallFinder<'a> {
    source: &'a str,
    attr_name: String,
    fn_name: String,
    param: ExitCodeParam,
    reports: Vec<MlintReport>,
}

impl<'ast> Visit<'ast> for ExitCallFinder<'_> {
    fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
        if is_exit_call(call) {
            self.check(call);
        }
        syn::visit::visit_expr_call(self, call);
    }

    // Nested items are linted on their own (or not at all); do not descend.
    fn visit_item(&mut self, _item: &'ast syn::Item) {}
}

impl ExitCallFinder<'_> {
    fn check(&mut self, call: &syn::ExprCall) {
        let mut suggestions = Vec::new();

        // `exit(2)` → `ec.exit_code = 2` (single-argument calls only).
        if call.args.len() == 1 {
            let arg_tokens = call.args.first().unwrap().to_token_stream().to_string();
            let replacement = format!("{}.exit_code = {arg_tokens}", self.param.name);
            if let Some(sugg) = single_line_suggestion(call, self.source, |start, end, _line| {
                Some((start, end, replacement))
            }) {
                suggestions.push(sugg);
            }
        }

        // Signature fix (`&` → `&mut`, or add the parameter) — shown once.
        if let Some(sig_suggestions) = self.param.signature_suggestions.take() {
            suggestions.extend(sig_suggestions);
        }

        let name = self.param.name.clone();
        self.reports.push(MlintReport {
            source_code: self.source.to_string(),
            level: MlintLevel::Warning,
            lint_code: "direct_exit_invoke".into(),
            message: format!(
                "direct `exit()` in `#[{}]` function `{}`: set `{name}.exit_code` instead of terminating the process",
                self.attr_name, self.fn_name,
            ),
            spans: vec![MlintReport::span_from_syn(call, self.source)],
            suggestions,
            attached_reports: vec![MlintReport {
                level: MlintLevel::Help,
                message: format!(
                    "`std::process::exit()` bypasses mingling's `ExitCodeSetup` hook; assign `{name}.exit_code` (declared as `{name}: &mut ResExitCode` parameter) and the framework exits with that code when the program finishes",
                ),
                ..Default::default()
            }],
            ..Default::default()
        });
    }
}

/// Returns `true` when the call targets `exit`, `process::exit`, etc.
fn is_exit_call(call: &syn::ExprCall) -> bool {
    matches!(
        &*call.func,
        syn::Expr::Path(path)
            if path
                .path
                .segments
                .last()
                .is_some_and(|seg| seg.ident == "exit")
    )
}

/// Build a suggestion from the call span when the whole call sits on one line.
fn single_line_suggestion(
    call: &syn::ExprCall,
    source: &str,
    range_of: impl FnOnce(usize, usize, &str) -> Option<(usize, usize, String)>,
) -> Option<LintSuggestion> {
    let span = call.span();
    let start = span.start();
    let end = span.end();
    if start.line != end.line {
        return None;
    }
    let line = source.lines().nth(start.line.saturating_sub(1))?;
    let (range_start, range_end, replacement) = range_of(start.column, end.column, line)?;
    Some(LintSuggestion {
        source: line.to_string(),
        line_start: start.line,
        byte_range: range_start..range_end,
        replacement,
    })
}

#[cfg(test)]
mod lint_test {
    use crate::{assert_detected, assert_not_detected};

    #[test]
    fn command_exit_detected() {
        assert_detected!(super::linter, syn::ItemFn => {
            #[command]
            fn do_exit(_: EntryGreet) -> Next {
                exit(1);
            }
        });
    }

    #[test]
    fn renderer_std_process_exit_detected() {
        assert_detected!(super::linter, syn::ItemFn => {
            #[renderer]
            fn render_greet(_: ResultGreet) {
                std::process::exit(2);
            }
        });
    }

    #[test]
    fn chain_fully_qualified_exit_detected() {
        assert_detected!(super::linter, syn::ItemFn => {
            #[chain]
            fn handle_greet(_: EntryGreet) {
                ::std::process::exit(0);
            }
        });
    }

    #[test]
    fn help_exit_detected() {
        assert_detected!(super::linter, syn::ItemFn => {
            #[help]
            fn help_greet(_: EntryGreet) {
                exit(3);
            }
        });
    }

    #[test]
    fn completion_exit_detected() {
        assert_detected!(super::linter, syn::ItemFn => {
            #[completion(EntryGreet)]
            fn complete(_: ShellContext) {
                exit(1);
            }
        });
    }

    #[test]
    fn no_attr_ok() {
        assert_not_detected!(super::linter, syn::ItemFn => {
            fn do_exit(_: EntryGreet) {
                exit(1);
            }
        });
    }

    #[test]
    fn exit_code_function_not_detected() {
        assert_not_detected!(super::linter, syn::ItemFn => {
            #[command]
            fn do_exit(_: EntryGreet) -> Next {
                let _ = exit_code();
            }
        });
    }

    #[test]
    fn nested_block_detected() {
        assert_detected!(super::linter, syn::ItemFn => {
            #[chain]
            fn handle_greet(_: EntryGreet) {
                if true {
                    exit(1);
                }
            }
        });
    }

    #[test]
    fn no_param_adds_ec_and_replaces_call() {
        let source = r#"#[command]
fn do_exit(_: EntryGreet) -> Next {
    exit(1);
}"#;
        let ast: syn::ItemFn = syn::parse_str(source).unwrap();
        let reports = super::linter(ast, source);
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].lint_code, "direct_exit_invoke");
        // 1. call-site replacement: `exit(1)` → `ec.exit_code = 1`
        let call_sugg = &reports[0].suggestions[0];
        assert_eq!(call_sugg.replacement, "ec.exit_code = 1");
        assert_eq!(call_sugg.line_start, 3);
        // 2. signature: append `ec: &mut ResExitCode` after the last parameter
        let sig_sugg = &reports[0].suggestions[1];
        assert_eq!(sig_sugg.replacement, ", ec: &mut ResExitCode");
        assert_eq!(sig_sugg.line_start, 2);
    }

    #[test]
    fn reuses_existing_mut_param_name() {
        let source = r#"#[command]
fn do_exit(_: EntryGreet, ex: &mut ResExitCode) {
    exit(2);
}"#;
        let ast: syn::ItemFn = syn::parse_str(source).unwrap();
        let reports = super::linter(ast, source);
        assert_eq!(reports.len(), 1);
        // Uses the user's name, no signature edit needed.
        assert_eq!(reports[0].suggestions.len(), 1);
        assert_eq!(reports[0].suggestions[0].replacement, "ex.exit_code = 2");
    }

    #[test]
    fn immutable_borrow_requires_mut() {
        let source = r#"#[chain]
fn handle_greet(_: EntryGreet, code: &ResExitCode) {
    exit(1);
}"#;
        let ast: syn::ItemFn = syn::parse_str(source).unwrap();
        let reports = super::linter(ast, source);
        assert_eq!(reports.len(), 1);
        // Reuses the name...
        assert_eq!(reports[0].suggestions[0].replacement, "code.exit_code = 1");
        // ...and rewrites `&` → `&mut `.
        assert_eq!(reports[0].suggestions[1].replacement, "&mut ");
    }

    #[test]
    fn sig_fix_shown_only_once_for_multiple_exit_calls() {
        let source = r#"#[command]
fn do_exit(_: EntryGreet) -> Next {
    exit(1);
    exit(2);
}"#;
        let ast: syn::ItemFn = syn::parse_str(source).unwrap();
        let reports = super::linter(ast, source);
        assert_eq!(reports.len(), 2);
        // First report: call fix + signature fix.
        assert_eq!(reports[0].suggestions.len(), 2);
        // Second report: call fix only.
        assert_eq!(reports[1].suggestions.len(), 1);
        assert_eq!(reports[1].suggestions[0].replacement, "ec.exit_code = 2");
    }

    #[test]
    fn empty_params_inserts_before_close_paren() {
        let source = r#"#[command]
fn do_exit() -> Next {
    exit(1);
}"#;
        let ast: syn::ItemFn = syn::parse_str(source).unwrap();
        let reports = super::linter(ast, source);
        assert_eq!(reports.len(), 1);
        let sig_sugg = &reports[0].suggestions[1];
        assert_eq!(sig_sugg.replacement, "ec: &mut ResExitCode");
        assert_eq!(sig_sugg.line_start, 2);
    }

    #[test]
    fn multi_line_params_append_after_last_param() {
        let source = r#"#[chain]
fn handle_greet(
    prev: EntryGreet,
) {
    exit(1);
}"#;
        let ast: syn::ItemFn = syn::parse_str(source).unwrap();
        let reports = super::linter(ast, source);
        assert_eq!(reports.len(), 1);
        let sig_sugg = &reports[0].suggestions[1];
        assert_eq!(sig_sugg.replacement, ", ec: &mut ResExitCode");
        // Appended on the line where the last parameter ends.
        assert_eq!(sig_sugg.line_start, 3);
    }
}
