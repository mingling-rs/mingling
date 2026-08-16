//! Example: Module Pathfinder (pathf)
//!
//! > This example demonstrates how to use the `pathf` feature to define types
//! > in submodules without needing explicit `use` in the main module.
//! > All type paths are resolved automatically at build time.
//!
//! Run:
//! ```bash
//! cargo run --manifest-path examples/example-pathfinder/Cargo.toml --quiet -- greet
//! cargo run --manifest-path examples/example-pathfinder/Cargo.toml --quiet -- greet Alice
//! ```
//!
//! Output:
//! ```plaintext
//! Hello, World!
//! Hello, Alice!
//! ```

mod sub;

use mingling::macros::gen_program;

fn main() {
    ThisProgram::new().exec_and_exit();
}

gen_program!();
