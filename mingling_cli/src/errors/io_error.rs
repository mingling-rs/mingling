use mingling::macros::{group, renderer};

group!(ErrorIo = std::io::Error);

#[renderer]
pub fn handle_error_io(err: ErrorIo) {
    panic!("{}", err.to_string())
}
