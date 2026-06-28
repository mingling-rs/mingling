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
use crate::sub::CMDGreet;

fn main() {
    let mut program = ThisProgram::new();
    program.with_dispatcher(CMDGreet);
    program.exec_and_exit();
}

gen_program!();
