#![allow(unused_imports)]

use mingling::{
    macros::{chain, gen_program, pack, r_println, renderer},
    res::ResExitCode,
};

mod cargo_style;
pub use cargo_style::*;

pub mod cli;
pub mod display;
pub mod errors;
pub mod pkg_mgr;
pub mod proj_mgr;
pub mod res;

gen_program!();
