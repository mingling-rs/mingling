use crate::Next;
use crate::ThisProgram;
use mingling::ProgramCollect;
use mingling::macros::metadata;
use mingling::prelude::*;
use std::io::Write;

// Implicit dispatcher form — creates `CMDHello` / `EntryHello` in this module
dispatcher!("hello");
// Creates `CMDDescription` / `EntryDescription` in this module
dispatcher!("desc", CMDDescription => EntryDescription);

/// The metadata type attached to an entry (`DataType`).
#[derive(Debug, PartialEq, Eq)]
pub struct Description {
    pub desc: String,
}

/// Attach a `Description` to `EntryHello`.
///
/// - `BindType` = `EntryHello` (the enum variant / entry type)
/// - `DataType` = `Description` (the function's return type)
#[metadata(EntryHello)]
pub fn hello_desc() -> Description {
    Description {
        desc: "okay".to_string(),
    }
}

pack!(ResultName = String);
pack!(DescResult = String);

/// Chain for `hello` — reads the name and produces a `ResultName`.
#[chain]
pub fn handle_hello(args: EntryHello) -> Next {
    let name: ResultName = args
        .inner
        .first()
        .cloned()
        .unwrap_or_else(|| "World".to_string())
        .into();
    name.into()
}

/// Chain for `desc` — looks up the metadata bound to `EntryHello`.
#[chain]
pub fn handle_desc(_args: EntryDescription) -> Next {
    // --------- IMPORTANT ---------
    let msg = match ThisProgram::get_metadata::<Description>(ThisProgram::EntryHello) {
        Some(d) => format!("EntryHello desc = {}", d.desc),
        None => "EntryHello has no description".to_string(),
    };
    // --------- IMPORTANT ---------
    DescResult::new(msg).to_render()
}

/// Renders the greeting message with the provided name.
#[renderer]
pub fn render_name(name: ResultName) -> RenderResult {
    let mut render_result = RenderResult::new();
    writeln!(render_result, "Hello, {}!", *name).ok();
    render_result
}

/// Renders the metadata query result.
#[renderer]
pub fn render_desc(msg: DescResult) -> RenderResult {
    let mut render_result = RenderResult::new();
    writeln!(render_result, "{}", *msg).ok();
    render_result
}
