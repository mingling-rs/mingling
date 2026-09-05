//! Example: Using the `import_type!()` Macro to Register Outside Types
//!
//! This example demonstrates how to use the `import_type!()` macro to make outside
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

use mingling::{macros::import_type, prelude::*};
use std::io::Write;
use std::{io::ErrorKind::Other, num::ParseIntError};

dispatcher!("parse");
dispatcher!("error");

#[chain]
fn handle_entry_error(_args: EntryError) -> Next {
    std::io::Error::new(Other, "Error").to_render()
}

// --------- IMPORTANT ---------
// You can directly use the `import_type!` macro to define outside types as types
// recognizable by Mingling
//      _____________ from std::num::ParseIntError
//     /
//     vvvvvvvvvvvvv
import_type!(std::num::ParseIntError);
import_type!(ErrorIo = std::io::Error);
//     ^^^^^^^^^^^^^^^^^^^^^^^^
//     \_____________ For types whose names may cause ambiguity,
//                      you can use this syntax to create an alias simultaneously
// --------- IMPORTANT ---------

#[derive(Grouped, Wrap)]
pub struct ParsedNumber(i32);

/// Parse the first argument as an `i32`
///
/// On success, routes to `render_number`.
/// On failure, routes to `render_parse_error` via the registered outside type.
#[chain]
fn parse_number(args: EntryParse) -> Next {
    let input = args.0.first().cloned().unwrap_or_default();
    match input.parse::<i32>() {
        Ok(num) => ParsedNumber(num).to_chain(),
        Err(e) => e.to_chain(),
    }
}

/// Renderer for successful parse — displays the parsed integer.
//                     _____________ Using std::num::ParseIntError as a chain input
//                    /
#[renderer] //        vvvvvvvvvvvv
fn render_number(num: ParsedNumber) -> RenderResult {
    let mut render_result = RenderResult::new();
    write!(render_result, "Parsed number: {}", *num).ok();
    render_result
}

/// Renderer for parse errors — using the outside `ParseIntError` type.
///
/// The `ParseIntError` type is registered via `import_type!` above, so it implements
/// `Grouped<ThisProgram>` and can be used directly in a `#[renderer]` function.
#[renderer]
fn render_parse_error(err: ParseIntError) -> RenderResult {
    let mut render_result = RenderResult::new();
    write!(render_result, "Parse error: {}", err).ok();
    render_result
}

/// Renderer for IO errors — using `std::io::Error` registered as `ErrorIo`.
//                       ________ Must use alias `ErrorIo` here, not bare `std::io::Error`
//                      /
#[renderer] //          vvvvvvv
fn render_error_io(err: ErrorIo) -> RenderResult {
    let mut render_result = RenderResult::new();
    write!(render_result, "IO_ERROR: {}", err).ok();
    render_result
}

fn main() {
    ThisProgram::new().exec_and_exit();
}

gen_program!();
