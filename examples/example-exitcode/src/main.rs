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

fn main() {
    let mut program = ThisProgram::new();
    program.with_setup(BasicProgramSetup);

    // --------- IMPORTANT ---------
    // Register `ExitCodeSetup` for the program to enable exit codes
    program.with_setup(ExitCodeSetup::default());
    // --------- IMPORTANT ---------

    program.with_dispatcher(CMDHello);
    program.exec_and_exit();
}

dispatcher!("hello", CMDHello => EntryHello);

pack!(ErrorNoNameProvided = ());
pack!(ResultName = String);

#[chain]
fn handle_hello(args: EntryHello) -> Next {
    let Some(name) = args.inner.first().cloned() else {
        // If no name is provided, pass ErrorNoNameProvided
        return ErrorNoNameProvided::default().to_render();
    };

    // If the name is valid, pass ResultName
    ResultName::new(name).to_render()
}

/// Renders a successful greeting with the given name.
#[renderer]
fn render_result_name(name: ResultName) {
    r_println!("Hello, {}", *name);
}

#[help]
fn help_hello(_p: EntryHello, ec: &mut ResExitCode) {
    r_println!("Usage: hello <NAME>");
    ec.exit_code = 2;
}

// Define renderer, render error message                      _______________ Inject exit code resource
//                                                           /
/// Renders the error when no name is provided               |
#[renderer] //                                               vvvvvvvvvvvvvvvv
fn render_error_no_name_provided(_: ErrorNoNameProvided, ec: &mut ResExitCode) {
    ec.exit_code = 1;

    // Prompt when no name is provided
    r_println!("No name provided (with exit code 1)");
}

gen_program!();
