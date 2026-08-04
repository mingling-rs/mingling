use mingling::{
    macros::{chain, dispatcher, entry, metadata},
    metadata::Description,
};

use crate::{linter::cmd_mlint::EntryLint, metadata::setup::ResUsingJson};

pub mod cmd_mlint;
pub mod mlint_attr;
pub mod mlint_report;

// Aliases

dispatcher!("ra-lint-clippy",
    CMDLinterSupportRustAnalyzerWithClippy => EntryLinterSupportRustAnalyzerWithClippy
);

dispatcher!("ra-lint-check",
    CMDLinterSupportRustAnalyzerWithCheck => EntryLinterSupportRustAnalyzerWithCheck
);

dispatcher!("ra-lint",
    CMDLinterSupportRustAnalyzer => EntryLinterSupportRustAnalyzer
);

#[chain]
pub fn handle_ra_lint(_: EntryLinterSupportRustAnalyzer, use_json: &mut ResUsingJson) -> EntryLint {
    use_json.using = true;
    entry!("--message-format=json")
}

#[chain]
pub fn handle_ra_lint_check(
    _: EntryLinterSupportRustAnalyzerWithCheck,
    use_json: &mut ResUsingJson,
) -> EntryLint {
    use_json.using = true;
    entry!("--message-format=json", "--with-checker=cargo,check")
}

#[chain]
pub fn handle_ra_lint_clippy(
    _: EntryLinterSupportRustAnalyzerWithClippy,
    use_json: &mut ResUsingJson,
) -> EntryLint {
    use_json.using = true;
    entry!("--message-format=json", "--with-checker=cargo,clippy")
}

#[metadata(EntryLinterSupportRustAnalyzer)]
pub fn desc_ra_lint() -> Description {
    "Run `mling lint` and output the results".to_string().into()
}

#[metadata(EntryLinterSupportRustAnalyzerWithCheck)]
pub fn desc_ra_lint_check() -> Description {
    "Run `mling lint` and `cargo check`, and output the combined results"
        .to_string()
        .into()
}

#[metadata(EntryLinterSupportRustAnalyzerWithClippy)]
pub fn desc_ra_lint_clippy() -> Description {
    "Run `mling lint` and `cargo clippy`, and output the combined results"
        .to_string()
        .into()
}
