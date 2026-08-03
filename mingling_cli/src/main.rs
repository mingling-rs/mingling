use crate::metadata::MinglingMetadataSetup;
use mingling::{
    ShellContext, Suggest,
    consts::HELP_FLAG,
    macros::{completion, gen_program, help, suggest},
    setup::{ExitCodeSetup, picker::HelpFlagSetup},
};

pub mod diagnostic;
pub mod errors;
pub mod linter;
pub mod lints;
pub mod message;
pub mod metadata;

#[tokio::main]
async fn main() {
    let mut program = ThisProgram::new();

    // Setups
    program.with_setup(HelpFlagSetup::default());
    program.with_setup(ExitCodeSetup::default());

    program.with_setup(MinglingMetadataSetup);

    // Exec
    program.exec_and_exit().await;
}

#[help]
pub fn help_global(_: EntryFallback) -> String {
    include_str!("../help/help.txt").to_string()
}

#[completion(EntryFallback)]
pub fn complete_global(_ctx: &ShellContext) -> Suggest {
    suggest! {
        HELP_FLAG: "Show help messages"
    }
}

gen_program!();
