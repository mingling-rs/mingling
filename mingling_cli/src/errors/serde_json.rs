use mingling::macros::{buffer, group, r_println, renderer};

group!(ErrorSerdeJson = serde_json::Error);

#[renderer(buffer)]
pub fn render_error_serde_json(_err: ErrorSerdeJson) {
    r_println!("serde");
}
