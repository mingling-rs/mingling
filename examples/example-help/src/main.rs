//! Example Help
//!
//! > This example demonstrates how to use the `#[help]` macro to generate help information,
//! > enabling `--help` to work
//!
//! Run
//! ```bash
//! cargo run --manifest-path examples/example-help/Cargo.toml --quiet -- greet --help
//! ```
//!
//! Output:
//! ```plain
//! Usage: greet <NAME>
//! ```

use mingling::{macros::help, prelude::*, setup::BasicProgramSetup};
use std::io::Write;

dispatcher!("greet", EntryGreet);

// Define help        _________ When `program.user_context.help` is `true`
//                   /            the command will not enter `#[chain]` / `#[renderer]`
#[help] //           vvvvvvvvvv   but instead enter this `#[help]` function
fn help_greet(_prev: EntryGreet) -> RenderResult {
    let mut render_result = RenderResult::new();
    writeln!(render_result, "Usage: greet <NAME>").ok();
    render_result
}

fn main() {
    let mut program = ThisProgram::new();

    // --------- IMPORTANT ---------
    // Add `BasicProgramSetup` to the program
    // to enable `--help`, `--quiet`, and other built-in features
    program.with_setup(BasicProgramSetup);
    // --------- IMPORTANT ---------

    program.exec_and_exit();
}

gen_program!();
