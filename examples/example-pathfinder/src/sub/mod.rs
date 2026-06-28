use mingling::prelude::*;
use crate::Next;

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

#[renderer]
pub fn render_name(name: ResultName) {
    r_println!("Hello, {}!", *name);
}
