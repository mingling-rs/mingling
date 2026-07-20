use crate::{linter::MinglingLinterSetup, metadata::MinglingMetadataSetup};
use mingling::{
    macros::gen_program,
    setup::{
        ExitCodeSetup,
        picker::{HelpFlagSetup, StructuralRendererSetup},
    },
};

pub mod linter;
pub mod metadata;

fn main() {
    let mut program = ThisProgram::new();

    // Setups
    program.with_setup(HelpFlagSetup::default());
    program.with_setup(ExitCodeSetup::default());
    program.with_setup(StructuralRendererSetup);

    program.with_setup(MinglingMetadataSetup);
    program.with_setup(MinglingLinterSetup);

    // Exec
    program.exec_and_exit();
}

gen_program!();
