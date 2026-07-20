use mingling::{Program, macros::program_setup};

use crate::metadata::{cmd_metadata::CMDMetadata, setup::CargoMetadataSetup};

pub mod cmd_metadata;
pub mod setup;

#[program_setup]
pub fn mingling_metadata_setup(program: &mut Program<crate::ThisProgram>) {
    program.with_setup(CargoMetadataSetup);
    program.with_dispatcher(CMDMetadata);
}
