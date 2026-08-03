use cargo_metadata::Metadata;
use mingling::{
    Program,
    macros::{buffer, group, program_setup, r_println, renderer, renderify},
};

use crate::metadata::setup::CargoMetadataSetup;

pub mod cmd_metadata;
pub mod setup;

#[program_setup]
pub fn mingling_metadata_setup(program: &mut Program<crate::ThisProgram>) {
    program.with_setup(CargoMetadataSetup);
}

group!(Metadata);

#[renderer(buffer, renderify)]
pub fn render_metadata(metadata: Metadata) {
    let result = serde_json::to_string(&metadata)?;
    r_println!("{}", result);
}
