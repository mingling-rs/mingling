use cargo_metadata::Message;
use mingling::macros::{buffer, group, r_println, renderer, renderify};

group!(Message);

#[renderer(buffer, renderify)]
pub fn render_message(msg: Message) {
    let r = serde_json::to_string(&msg)?;
    r_println!("{}", r);
}
