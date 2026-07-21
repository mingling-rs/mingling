use mingling::{
    Program,
    macros::{chain, dispatcher, entry, program_setup},
};

use crate::linter::cmd_mlint::{CMDMinglingLinter, EntryMinglingLinter};

pub mod cmd_mlint;
pub mod mlint_attr;
pub mod mlint_report;

#[program_setup]
pub fn mingling_linter_setup(program: &mut Program<crate::ThisProgram>) {
    program.with_setup(MinglingLinterCommandSetup);
}

#[program_setup]
pub fn mingling_linter_command_setup(program: &mut Program<crate::ThisProgram>) {
    program.with_dispatcher(CMDMinglingLinter);
    program.with_dispatcher(CMDLinterSupportRustAnalyzer);
    program.with_dispatcher(CMDLinterSupportRustAnalyzerWithClippy);
    program.with_dispatcher(CMDLinterSupportRustAnalyzerWithCheck);
}

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
pub fn handle_ra_lint(_: EntryLinterSupportRustAnalyzer) -> EntryMinglingLinter {
    entry!("--message-format=json")
}

#[chain]
pub fn handle_ra_lint_check(_: EntryLinterSupportRustAnalyzerWithCheck) -> EntryMinglingLinter {
    entry!("--message-format=json", "--with-checker=cargo,check")
}

#[chain]
pub fn handle_ra_lint_clippy(_: EntryLinterSupportRustAnalyzerWithClippy) -> EntryMinglingLinter {
    entry!("--message-format=json", "--with-checker=cargo,clippy")
}
