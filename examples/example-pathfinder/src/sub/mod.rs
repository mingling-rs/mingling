use crate::Next;
use mingling::prelude::*;
use std::io::Write;

dispatcher!("greet", CMDGreet => EntryGreet);
pack!(ResultName = String);

#[chain]
pub fn handle_greet(args: EntryGreet) -> Next {
    let name: ResultName = args
        .inner
        .first()
        .cloned()
        .unwrap_or_else(|| "World".to_string())
        .into();
    name
}

/// Renders the name.
#[renderer]
// But renderers cannot use the `async` keyword
pub fn render_name(name: ResultName) -> RenderResult {
    let mut render_result = RenderResult::new();
    writeln!(render_result, "Hello, {}!", *name).ok();
    render_result
}
