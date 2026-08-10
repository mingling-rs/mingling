//! Example: Combining `pathf` + `dispatch_tree`
//!
//! > This example demonstrates how to use `pathf` and `dispatch_tree` together.
//! > Types are defined in a submodule (`sub`), and `gen_program!()` resolves
//! > them automatically via pathf without explicit `use` imports.
//! >
//! > **Important**: `dispatch_tree` must be enabled in BOTH `[dependencies]`
//! > AND `[build-dependencies]` so that pathf's builder can detect
//! > `__internal_dispatcher_*` types needed by the dispatch tree.
//! >
//! > Also requires `extras` for the implicit `dispatcher!("hello")` form.
//!
//! Run:
//! ```bash
//! cargo run --manifest-path examples/example-combine-pathf-dispatch-tree/Cargo.toml --quiet -- hello Alice
//! ```
//!
//! Output:
//! ```plaintext
//! Hello, Alice!
//! ```

mod sub;

use mingling::macros::gen_program;

fn main() {
    ThisProgram::new().exec_and_exit();
}

gen_program!();
