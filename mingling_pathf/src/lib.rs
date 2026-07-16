#![allow(clippy::needless_doctest_main)]
#![doc = include_str!("../README.md")]

pub mod config;
pub mod error;
pub mod module_pathf;
pub mod pattern_analyzer;
pub mod patterns;

mod type_mapping_builder;
pub use type_mapping_builder::analyze_and_build_type_mapping;
pub use type_mapping_builder::analyze_and_build_type_mapping_for;
