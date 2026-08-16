use std::ops::Range;

use cargo_metadata::diagnostic::{
    DiagnosticCodeBuilder, DiagnosticLevel as CargoLevel, DiagnosticSpanBuilder,
    DiagnosticSpanLineBuilder,
};

use cargo_metadata::{Message, PackageId};

use annotate_snippets::level::{ERROR, HELP, NOTE, WARNING};
use annotate_snippets::{AnnotationKind, Group, Patch, Renderer, Snippet};
use mingling::macros::{buffer, chain, r_append, r_eprintln, renderer};
use mingling::{Grouped, RendererInvoker, Routable, Wrap};

use crate::Next;
use crate::metadata::setup::ResUsingJson;

/// Complete structure of a Lint report, containing inspection results and associated metadata.
#[derive(Default)]
pub struct MlintReport {
    /// Source file name
    pub file_name: String,

    /// Full source text of the file, used to extract line content and compute byte offsets
    pub source_code: String,

    /// Severity level of the report
    pub level: MlintLevel,

    /// Name of the Lint
    pub lint_code: String,

    /// Content of the report
    pub message: String,

    /// Source code locations
    pub spans: Vec<LintSpan>,

    /// Attached sub-reports for this report
    pub attached_reports: Vec<MlintReport>,

    /// Package ID that this report belongs to
    pub package_id: Option<String>,

    /// Compilation target name that this report belongs to
    pub target_name: Option<String>,

    /// Compilation target type that this report belongs to
    pub target_kind: Option<String>,

    /// Compilation target source file path that this report belongs to
    pub target_src_path: Option<String>,

    /// Suggestions for automatic fix (shown as diff in annotated output)
    pub suggestions: Vec<LintSuggestion>,
}

/// Report severity level, indicating the seriousness of the Lint result.
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum MlintLevel {
    #[default]
    Note,
    Error,
    Warning,
    Help,
}

/// Source code location span, representing a range of source code and its associated text information.
pub struct LintSpan {
    /// Starting line number (1-based)
    pub line_start: usize,
    /// Ending line number (1-based)
    pub line_end: usize,
    /// Starting column (1-based char offset)
    pub column_start: usize,
    /// Ending column (1-based char offset)
    pub column_end: usize,
    /// Source lines at this location
    pub text: Vec<LintSpanLine>,
    /// Optional label description
    pub label: Option<String>,
}

/// A single line of text in a source location with highlight range.
pub struct LintSpanLine {
    /// Full source line text (no trailing `\n`)
    pub text: String,
    /// Highlight start (1-based char offset)
    pub highlight_start: usize,
    /// Highlight end (1-based char offset)
    pub highlight_end: usize,
}

/// A suggestion shown as a diff in the output (e.g. `- old code` / `+ new code`).
#[derive(Clone, Debug, Default)]
pub struct LintSuggestion {
    /// Source text that the suggestion applies to (a single line or snippet)
    pub source: String,
    /// Line number where the suggestion applies
    pub line_start: usize,
    /// Byte range within `source` to replace
    pub byte_range: Range<usize>,
    /// Replacement text
    pub replacement: String,
}

impl MlintReport {
    /// Build a `LintSpan` from a syn spanned item and the source text.
    pub fn span_from_syn<T: syn::spanned::Spanned>(value: &T, source: &str) -> LintSpan {
        let span = value.span();
        let start = span.start();
        let end = span.end();

        // Extract line content from source
        let lines: Vec<&str> = source.lines().collect();
        let text = if start.line == end.line && start.line <= lines.len() {
            let line_text = lines[start.line.saturating_sub(1)];
            let hl_start = proc_macro2_byte_col_to_char_1based(line_text, start.column);
            let hl_end = proc_macro2_byte_col_to_char_1based(line_text, end.column);
            vec![LintSpanLine {
                text: line_text.to_string(),
                highlight_start: hl_start,
                highlight_end: hl_end,
            }]
        } else {
            // Multi-line: generate line by line
            (start.line..=end.line.min(lines.len()))
                .map(|i| {
                    let line_text = lines[i.saturating_sub(1)];
                    let (hl_start, hl_end) = if i == start.line {
                        (
                            proc_macro2_byte_col_to_char_1based(line_text, start.column),
                            line_text.chars().count(),
                        )
                    } else if i == end.line {
                        (
                            1,
                            proc_macro2_byte_col_to_char_1based(line_text, end.column),
                        )
                    } else {
                        (1, line_text.chars().count())
                    };
                    LintSpanLine {
                        text: line_text.to_string(),
                        highlight_start: hl_start,
                        highlight_end: hl_end,
                    }
                })
                .collect::<Vec<_>>()
        };

        LintSpan {
            line_start: start.line,
            line_end: end.line,
            column_start: proc_macro2_byte_col_to_char_1based(
                lines.get(start.line.saturating_sub(1)).unwrap_or(&""),
                start.column,
            ),
            column_end: proc_macro2_byte_col_to_char_1based(
                lines.get(end.line.saturating_sub(1)).unwrap_or(&""),
                end.column,
            ),
            text,
            label: None,
        }
    }

    /// Compute byte offset from (line, column) within source.
    /// line: 1-based, column: 1-based char offset.
    pub fn line_col_to_byte_offset(&self, line: usize, col: usize) -> usize {
        let mut byte_pos = 0usize;
        for (i, line_str) in self.source_code.lines().enumerate() {
            if i + 1 == line {
                return byte_pos + char_1based_to_byte_offset(line_str, col);
            }
            byte_pos += line_str.len() + 1; // +1 for \n
        }
        self.source_code.len()
    }
}

/// proc-macro2's LineColumn.column is **0-based byte offset**.
/// Convert to 1-based char offset.
fn proc_macro2_byte_col_to_char_1based(line: &str, byte_col: usize) -> usize {
    line.char_indices()
        .position(|(i, _)| i >= byte_col)
        .map(|pos| pos + 1) // → 1-based
        .unwrap_or(line.chars().count().max(1))
}

/// 1-based char offset → byte offset within a string
fn char_1based_to_byte_offset(s: &str, char_1based: usize) -> usize {
    s.char_indices()
        .nth(char_1based.saturating_sub(1))
        .map(|(i, _)| i)
        .unwrap_or(s.len())
}

impl MlintReport {
    pub fn to_annotate_snippet_render(&self) -> String {
        let level = mlinit_level_to_annotate(&self.level);
        let title = level.clone().primary_title(&self.message);
        // code 不放在 title 里，改放在 note 中
        let mut group = Group::with_title(title);

        for span in &self.spans {
            let source = span
                .text
                .iter()
                .map(|l| l.text.as_str())
                .collect::<Vec<_>>()
                .join("\n");

            let byte_range = if span.text.len() == 1 {
                let line = &span.text[0];
                let start = char_1based_to_byte_offset(&line.text, line.highlight_start);
                let end = char_1based_to_byte_offset(&line.text, line.highlight_end);
                start..end
            } else {
                let first = &span.text[0];
                let last = span.text.last().unwrap();
                let start = char_1based_to_byte_offset(&first.text, first.highlight_start);
                let prefix_len: usize = span.text[..span.text.len() - 1]
                    .iter()
                    .map(|l| l.text.len() + 1)
                    .sum();
                let end = prefix_len + char_1based_to_byte_offset(&last.text, last.highlight_end);
                start..end
            };

            let mut snippet = Snippet::source(source)
                .line_start(span.line_start)
                .path(&self.file_name);

            let annotation = match &span.label {
                Some(label) => AnnotationKind::Primary
                    .span(byte_range)
                    .label(label.as_str()),
                None => AnnotationKind::Primary.span(byte_range),
            };
            snippet = snippet.annotation(annotation);
            group = group.element(snippet);
        }

        for child in &self.attached_reports {
            let child_level = mlinit_level_to_annotate(&child.level);
            let msg = child_level.clone().message(&child.message);
            group = group.element(msg);
        }

        // Render suggestions as diffs
        for sugg in &self.suggestions {
            let patch_snippet = Snippet::source(sugg.source.clone())
                .line_start(sugg.line_start)
                .path(&self.file_name)
                .patch(Patch::new(
                    sugg.byte_range.clone(),
                    sugg.replacement.clone(),
                ));
            group = group.element(patch_snippet);
        }

        if !self.lint_code.is_empty() {
            let level_name = match self.level {
                MlintLevel::Error => "deny",
                MlintLevel::Warning => "warn",
                MlintLevel::Note | MlintLevel::Help => "allow",
            };
            let note_text = format!("`#[mlint({level_name}({}))]` on by default", self.lint_code);
            let note_msg = NOTE.clone().message(note_text);
            group = group.element(note_msg);
        }

        let renderer = Renderer::styled();
        renderer.render(&[group])
    }
}

fn mlinit_level_to_annotate(level: &MlintLevel) -> &'static annotate_snippets::Level<'static> {
    match level {
        MlintLevel::Error => &ERROR,
        MlintLevel::Warning => &WARNING,
        MlintLevel::Note => &NOTE,
        MlintLevel::Help => &HELP,
    }
}

impl MlintReport {
    pub fn to_compiler_message(&self) -> Message {
        use cargo_metadata::{CompilerMessageBuilder, Edition, TargetBuilder};

        let target_kind_str = self.target_kind.as_deref().unwrap_or("bin");
        let target_kind_parsed: cargo_metadata::TargetKind = target_kind_str.into();
        let crate_kind: cargo_metadata::CrateType = target_kind_str.into();

        let target = TargetBuilder::default()
            .name(self.target_name.as_deref().unwrap_or_default())
            .kind(vec![target_kind_parsed])
            .crate_types(vec![crate_kind])
            .required_features(Vec::<String>::new())
            .src_path(self.target_src_path.as_deref().unwrap_or_default())
            .edition(Edition::E2021)
            .doctest(false)
            .test(false)
            .doc(false)
            .build()
            .unwrap();

        let diagnostic = self.build_diagnostic(
            &self.message,
            &self.lint_code,
            &self.level,
            &self.spans,
            &self.attached_reports,
        );

        Message::CompilerMessage(
            CompilerMessageBuilder::default()
                .package_id(PackageId {
                    repr: self.package_id.clone().unwrap_or_else(|| "unknown".into()),
                })
                .target(target)
                .message(diagnostic)
                .build()
                .unwrap(),
        )
    }

    fn build_diagnostic(
        &self,
        message: &str,
        code: &str,
        level: &MlintLevel,
        spans: &[LintSpan],
        children: &[MlintReport],
    ) -> cargo_metadata::diagnostic::Diagnostic {
        cargo_metadata::diagnostic::DiagnosticBuilder::default()
            .message(message)
            .code(
                DiagnosticCodeBuilder::default()
                    .code(code)
                    .explanation(None)
                    .build()
                    .unwrap(),
            )
            .level(match level {
                MlintLevel::Error => CargoLevel::Error,
                MlintLevel::Warning => CargoLevel::Warning,
                MlintLevel::Note => CargoLevel::Note,
                MlintLevel::Help => CargoLevel::Help,
            })
            .spans(
                spans
                    .iter()
                    .map(|s| self.lint_span_to_diagnostic_span(s))
                    .collect::<Vec<_>>(),
            )
            .children(
                children
                    .iter()
                    .map(|c| {
                        self.build_diagnostic(
                            &c.message,
                            &c.lint_code,
                            &c.level,
                            &c.spans,
                            &c.attached_reports,
                        )
                    })
                    .collect::<Vec<_>>(),
            )
            .rendered(None)
            .build()
            .unwrap()
    }

    fn lint_span_to_diagnostic_span(
        &self,
        span: &LintSpan,
    ) -> cargo_metadata::diagnostic::DiagnosticSpan {
        let byte_start = self.line_col_to_byte_offset(span.line_start, span.column_start);
        let byte_end = self.line_col_to_byte_offset(span.line_end, span.column_end);

        DiagnosticSpanBuilder::default()
            .file_name(&self.file_name)
            .byte_start(byte_start as u32)
            .byte_end(byte_end as u32)
            .line_start(span.line_start)
            .line_end(span.line_end)
            .column_start(span.column_start)
            .column_end(span.column_end)
            .is_primary(true)
            .text(
                span.text
                    .iter()
                    .map(|l| {
                        DiagnosticSpanLineBuilder::default()
                            .text(&l.text)
                            .highlight_start(l.highlight_start)
                            .highlight_end(l.highlight_end)
                            .build()
                            .unwrap()
                    })
                    .collect::<Vec<_>>(),
            )
            .label(span.label.clone())
            .suggested_replacement(self.suggestions.first().map(|s| s.replacement.clone()))
            .suggestion_applicability(if !self.suggestions.is_empty() {
                Some(cargo_metadata::diagnostic::Applicability::MachineApplicable)
            } else {
                None
            })
            .expansion(None)
            .build()
            .unwrap()
    }
}

#[derive(Grouped, Wrap)]
pub struct StateLintReports(pub Vec<MlintReport>);
#[derive(Grouped, Wrap)]
pub struct ResultLintReportsAnnotateSnippet(Vec<MlintReport>);
#[derive(Grouped, Wrap)]
pub struct ResultLintReportsJson(Vec<MlintReport>);

#[chain]
pub fn handle_state_lint_reports(reports: StateLintReports, using_json: &ResUsingJson) -> Next {
    if using_json.using {
        ResultLintReportsJson(reports.0).to_render()
    } else {
        ResultLintReportsAnnotateSnippet(reports.0).to_render()
    }
}

#[renderer(buffer)]
pub fn render_lint_reports(reports: ResultLintReportsAnnotateSnippet) {
    for report in reports.0 {
        r_eprintln!("{}", report.to_annotate_snippet_render());
    }
}

#[renderer(buffer)]
pub fn render_lint_reports_json(
    reports: ResultLintReportsJson,
    message_renderer: &RendererInvoker<Message>,
) {
    for report in reports.0 {
        let message = report.to_compiler_message();
        let result = message_renderer.invoke(message);
        r_append!(result);
    }
}
