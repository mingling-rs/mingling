#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]
#![allow(clippy::redundant_pub_crate)]
#![allow(clippy::missing_const_for_fn)]

use mingling::macros::{gen_program, help};

pub(crate) mod cmd;
pub(crate) mod task;

/// Mingling CI's Resources
pub mod res;

/// Log exporter for CI reports
pub mod reporter;

pub(crate) mod examples;
pub(crate) mod markdown;
pub(crate) mod progress;
pub(crate) mod tools;

#[help]
pub fn render_fallback(_: EntryFallback) -> String {
    include_str!("../help.txt").to_string()
}

gen_program!();
