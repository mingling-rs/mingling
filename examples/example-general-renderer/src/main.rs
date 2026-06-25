//! Example General Renderer
//!
//! > This example demonstrates how to use the `general_renderer` feature to render data into structures such as json / yaml
//!
//! Run
//! ```bash
//! cargo run --manifest-path examples/example-general-renderer/Cargo.toml --quiet -- render Bob 22
//! cargo run --manifest-path examples/example-general-renderer/Cargo.toml --quiet -- render Bob 22 --json
//! cargo run --manifest-path examples/example-general-renderer/Cargo.toml --quiet -- render Bob 22 --yaml
//! ```
//!
//! Output:
//! ```plain
//! Bob is 22 years old
//! {"member_name":"Bob","member_age":22}
//! member_name: Bob
//! member_age: 22
//! ```

use mingling::prelude::*;
use mingling::{parser::Picker, setup::GeneralRendererSetup, StructuralData, Groupped};
use serde::Serialize;

dispatcher!("render", CMDRender => EntryRender);

fn main() {
    let mut program = ThisProgram::new();
    // Add `GeneralRendererSetup` to receive user input `--json` `--yaml` parameters
    program.with_setup(GeneralRendererSetup);
    program.with_dispatcher(CMDRender);
    let _ = program.exec();
}

// --------- IMPORTANT ---------
// For beautiful output structure, do not use `pack!` to wrap the types that need to be output.
// Instead, manually implement
//        __________________________________ Mark as structured data so it can be rendered
//       /              ____________________ Implement serde::Serialize
//       |             /           _________ Implement mingling::Groupped
//       |             |          /            to ensure Mingling can recognize the type
//       vvvvvvvvvvvv  vvvvvvvvv  vvvvvvvv
#[derive(StructuralData, Serialize, Groupped)]
struct Info {
    #[serde(rename = "member_name")]
    name: String,
    #[serde(rename = "member_age")]
    age: i32,
}
// This will output: {"member_name":"name","member_age":32} structure

// If using pack!(Info = (String, i32));
// Output: {"inner":["name", 32]}

// --------- IMPORTANT ---------

#[chain]
fn parse_render(prev: EntryRender) -> Next {
    let (name, age) = Picker::new(prev.inner)
        .pick::<String>(())
        .pick::<i32>(())
        .unpack();
    Info { name, age }.to_render()
}

/// Implement default renderer for when general_renderer is not specified
#[renderer]
fn render_info(prev: Info) {
    r_println!("{} is {} years old", prev.name, prev.age);
}

gen_program!();
