use mingling::macros::gen_program;

pub mod diagnostic;
pub mod errors;
pub mod linter;
pub mod lints;
pub mod message;
pub mod metadata;
pub mod pkg_mgr;
pub mod proj_mgr;
pub mod utils;

gen_program!();
