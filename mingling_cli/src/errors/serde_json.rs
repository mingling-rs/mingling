use mingling::macros::{buffer, import_type, r_println, renderer};

import_type!(ErrorSerdeJson = serde_json::Error);

#[renderer(buffer)]
pub fn render_error_serde_json(_err: ErrorSerdeJson) {
    r_println!("serde");
}
