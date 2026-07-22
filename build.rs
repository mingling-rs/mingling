use std::{env::current_dir, fs};

fn main() {
    gen_fake_cargo_toml_in_temp_dir();
}

fn gen_fake_cargo_toml_in_temp_dir() {
    fs::write(
        current_dir().unwrap().join(".temp").join("Cargo.toml"),
        "[workspace]",
    )
    .unwrap();
}
