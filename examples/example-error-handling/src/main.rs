//! Example Error Handling
//!
//! > This example demonstrates how to handle errors in Mingling, including custom error types and error rendering.
//!
//! Run:
//! ```bash
//! cargo run --manifest-path examples/example-error-handling/Cargo.toml --quiet -- hallo
//! cargo run --manifest-path examples/example-error-handling/Cargo.toml --quiet -- hello
//! cargo run --manifest-path examples/example-error-handling/Cargo.toml --quiet -- hello Alice
//! cargo run --manifest-path examples/example-error-handling/Cargo.toml --quiet -- hello MyBestFriendAlice
//! cargo run --manifest-path examples/example-error-handling/Cargo.toml --quiet -- hello Peter
//! ```
//!
//! Output:
//! ```plaintext
//! Command not found: "hallo"
//! No name provided
//! Name not available
//! Name too long: 17 > 10
//! Hello, Peter
//! ```

use mingling::prelude::*;
use std::io::Write;

// In Mingling, instead of using ? to propagate errors upward,
// errors are treated as branches that continue execution.

dispatcher!("hello", EntryHello);

// Define error types
#[derive(Grouped)]
pub struct ErrorNoNameProvided;

#[derive(Grouped, Wrap)]
pub struct ErrorNameTooLong(u16);

#[derive(Grouped)]
pub struct ErrorNameNotAvailable;

// Define success type
#[derive(Grouped, Wrap)]
pub struct ResultName(String);

// Pre-registered names
static VEC_REGISTERED_NAMES: &[&str] = &["Alice", "Bob", "Charlie", "David", "Eve"];

#[chain]
fn handle_hello(args: EntryHello) -> Next {
    let Some(name) = args.0.first().cloned() else {
        // If no name is provided, pass ErrorNoNameProvided
        return ErrorNoNameProvided.to_render();
    };

    if name.len() > 10 {
        // If the name is too long, pass ErrorNameTooLong
        return ErrorNameTooLong(name.len() as u16).to_render();
    }

    if VEC_REGISTERED_NAMES.contains(&name.as_str()) {
        // If the name already exists, pass ErrorNameNotAvailable
        return ErrorNameNotAvailable.to_render();
    }

    // If the name is valid, pass ResultName
    ResultName(name).to_render()
}

/// Renders a successful greeting with the given name.
#[renderer]
fn render_result_name(name: ResultName) -> RenderResult {
    let mut render_result = RenderResult::new();
    writeln!(render_result, "Hello, {}", *name).ok();
    render_result
}

/// Renders the error when no name is provided.
#[renderer]
fn render_error_no_name_provided(_: ErrorNoNameProvided) -> RenderResult {
    let mut render_result = RenderResult::new();
    writeln!(render_result, "No name provided").ok();
    render_result
}

/// Renders the error when the name is already taken.
#[renderer]
fn render_error_name_not_available(_: ErrorNameNotAvailable) -> RenderResult {
    let mut render_result = RenderResult::new();
    writeln!(render_result, "Name not available").ok();
    render_result
}

/// Renders the error when the name exceeds the maximum length.
#[renderer]
fn render_error_name_too_long(len: ErrorNameTooLong) -> RenderResult {
    let mut render_result = RenderResult::new();
    writeln!(render_result, "Name too long: {} > 10", *len).ok();
    render_result
}

/// Renders the error when the dispatcher (subcommand) is not found.
#[renderer]
fn render_entry_fallback(err: EntryFallback) -> RenderResult {
    let mut render_result = RenderResult::new();
    writeln!(render_result, "Command not found: \"{}\"", err.0.join(" ")).ok();
    render_result
}

gen_program!();

fn main() {
    ThisProgram::new().exec_and_exit();
}
