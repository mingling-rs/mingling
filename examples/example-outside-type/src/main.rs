//! Example: Using the `group!()` Macro to Register Outside Types
//!
//! This example demonstrates how to use the `group!()` macro to make outside
//! types (from `std` or other crates) recognizable by the Mingling framework,
//! without modifying the original type definition.
//!
//! Run:
//! ```bash
//! cargo run --manifest-path examples/example-outside-type/Cargo.toml --quiet -- parse 42
//! cargo run --manifest-path examples/example-outside-type/Cargo.toml --quiet -- parse hello
//! cargo run --manifest-path examples/example-outside-type/Cargo.toml --quiet -- error
//! ```
//!
//! Output:
//! ```plaintext
//! Parsed number: 42
//! Parse error: invalid digit found in string
//! IO_ERROR: Error
//! ```

use mingling::{macros::group, prelude::*};
use std::{io::ErrorKind::Other, num::ParseIntError};

dispatcher!("parse");
dispatcher!("error");

#[chain]
fn handle_entry_error(_args: EntryError) -> Next {
    std::io::Error::new(Other, "Error").to_render()
}

// --------- IMPORTANT ---------
// You can directly use the `group!` macro to define outside types as types
// recognizable by Mingling
//      _____________ from std::num::ParseIntError
//     /
//     vvvvvvvvvvvvv
group!(ParseIntError);
group!(ErrorIo = std::io::Error);
//     ^^^^^^^^^^^^^^^^^^^^^^^^
//     \_____________ For types whose names may cause ambiguity,
//                      you can use this syntax to create an alias simultaneously
// --------- IMPORTANT ---------

pack!(ParsedNumber = i32);

/// Parse the first argument as an `i32`
///
/// On success, routes to `render_number`.
/// On failure, routes to `render_parse_error` via the registered outside type.
#[chain]
fn parse_number(args: EntryParse) -> Next {
    let input = args.inner.first().cloned().unwrap_or_default();
    match input.parse::<i32>() {
        Ok(num) => ParsedNumber::new(num).to_chain(),
        Err(e) => e.to_chain(),
    }
}

/// Renderer for successful parse — displays the parsed integer.
//                     _____________ Using std::num::ParseIntError as a chain input
//                    /
#[renderer] //        vvvvvvvvvvvv
fn render_number(num: ParsedNumber) {
    r_println!("Parsed number: {}", *num);
}

/// Renderer for parse errors — using the outside `ParseIntError` type.
///
/// The `ParseIntError` type is registered via `group!` above, so it implements
/// `Groupped<ThisProgram>` and can be used directly in a `#[renderer]` function.
#[renderer]
fn render_parse_error(err: ParseIntError) {
    r_println!("Parse error: {}", err);
}

/// Renderer for IO errors — using `std::io::Error` registered as `ErrorIo`.
//                       ________ Must use alias `ErrorIo` here, not bare `std::io::Error`
//                      /
#[renderer] //          vvvvvvv
fn render_error_io(err: ErrorIo) {
    r_println!("IO_ERROR: {}", err.to_string());
}

fn main() {
    let mut program = ThisProgram::new();
    program.with_dispatcher(CMDParse);
    program.with_dispatcher(CMDError);
    program.exec_and_exit();
}

gen_program!();
