//! Example Completion
//!
//! > This example demonstrates how to use **Mingling** to create fully dynamic command-line completions
//!
//! ## About Completion Scripts
//!
//! To make your completions work, you need to generate a completion script using Mingling's tools
//!
//! 1. Enable features
//!    You need to enable the `build` and `comp` features for `mingling` in `[build-dependencies]`
//!
//! 2. Write `build.rs`
//!    Write the following in `build.rs`
//!
//! ```rust,ignore
//! fn main() {
//!     build_scripts();
//! }
//!
//! /// Generate completion scripts
//! fn build_scripts() {
//!     // `env!("CARGO_PKG_NAME")` equals the crate name, which matches the binary name.
//!     // If your binary name differs from the crate name, specify it explicitly.
//!     mingling::build::build_comp_scripts(
//!         // Your binary name:
//!         env!("CARGO_PKG_NAME"),
//!     )
//!     .unwrap();
//! }
//! ```
//!
//! 3. Verify
//!    Build your project with `cargo build --release`. The completion scripts will be generated in `target/release/`
//!
//!    Execute the script or have it be automatically sourced by your Shell
//!
//! Run:
//! ```bash
//! cargo run --manifest-path examples/example-completion/Cargo.toml --quiet -- greet Alice --repeat 3
//! ```
//!
//! Output:
//! ```plaintext
//! Hello, Alice, Alice, Alice!
//! ```

use mingling::{macros::suggest, prelude::*, ShellContext, Suggest};
use std::io::Write;

fn main() {
    let mut program = ThisProgram::new();

    program.with_dispatcher(CMDGreet);

    // --------- IMPORTANT ---------
    // The `comp` feature makes `gen_program!()` generate a CMDCompletion automatically
    // It adds a hidden `__comp` subcommand for communication with the completion script
    program.with_dispatcher(crate::CMDCompletion);
    // --------- IMPORTANT ---------

    // TIP: Note that the completion script reads stdout,
    // so make sure no output is produced before the CMDCompletion is dispatched.
    program.exec_and_exit();
}

// --------- IMPORTANT ---------
//            __________________________________________ Entry point bound to completion behavior
//           /                 _________________________ Shell context for obtaining user input state
//           |                /                 ________ Suggest, used to return completion results
//           vvvvvvvvvv       |                /
#[completion(EntryGreet)] //  vvvvvvvvvvvv     vvvvvvv
fn complete_greet_entry(ctx: &ShellContext) -> Suggest {
    // When the previous word is `greet` (the current command being typed)
    if ctx.previous_word == "greet" {
        // Return suggestions
        return suggest! {
            "Bob": "Likes to pass messages",
            "Alice": "Likes to receive messages",
            "Hacker": "YOU",
            "World"
        };
    }

    // When the user is typing `--repeat`
    if ctx.filling_argument(["-r", "--repeat"]) {
        return suggest! {}; // Don't suggest anything
    }

    // When the user is typing `-`
    if ctx.typing_argument() {
        return suggest! {
            "-r": "Number of repetitions",
            "--repeat": "Number of repetitions",
        }
        // Remove arguments that have already been typed by the user
        .strip_typed_argument(ctx);
    }

    // Otherwise, suggest nothing
    suggest!()
    // // You can also enable file completions using the following code,
    // // which will invoke the Shell's default behavior
    // Suggest::file_comp()
}
// --------- IMPORTANT ---------

dispatcher!("greet", CMDGreet => EntryGreet);
pack!(ResultName = (u8, String));

#[chain]
fn handle_greet(args: EntryGreet) -> Next {
    let result: ResultName = args
        .pick_or(["-r", "--repeat"], 1)
        .pick_or((), "World")
        .unpack()
        .into();
    result.into()
}

/// Renders the greeting with the result name and repeat count.
#[renderer]
fn render_name(result: ResultName) -> RenderResult {
    let (repeat, name) = result.inner;
    let mut render_result = RenderResult::new();
    let mut parts = Vec::with_capacity(repeat as usize);
    for _ in 0..repeat {
        parts.push(name.clone());
    }
    writeln!(render_result, "Hello, {}!", parts.join(", ")).ok();
    render_result
}

gen_program!();
