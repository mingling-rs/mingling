//! Example: Entry Metadata (no `pathf`)
//!
//! > Demonstrates attaching arbitrary, compile-time-typed metadata (`Description`)
//! > to an entry via `#[metadata(Entry)]`, and retrieving it at runtime through
//! > `ProgramCollect::get_metadata`. The `desc` and `nodoc` subcommands dispatch
//! > through the normal chain/render pipeline — exactly like `example-basic`.
//!
//! Run:
//! ```bash
//! cargo run --manifest-path examples/example-metadata/Cargo.toml --quiet -- greet Alice
//! cargo run --manifest-path examples/example-metadata/Cargo.toml --quiet -- greet
//! cargo run --manifest-path examples/example-metadata/Cargo.toml --quiet -- desc
//! cargo run --manifest-path examples/example-metadata/Cargo.toml --quiet -- nodoc
//! ```
//!
//! Output:
//! ```plaintext
//! Hello, Alice!
//! Hello, World!
//! EntryGreet desc = ok
//! EntryDescription has no description
//! ```

use mingling::{macros::metadata, prelude::*};
use std::io::Write;

// Define the `greet` subcommand
dispatcher!("greet", CMDGreet => EntryGreet);

// Define the `desc` subcommand, which queries metadata bound to EntryGreet
dispatcher!("desc", CMDDescription => EntryDescription);

// Define the `nodoc` subcommand, which queries metadata for an entry that has none
dispatcher!("nodoc", CMDNoDescription => EntryNoDescription);

fn main() {
    let mut program = ThisProgram::new();
    program.with_dispatcher(CMDGreet);
    program.with_dispatcher(CMDDescription);
    program.with_dispatcher(CMDNoDescription);
    program.exec_and_exit();
}

/// The metadata type attached to an entry.
#[derive(Debug, PartialEq, Eq)]
pub struct Description {
    pub desc: String,
}

// --------- IMPORTANT ---------
/// Attach a `Description` to `EntryGreet`.
///
/// - `BindType` = `EntryGreet` (the enum variant / entry type)
/// - `DataType` = `Description` (the function's return type)
#[metadata(EntryGreet)]
pub fn greet_desc() -> Description {
    Description {
        desc: "ok".to_string(),
    }
}
// --------- IMPORTANT ---------

pack!(ResultName = String);
pack!(DescResult = String);

/// Chain for `greet` — reads the name and produces a `ResultName`.
#[chain]
fn handle_greet(args: EntryGreet) -> Next {
    let name: ResultName = args
        .inner
        .first()
        .cloned()
        .unwrap_or_else(|| "World".to_string())
        .into();
    name.into()
}

/// Chain for `desc` — looks up the metadata bound to `EntryGreet`.
#[chain]
fn handle_desc(_args: EntryDescription) -> Next {
    use mingling::ProgramCollect;
    // --------- IMPORTANT ---------
    let msg = match ThisProgram::get_metadata::<Description>(ThisProgram::EntryGreet) {
        Some(d) => format!("EntryGreet desc = {}", d.desc),
        None => "EntryGreet has no description".to_string(),
    };
    // --------- IMPORTANT ---------
    DescResult::new(msg).to_render()
}

/// Chain for `nodoc` — asks for metadata on an entry that has none.
#[chain]
fn handle_nodoc(_args: EntryNoDescription) -> Next {
    use mingling::ProgramCollect;
    // --------- IMPORTANT ---------
    let msg = match ThisProgram::get_metadata::<Description>(ThisProgram::EntryDescription) {
        Some(d) => format!("EntryDescription desc = {}", d.desc),
        None => "EntryDescription has no description".to_string(),
    };
    // --------- IMPORTANT ---------
    DescResult::new(msg).to_render()
}

/// Renders the greeting message with the provided name.
#[renderer]
fn render_name(name: ResultName) -> RenderResult {
    let mut render_result = RenderResult::new();
    writeln!(render_result, "Hello, {}!", *name).ok();
    render_result
}

/// Renders the metadata query result.
#[renderer]
fn render_desc(msg: DescResult) -> RenderResult {
    let mut render_result = RenderResult::new();
    writeln!(render_result, "{}", *msg).ok();
    render_result
}

gen_program!();
