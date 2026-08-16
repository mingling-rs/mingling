//! Example Unit Test
//!
//! > This example shows how to write unit tests for Chain and Renderer in Mingling
//!
//! ```bash
//! cargo test --manifest-path examples/example-unit-test/Cargo.toml
//! ```

use mingling::prelude::*;
use std::io::Write;

#[cfg(test)]
mod tests {
    use super::*;
    use mingling::macros::entry;
    use mingling::{assert_member_id, assert_render_result, unpack_chain_process};

    // --------- IMPORTANT ---------
    #[test]
    fn test_handle_hello() {
        let hello_without_args = handle_hello(entry!()).into();
        assert_render_result!(hello_without_args);
        assert_member_id!(hello_without_args, ThisProgram::ErrorNoNameProvided);

        let hello_with_registered_name = handle_hello(entry!("Alice")).into();
        assert_render_result!(hello_with_registered_name);
        assert_member_id!(
            hello_with_registered_name,
            ThisProgram::ErrorNameNotAvailable
        );

        let hello_with_long_name = handle_hello(entry!("It's a VeryLongName")).into();
        assert_render_result!(hello_with_long_name);
        assert_member_id!(hello_with_long_name, ThisProgram::ErrorNameTooLong);

        let hello_with_valid_name = handle_hello(entry!("Peter")).into();
        assert_render_result!(hello_with_valid_name);
        let result_name = unpack_chain_process!(hello_with_valid_name, ResultName);
        assert_eq!(result_name.0, "Peter");
    }

    #[test]
    fn test_render_result_name() {
        let r = render_result_name(ResultName("Peter".into()));
        assert_eq!(r.to_string().as_str(), "Hello, Peter!")
    }

    #[test]
    fn test_render_error_no_name_provided() {
        let r = render_error_no_name_provided(ErrorNoNameProvided);
        assert_eq!(r.to_string().as_str(), "No name provided")
    }

    #[test]
    fn test_render_error_name_not_available() {
        let r = render_error_name_not_available(ErrorNameNotAvailable);
        assert_eq!(r.to_string().as_str(), "Name not available")
    }

    #[test]
    fn test_render_error_name_too_long() {
        let r = render_error_name_too_long(ErrorNameTooLong(17));
        assert_eq!(r.to_string().as_str(), "Name too long: 17 > 10")
    }
    // --------- IMPORTANT ---------
}

dispatcher!("hello", EntryHello);

#[derive(Grouped)]
pub struct ErrorNoNameProvided;

#[derive(Grouped, Wrap)]
pub struct ErrorNameTooLong(u16);

#[derive(Grouped)]
pub struct ErrorNameNotAvailable;

#[derive(Grouped, Wrap)]
pub struct ResultName(String);

static VEC_REGISTERED_NAMES: &[&str] = &["Alice", "Bob", "Charlie", "David", "Eve"];

#[chain]
fn handle_hello(args: EntryHello) -> Next {
    let Some(name) = args.0.first().cloned() else {
        return ErrorNoNameProvided.to_render();
    };

    if name.len() > 10 {
        return ErrorNameTooLong(name.len() as u16).to_render();
    }

    if VEC_REGISTERED_NAMES.contains(&name.as_str()) {
        return ErrorNameNotAvailable.to_render();
    }

    ResultName(name).to_render()
}

/// Renders a successful greeting with the given name.
#[renderer]
fn render_result_name(name: ResultName) -> RenderResult {
    let mut render_result = RenderResult::new();
    writeln!(render_result, "Hello, {}!", *name).ok();
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
