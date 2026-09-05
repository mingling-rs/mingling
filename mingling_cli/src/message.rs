use cargo_metadata::Message;
use mingling::macros::{buffer, import_type, r_println, renderer, renderify};

import_type!(cargo_metadata::Message);

#[renderer(buffer, renderify)]
pub fn render_message(msg: Message) {
    let r = serde_json::to_string(&msg)?;
    r_println!("{}", r);
}
