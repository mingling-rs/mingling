use crate::Next;
use mingling::prelude::*;
use std::io::Write;

dispatcher!("hello");

#[derive(Grouped, Wrap)]
pub struct ResultMessage(String);

#[chain]
pub fn handle_my(args: EntryHello) -> Next {
    let name: ResultMessage = args
        .0
        .first()
        .cloned()
        .unwrap_or_else(|| "World".to_string())
        .into();
    name.into()
}

#[renderer]
pub fn render_my(msg: ResultMessage) -> RenderResult {
    let mut render_result = RenderResult::new();
    writeln!(render_result, "Hello, {}!", *msg).ok();
    render_result
}
