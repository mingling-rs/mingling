//! Example Clap Binding
//!
//! > This example demonstrates how to bind clap_derive to Mingling
//!
//! **Note**:
//! If the `error` parameter of the `dispatcher_clap!` macro is enabled, arguments will be parsed using `try_parse_from`.
//! If you need such output to support ANSI colors, enable the `color` feature of `clap`.
//!
//! Run:
//! ```bash
//! cargo run --manifest-path examples/example-clap-binding/Cargo.toml --quiet -- greet
//! cargo run --manifest-path examples/example-clap-binding/Cargo.toml --quiet -- greet Alice
//! cargo run --manifest-path examples/example-clap-binding/Cargo.toml --quiet -- greet Alice -r 5
//! cargo run --manifest-path examples/example-clap-binding/Cargo.toml --quiet -- greet --help
//! cargo run --manifest-path examples/example-clap-binding/Cargo.toml --quiet -- greet --rppat
//! ```
//!
//! Output:
//! ```plaintext
//! Hello, World!
//! Hello, Alice!
//! Hello, Alice, Alice, Alice, Alice, Alice!
//! Usage: example-clap-binding [OPTIONS] [NAME]
//!
//! Arguments:
//!   [NAME]  [default: World]
//!
//! Options:
//!   -r, --repeat <REPEAT>  [default: 1]
//!   -h, --help             Print help
//!
//! error: unexpected argument '--rppat' found
//!
//!   tip: a similar argument exists: '--repeat'
//!
//! Usage: example-clap-binding --repeat <REPEAT> [NAME]
//!
//! For more information, try '--help'.
//! ```

use mingling::{Grouped, macros::dispatcher_clap, prelude::*, setup::BasicProgramSetup};
use std::io::Write;

fn main() {
    let mut program = ThisProgram::new();

    // --------- IMPORTANT ---------
    // Introduce BasicProgramSetup to support ["--help", "-h"] options
    program.with_setup(BasicProgramSetup);

    // Set clap help output mode
    program.stdout_setting.clap_help_print_behaviour =
        mingling::config::ClapHelpPrintBehaviour::WriteToRenderResult;
    //  mingling::config::ClapHelpPrintBehaviour::PrintDirectly
    //
    // PrintDirectly:
    //   Let Clap print help information directly to stdout
    //
    // WriteToRenderResult:
    //   Capture Clap's help information and write to RenderResult
    // --------- IMPORTANT ---------

    program.exec_and_exit();
}

// Implement Clap Parser, and bind to Dispatcher
//        _______________________________ Default trait, provides fallback on parse failure
//       /         ______________________ clap::Parser, parsing logic implemented by Clap
//       |        /              ________ Implement mingling::Grouped
//       |        |             /           to ensure Mingling can recognize the type
//       vvvvvvv  vvvvvvvvvvvv  vvvvvvv
#[derive(Default, clap::Parser, Grouped)]
#[dispatcher_clap(
    "greet", CMDGreet,        // Bind EntryGreet to "greet" command
    help = true,              // Generate clap help for EntryGreet
    error = ErrorGreetParsed, // Generate and bind error type for parse failure
//  ^^^^^\__ Using `error` intercepts parse failure information into the specified type,
//              which is then rendered by the renderer
)]
pub struct EntryGreet {
    // Positional argument
    #[clap(default_value = "World")]
    name: String,

    // Option argument
    #[arg(short, long, default_value_t = 1)]
    repeat: i32,
}

/// Renders the greet output with optional repetition.
#[renderer]
fn render_greet(greet: EntryGreet) -> RenderResult {
    let name = greet.name;
    let count = greet.repeat.max(0) as usize;

    let mut render_result = RenderResult::default();
    write!(render_result, "Hello, ").ok();
    for i in 0..count {
        write!(render_result, "{name}").ok();
        if i < count - 1 {
            write!(render_result, ", ").ok();
        }
    }
    writeln!(render_result, "!").ok();
    render_result
}

/// Renders the error message when greet argument parsing fails.
#[renderer]
// renderers can return a RenderResult instead of using r_println!
pub fn render_greet_parse_failed(err: ErrorGreetParsed) -> RenderResult {
    let mut render_result = RenderResult::default();
    writeln!(render_result, "{}", *err).ok();
    render_result
}

gen_program!();
