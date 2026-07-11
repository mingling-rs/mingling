use crate::Next;
use mingling::prelude::*;
use std::io::Write;

dispatcher!("hello");

pack!(ResultMessage = String);

#[chain]
pub fn handle_my(args: EntryHello) -> Next {
    let name: ResultMessage = args
        .inner
        .first()
        .cloned()
        .unwrap_or_else(|| "World".to_string())
        .into();
    name
}

#[renderer]
pub fn render_my(msg: ResultMessage) -> RenderResult {
    let mut render_result = RenderResult::new();
    writeln!(render_result, "Hello, {}!", *msg).ok();
    render_result
}
