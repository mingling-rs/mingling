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
    let mut program = ThisProgram::new();
    program.with_dispatcher(sub::CMDHello);
    program.with_dispatcher(sub::CMDDescription);
    program.exec_and_exit();
}

gen_program!();
