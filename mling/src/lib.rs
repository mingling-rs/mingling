#![allow(unused_imports)]

use mingling::{
    macros::{chain, gen_program, pack, r_println, renderer},
    res::ResExitCode,
};

pub mod cli;
pub use cli::*;

mod cargo_style;
pub use cargo_style::*;
pub mod display;
pub mod res;

mod pkg_mgr;
pub use pkg_mgr::*;

mod proj_mgr;
pub use proj_mgr::*;

mod errors;
pub use errors::*;

use crate::display::markdown;

gen_program!();
