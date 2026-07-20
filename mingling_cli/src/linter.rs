use mingling::{Program, macros::program_setup};

use crate::linter::cmd_mlint::CMDMinglingLinter;

pub mod cmd_mlint;

#[program_setup]
pub fn mingling_linter_setup(program: &mut Program<crate::ThisProgram>) {
    program.with_setup(MinglingLinterCommandSetup);
}

#[program_setup]
pub fn mingling_linter_command_setup(program: &mut Program<crate::ThisProgram>) {
    program.with_dispatcher(CMDMinglingLinter);
}
