//! Example Resource Injection
//!
//! > This example demonstrates how to read and write the program's global state using Mingling's resource system
//!
//! Run:
//! ```bash
//! cargo run --manifest-path examples/example-resources/Cargo.toml --quiet current
//! cargo run --manifest-path examples/example-resources/Cargo.toml --quiet modify-current src
//! ```
//!
//! Output:
//! ```plaintext
//! Current directory: /home/alice/mingling
//! Current directory: /home/alice/mingling/src
//! ```

use mingling::prelude::*;
use std::io::Write;
use std::path::PathBuf;

// Create resource
//        ______________ Resource needs to
//       /        /        implement the following two traits
//       vvvvvvv  vvvvv
#[derive(Default, Clone)]
struct ResCurrentDir {
    current_dir: PathBuf,
}

fn main() {
    let mut program = ThisProgram::new();

    // --------- IMPORTANT ---------
    // Use `with_resource` to inject a singleton into the program
    program.with_resource(ResCurrentDir {
        current_dir: std::env::current_dir().unwrap(),
    });
    // --------- IMPORTANT ---------

    program.exec_and_exit();
}

dispatcher!("current", EntryCurrent);
dispatcher!("modify-current", EntryModifyCurrent);

// Define chain for modifying current directory                  _________________ Injected muttable resource
//                                                              /
#[chain] //                                                     vvvvvvvvvvvvvvvvvv
fn render_modify_current(args: EntryModifyCurrent, current_dir: &mut ResCurrentDir) -> Next {
    current_dir.current_dir = current_dir
        .current_dir
        .join(args.pick_or_default(&arg![String]).unwrap());
    EntryCurrent::default().into()
}

// Define renderer for output current path       _____________ Injected resource
//                                              /
/// Renders the current directory path.         |
#[renderer] //                                  vvvvvvvvvvvvvv
fn render_current(_: EntryCurrent, current_dir: &ResCurrentDir) -> RenderResult {
    let mut render_result = RenderResult::new();
    write!(
        render_result,
        "Current directory: {}",
        current_dir.current_dir.display()
    )
    .ok();
    render_result
}

gen_program!();
