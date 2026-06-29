use crate::Next;
use mingling::{macros::r_println, prelude::*};

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
pub fn render_my(msg: ResultMessage) {
    r_println!("Hello, {}!", *msg);
}
