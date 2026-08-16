//! Example The Basic Usage of Mingling
//!
//! Run:
//! ```base
//! cargo run --manifest-path examples/example-basic/Cargo.toml --quiet -- greet
//! cargo run --manifest-path examples/example-basic/Cargo.toml --quiet -- greet Alice
//! ```
//!
//! Output:
//! ```plaintext
//! Hello, World!
//! Hello, Alice!
//! ```

// Import commonly used Mingling modules
use mingling::prelude::*;
use std::io::Write;

// Define the `greet` subcommand
//            _________________ subcmd name, can be nested (e.g. "remote.add" "remote.rm")
//           /
//           |        _________ entry, records raw arguments
//           |       /                         ^^^^^^^^^^^^^
//           vvvvv   vvvvvvvvvv                \_ a newtype wrapper around Vec<String>
dispatcher!("greet", EntryGreet);

fn main() {
    // Create a new ThisProgram
    let program = ThisProgram::new();

    // Run the program, then exit the process
    program.exec_and_exit();
}

// Quickly wrap a type into a type recognizable by the current program
//     ___________________  Registers this type into ThisProgram
//    /            _______  Adds DerefMut, Deref, Into, From wrappers
//    |           /
//    vvvvvvvvvv  vvvvv
#[derive(Grouped, Wrap)]
pub struct ResultName(String);

// Define the `handle_greet` chain for parsing input text
//                     ____________________ Previous type:
//                    /                       Mingling deduces types at runtime and routes them to this function
//                    |               _____ will be expanded to:
//                    |              /        ChainProcess<ThisProgram>
#[chain] //           vvvvvvvvvv     vvvv
fn handle_greet(args: EntryGreet) -> Next {
    let name: ResultName = args
        .0
        .first()
        .cloned()
        .unwrap_or_else(|| "World".to_string())
        .into();
    name.into()
}

// Define renderer `render_name`, used to render `ResultName`
/// Renders the greeting message with the provided name.
#[renderer]
fn render_name(name: ResultName) -> RenderResult {
    let mut render_result = RenderResult::new();
    writeln!(render_result, "Hello, {}!", *name).ok();
    render_result
}

// Note: This macro generates the program entry point.
// It must be placed at the end of the root module of the crate (>= mingling@0.1.8).
//                          ^^^^^^     ^^^^^^^^^^^
// For example: lib.rs, main.rs
gen_program!();
