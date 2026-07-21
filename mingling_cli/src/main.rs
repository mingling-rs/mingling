use crate::{linter::MinglingLinterSetup, metadata::MinglingMetadataSetup};
use mingling::{
    macros::gen_program,
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
    program.with_setup(MinglingLinterSetup);

    // Exec
    program.exec_and_exit().await;
}

gen_program!();
