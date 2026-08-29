//! Direct Stdout Bypass
//!
//! ## Summary
//!
//! Detects direct `println!`, `print!`, `eprint!`, and `eprintln!` calls inside
//! functions annotated with `#[chain]`, `#[renderer]`, `#[command]`,
//! `#[completion]`, or `#[help]`.
//!
//! In `#[renderer]` and `#[help]` functions the output should be written into
//! the `RenderResult` with the `r_println!` / `r_eprintln!` / `r_print!` /
//! `r_eprint!` family instead of going to stdout.
//!
//! In `#[chain]`, `#[command]`, and `#[completion]` functions there is no
//! render buffer, so direct stdout output bypasses the mingling pipeline
//! entirely — remove it (or route it through a renderer).
//!
//! ## Metadata
//!
//! Author: `Weicao-CatilGrass`
//! Default: `warn`

use crate::linter::mlint_report::{LintSuggestion, MlintLevel, MlintReport};
use syn::spanned::Spanned;
use syn::visit::Visit;

/// How a direct stdout call should be fixed, depending on the enclosing attribute.
#[derive(Clone, Copy)]
enum FixHint {
    /// `#[renderer]` / `#[help]` → rewrite to the `r_*` family
    UseRPrintFamily,
    /// `#[chain]` / `#[command]` / `#[completion]` → remove the call
    Remove,
}

pub fn linter(ast: syn::ItemFn, source: &str) -> Vec<MlintReport> {
    let Some((attr_name, fix_hint)) = find_mingling_attr(&ast) else {
        return vec![];
    };

    let mut finder = StdoutMacroFinder {
        source,
        attr_name,
        fix_hint,
        fn_name: ast.sig.ident.to_string(),
        reports: Vec::new(),
    };
    finder.visit_block(&ast.block);
    finder.reports
}

/// Returns the name of the first mingling attribute that applies to this lint,
/// together with the fix hint for that attribute.
///
/// Handles both bare (`#[renderer]`) and fully-qualified
/// (`#[::mingling::macros::renderer]`) paths by matching the last segment.
fn find_mingling_attr(ast: &syn::ItemFn) -> Option<(String, FixHint)> {
    ast.attrs.iter().find_map(|attr| {
        let name = attr.path().segments.last()?.ident.to_string();
        let hint = match name.as_str() {
            "renderer" | "help" => FixHint::UseRPrintFamily,
            "chain" | "command" | "completion" => FixHint::Remove,
            _ => return None,
        };
        Some((name, hint))
    })
}

struct StdoutMacroFinder<'a> {
    source: &'a str,
    attr_name: String,
    fix_hint: FixHint,
    fn_name: String,
    reports: Vec<MlintReport>,
}

impl<'ast> Visit<'ast> for StdoutMacroFinder<'_> {
    fn visit_stmt(&mut self, stmt: &'ast syn::Stmt) {
        if let syn::Stmt::Macro(stmt_mac) = stmt {
            self.check(&stmt_mac.mac, true);
        }
        syn::visit::visit_stmt(self, stmt);
    }

    fn visit_expr_macro(&mut self, expr_mac: &'ast syn::ExprMacro) {
        self.check(&expr_mac.mac, false);
        syn::visit::visit_expr_macro(self, expr_mac);
    }

    // Nested items are linted on their own (or not at all); do not descend,
    // otherwise a nested `#[chain] fn` would be classified with the outer attr.
    fn visit_item(&mut self, _item: &'ast syn::Item) {}
}

impl StdoutMacroFinder<'_> {
    fn check(&mut self, mac: &syn::Macro, is_stmt: bool) {
        let Some(mac_name) = stdout_macro_name(mac) else {
            return;
        };

        let suggestions = match self.fix_hint {
            FixHint::UseRPrintFamily => self.r_family_suggestion(mac, &mac_name),
            // Removing an expression-position macro would leave broken code
            // (e.g. `let x = ;`), so only suggest removal for whole statements.
            FixHint::Remove if is_stmt => self.removal_suggestion(mac),
            FixHint::Remove => Vec::new(),
        };

        let (message, help_message) = match self.fix_hint {
            FixHint::UseRPrintFamily => (
                format!(
                    "direct `{mac_name}!` in `#[{}]` function `{}`: use `r_{mac_name}!` to write into the render result instead of stdout",
                    self.attr_name, self.fn_name,
                ),
                "the `r_*` family (`r_println!`, `r_eprintln!`, `r_print!`, `r_eprint!`, `r_append!`) targets the `RenderResult` instead of stdout; with `#[renderer(buffer)]` use the implicit form `r_println!(...)` without the buffer argument".to_string(),
            ),
            FixHint::Remove => (
                format!(
                    "direct `{mac_name}!` in `#[{}]` function `{}`: remove it to avoid bypassing the mingling pipeline",
                    self.attr_name, self.fn_name,
                ),
                "direct stdout output bypasses mingling's render pipeline; remove the call or route the output through a `#[renderer]`".to_string(),
            ),
        };

        self.reports.push(MlintReport {
            source_code: self.source.to_string(),
            level: MlintLevel::Warning,
            lint_code: "direct_stdout_bypass".into(),
            message,
            spans: vec![MlintReport::span_from_syn(mac, self.source)],
            suggestions,
            attached_reports: vec![MlintReport {
                level: MlintLevel::Help,
                message: help_message,
                ..Default::default()
            }],
            ..Default::default()
        });
    }

    /// `println!` → `r_println!` (and friends) for single-segment, single-line calls.
    fn r_family_suggestion(&self, mac: &syn::Macro, mac_name: &str) -> Vec<LintSuggestion> {
        if mac.path.segments.len() != 1 {
            return Vec::new();
        }
        let r_name = format!("r_{mac_name}");
        single_line_suggestion(mac, self.source, |start, _end, _line| {
            Some((start, start + mac_name.len(), r_name))
        })
    }

    /// Remove the whole `println!(...);` statement (single-line calls only).
    fn removal_suggestion(&self, mac: &syn::Macro) -> Vec<LintSuggestion> {
        single_line_suggestion(mac, self.source, |start, mut end, line| {
            // Also swallow a trailing `;` so the whole statement vanishes cleanly.
            if line.as_bytes().get(end) == Some(&b';') {
                end += 1;
            }
            // If the call is the only thing on the line, blank the whole line.
            if line[..start].trim().is_empty() && line[end..].trim().is_empty() {
                Some((0, line.len(), String::new()))
            } else {
                Some((start, end, String::new()))
            }
        })
    }
}

/// Returns the macro name when `mac` is one of the direct stdout macros.
fn stdout_macro_name(mac: &syn::Macro) -> Option<String> {
    let name = mac.path.segments.last()?.ident.to_string();
    matches!(name.as_str(), "println" | "print" | "eprint" | "eprintln").then_some(name)
}

/// Build a suggestion from the macro span when the whole call sits on one line.
fn single_line_suggestion(
    mac: &syn::Macro,
    source: &str,
    range_of: impl FnOnce(usize, usize, &str) -> Option<(usize, usize, String)>,
) -> Vec<LintSuggestion> {
    let span = mac.span();
    let start = span.start();
    let end = span.end();
    if start.line != end.line {
        return Vec::new();
    }
    let Some(line) = source.lines().nth(start.line.saturating_sub(1)) else {
        return Vec::new();
    };
    let Some((range_start, range_end, replacement)) = range_of(start.column, end.column, line)
    else {
        return Vec::new();
    };
    vec![LintSuggestion {
        source: line.to_string(),
        line_start: start.line,
        byte_range: range_start..range_end,
        replacement,
    }]
}

#[cfg(test)]
mod lint_test {
    use crate::{assert_detected, assert_not_detected};

    #[test]
    fn renderer_plain_println() {
        assert_detected!(super::linter, syn::ItemFn => {
            #[renderer]
            fn render_greet(_: ResultGreet) {
                println!("hello");
            }
        });
    }

    #[test]
    fn renderer_buffer_eprintln() {
        assert_detected!(super::linter, syn::ItemFn => {
            #[renderer(buffer)]
            fn render_greet(_: ResultGreet) {
                eprintln!("oops");
            }
        });
    }

    #[test]
    fn help_print() {
        assert_detected!(super::linter, syn::ItemFn => {
            #[help]
            fn help_greet(_: EntryGreet) {
                print!("help");
            }
        });
    }

    #[test]
    fn chain_println() {
        assert_detected!(super::linter, syn::ItemFn => {
            #[chain]
            fn handle_greet(_: EntryGreet) {
                println!("hi");
            }
        });
    }

    #[test]
    fn command_println() {
        assert_detected!(super::linter, syn::ItemFn => {
            #[command]
            fn greet(args: Vec<String>) -> Next {
                println!("{args:?}");
            }
        });
    }

    #[test]
    fn completion_eprint() {
        assert_detected!(super::linter, syn::ItemFn => {
            #[completion(EntryGreet)]
            fn complete(_: ShellContext) {
                eprint!(".");
            }
        });
    }

    #[test]
    fn fully_qualified_renderer() {
        assert_detected!(super::linter, syn::ItemFn => {
            #[::mingling::macros::renderer]
            fn render_greet(_: ResultGreet) {
                println!("hello");
            }
        });
    }

    #[test]
    fn no_attr_ok() {
        assert_not_detected!(super::linter, syn::ItemFn => {
            fn greet(_: EntryGreet) {
                println!("hi");
            }
        });
    }

    #[test]
    fn renderer_uses_r_println_ok() {
        assert_not_detected!(super::linter, syn::ItemFn => {
            #[renderer(buffer)]
            fn render_greet(_: ResultGreet) {
                r_println!("hello");
            }
        });
    }

    #[test]
    fn chain_uses_r_println_still_ok() {
        assert_not_detected!(super::linter, syn::ItemFn => {
            #[chain]
            fn handle_greet(_: EntryGreet) {
                r_println!("hi");
            }
        });
    }

    #[test]
    fn nested_block_detected() {
        assert_detected!(super::linter, syn::ItemFn => {
            #[chain]
            fn handle_greet(_: EntryGreet) {
                if true {
                    println!("hi");
                }
            }
        });
    }

    #[test]
    fn nested_fn_not_detected() {
        assert_not_detected!(super::linter, syn::ItemFn => {
            #[chain]
            fn handle_greet(_: EntryGreet) {
                fn helper() {
                    println!("hi");
                }
            }
        });
    }

    #[test]
    fn renderer_suggestion_rewrites_to_r_println() {
        let source = r#"#[renderer]
fn render_greet(_: ResultGreet) {
    println!("hello");
}"#;
        let ast: syn::ItemFn = syn::parse_str(source).unwrap();
        let reports = super::linter(ast, source);
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].lint_code, "direct_stdout_bypass");
        // The warning span sits on the macro call (line 3).
        assert_eq!(reports[0].spans[0].line_start, 3);
        // Machine-applicable rewrite `println!` → `r_println!`.
        let sugg = &reports[0].suggestions[0];
        assert_eq!(sugg.replacement, "r_println");
        assert_eq!(&sugg.source[sugg.byte_range.clone()], "println");
        assert_eq!(sugg.line_start, 3);
    }

    #[test]
    fn chain_removal_suggestion() {
        let source = r#"#[chain]
fn handle_greet(_: EntryGreet) {
    println!("hi");
}"#;
        let ast: syn::ItemFn = syn::parse_str(source).unwrap();
        let reports = super::linter(ast, source);
        assert_eq!(reports.len(), 1);
        // Whole statement removed (including the `;`).
        let sugg = &reports[0].suggestions[0];
        assert_eq!(sugg.replacement, "");
        assert_eq!(sugg.byte_range, 0..sugg.source.len());
    }

    #[test]
    fn expression_macro_in_chain_has_no_removal_suggestion() {
        let source = r#"#[chain]
fn handle_greet(_: EntryGreet) {
    let _ = println!("hi");
}"#;
        let ast: syn::ItemFn = syn::parse_str(source).unwrap();
        let reports = super::linter(ast, source);
        assert_eq!(reports.len(), 1);
        assert!(reports[0].suggestions.is_empty());
    }
}
