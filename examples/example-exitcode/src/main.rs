//! Example Error Handling
//!
//! > This example demonstrates how to handle errors in Mingling, including custom error types and error rendering.
//!
//! Run:
//! ```bash
//! cargo run --manifest-path examples/example-exitcode/Cargo.toml --quiet -- hello Alice
//! cargo run --manifest-path examples/example-exitcode/Cargo.toml --quiet -- hello
//! ```
//!
//! Output:
//! ```plaintext
//! Hello, Alice
//! No name provided (with exit code 1)
//! ```

use mingling::{
    macros::help,
    prelude::*,
    res::ResExitCode,
    setup::{BasicProgramSetup, ExitCodeSetup},
};
use std::io::Write;

fn main() {
    let mut program = ThisProgram::new();
    program.with_setup(BasicProgramSetup);

    // --------- IMPORTANT ---------
    // Register `ExitCodeSetup` for the program to enable exit codes
    program.with_setup(ExitCodeSetup::default());
    // --------- IMPORTANT ---------

    program.exec_and_exit();
}

dispatcher!("hello", EntryHello);

#[derive(Grouped)]
pub struct ErrorNoNameProvided;

#[derive(Grouped, Wrap)]
pub struct ResultName(String);

#[chain]
fn handle_hello(args: EntryHello) -> Next {
    let Some(name) = args.0.first().cloned() else {
        // If no name is provided, pass ErrorNoNameProvided
        return ErrorNoNameProvided.to_render();
    };

    // If the name is valid, pass ResultName
    ResultName(name).to_render()
}

/// Renders a successful greeting with the given name.
#[renderer]
fn render_result_name(name: ResultName) -> RenderResult {
    let mut result = RenderResult::new();
    writeln!(result, "Hello, {}", *name).ok();
    result
}

#[help]
fn help_hello(_p: EntryHello, ec: &mut ResExitCode) -> RenderResult {
    let mut result = RenderResult::new();
    writeln!(result, "Usage: hello <NAME>").ok();
    ec.exit_code = 2;
    result
}

// Define renderer, render error message                      _______________ Inject exit code resource
//                                                           /
/// Renders the error when no name is provided               |
#[renderer] //                                               vvvvvvvvvvvvvvvv
fn render_error_no_name_provided(_: ErrorNoNameProvided, ec: &mut ResExitCode) -> RenderResult {
    ec.exit_code = 1;

    let mut result = RenderResult::new();

    // Prompt when no name is provided
    writeln!(result, "No name provided (with exit code 1)").ok();
    result
}

gen_program!();
