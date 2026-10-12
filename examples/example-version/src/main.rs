//! Example Version
//!
//! > This example demonstrates how `--version` hijacks execution to print
//! > version information. The flag comes from `BasicProgramSetup`, and the
//! > output is produced by an `on_version` hook.
//!
//! Run
//! ```bash
//! cargo run --manifest-path examples/example-version/Cargo.toml --quiet -- --version
//! ```
//!
//! Output:
//! ```plain
//! example-version 0.1.0
//! ```

use mingling::{hook::ProgramHook, prelude::*, setup::BasicProgramSetup};
use std::io::Write;

dispatcher!("greet", EntryGreet);

fn main() {
    let mut program = ThisProgram::new();

    // --------- IMPORTANT ---------
    // `BasicProgramSetup` registers the `--help`, `--version`, `--verbose`,
    // `--quiet` and `--confirm` flags.
    program.with_setup(BasicProgramSetup);

    // The `version` hook runs when `--version` is passed; write the version
    // output into the provided render result.
    program.with_hook(ProgramHook::<ThisProgram>::empty().on_version(|info| {
        writeln!(info.result, "example-version {}", env!("CARGO_PKG_VERSION")).ok();
    }));
    // --------- IMPORTANT ---------

    program.exec_and_exit();
}

// Quickly wrap a type into a type recognizable by the current program
#[derive(Grouped, Wrap)]
pub struct ResultName(String);

#[chain]
fn handle_greet(args: EntryGreet) -> Next {
    let name: ResultName = args
        .0
        .first()
        .cloned()
        .unwrap_or_else(|| "World".to_string())
        .into();
    name.into()
}

/// Renders the greeting message with the provided name.
#[renderer]
fn render_name(name: ResultName) -> RenderResult {
    let mut render_result = RenderResult::new();
    writeln!(render_result, "Hello, {}!", *name).ok();
    render_result
}

gen_program!();
