use crate::Next;
use mingling::prelude::*;
use std::io::Write;

dispatcher!("greet", EntryGreet);

#[derive(Grouped, Wrap)]
pub struct ResultName(String);

#[chain]
pub fn handle_greet(args: EntryGreet) -> Next {
    let name: ResultName = args
        .0
        .first()
        .cloned()
        .unwrap_or_else(|| "World".to_string())
        .into();
    name.into()
}

/// Renders the name.
#[renderer]
// But renderers cannot use the `async` keyword
pub fn render_name(name: ResultName) -> RenderResult {
    let mut render_result = RenderResult::new();
    writeln!(render_result, "Hello, {}!", *name).ok();
    render_result
}
