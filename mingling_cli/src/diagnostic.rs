use annotate_snippets::level::{ERROR, HELP, NOTE, WARNING};
use annotate_snippets::{AnnotationKind, Group, Renderer, Snippet};
use cargo_metadata::diagnostic::Diagnostic;
use mingling::macros::{buffer, import_type, r_println, renderer};

import_type!(cargo_metadata::diagnostic::Diagnostic);

#[renderer(buffer)]
pub fn render_diagnostic(diagnostic: Diagnostic) {
    let report = diagnostic_to_report(&diagnostic);
    let renderer = Renderer::styled();
    let rendered = renderer.render(&report);
    r_println!("{rendered}");
}

fn cargo_level_to_annotate(
    level: cargo_metadata::diagnostic::DiagnosticLevel,
) -> &'static annotate_snippets::Level<'static> {
    use cargo_metadata::diagnostic::DiagnosticLevel;
    match level {
        DiagnosticLevel::Ice | DiagnosticLevel::Error => &ERROR,
        DiagnosticLevel::Warning => &WARNING,
        DiagnosticLevel::Note | DiagnosticLevel::FailureNote => &NOTE,
        DiagnosticLevel::Help => &HELP,
        _ => &ERROR,
    }
}

/// Convert 1-based char offset to 0-based byte offset
fn char_offset_to_byte_offset(s: &str, char_offset: usize) -> usize {
    s.char_indices()
        .nth(char_offset.saturating_sub(1))
        .map(|(i, _)| i)
        .unwrap_or(s.len())
}

fn diagnostic_to_report<'a>(d: &'a Diagnostic) -> Vec<Group<'a>> {
    let level = cargo_level_to_annotate(d.level);
    let mut title = level.clone().primary_title(d.message.as_str());
    if let Some(ref code) = d.code {
        title = title.id(code.code.as_str());
    }

    let mut group = Group::with_title(title);

    for span in &d.spans {
        if !span.is_primary {
            continue;
        }

        let source: String = span
            .text
            .iter()
            .map(|l| l.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        let byte_range = if span.text.len() == 1 {
            let line = &span.text[0];
            let start = char_offset_to_byte_offset(&line.text, line.highlight_start);
            let end = char_offset_to_byte_offset(&line.text, line.highlight_end);
            start..end
        } else {
            let first = &span.text[0];
            let last = span.text.last().unwrap();
            let start = char_offset_to_byte_offset(&first.text, first.highlight_start);
            let prefix_len: usize = span.text[..span.text.len() - 1]
                .iter()
                .map(|l| l.text.len() + 1)
                .sum();
            let end = prefix_len + char_offset_to_byte_offset(&last.text, last.highlight_end);
            start..end
        };

        let mut snippet = Snippet::source(source)
            .line_start(span.line_start)
            .path(span.file_name.as_str());

        if let Some(ref label) = span.label {
            let annotation = AnnotationKind::Primary
                .span(byte_range)
                .label(label.as_str());
            snippet = snippet.annotation(annotation);
        } else {
            let annotation = AnnotationKind::Primary.span(byte_range);
            snippet = snippet.annotation(annotation);
        }

        group = group.element(snippet);
    }

    for child in &d.children {
        let msg_level = cargo_level_to_annotate(child.level);
        let msg = msg_level.clone().message(child.message.as_str());
        group = group.element(msg);
    }

    vec![group]
}
