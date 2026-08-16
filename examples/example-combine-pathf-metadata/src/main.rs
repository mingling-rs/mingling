//! Example: Combining pathf + entry metadata
//!
//! > Demonstrates combining the `pathf` feature with entry metadata. The metadata
//! > `DataType` (`Description`) and the dispatchers/entries are defined in the `sub`
//! > module. Thanks to `pathf`, `gen_program!()` resolves these types across
//! > modules automatically, so `main` stays minimal.
//!
//! Run:
//! ```bash
//! cargo run --manifest-path examples/example-combine-pathf-metadata/Cargo.toml --quiet -- hello Alice
//! cargo run --manifest-path examples/example-combine-pathf-metadata/Cargo.toml --quiet -- hello
//! cargo run --manifest-path examples/example-combine-pathf-metadata/Cargo.toml --quiet -- desc
//! ```
//!
//! Output:
//! ```plaintext
//! Hello, Alice!
//! Hello, World!
//! EntryHello desc = okay
//! ```

mod sub;

use mingling::prelude::*;

fn main() {
    ThisProgram::new().exec_and_exit();
}

gen_program!();
