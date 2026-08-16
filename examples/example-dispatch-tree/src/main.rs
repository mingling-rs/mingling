//! Example Dispatch Tree
//!
//! > This example will introduce how to use `dispatch_tree`
//! > to optimize your command line lookup efficiency
//!
//! When the number of commands in your project increases, you can enable
//! `dispatch_tree` to switch command matching from a linear scan to a
//! character-level trie.
//!
//! Run:
//! ```bash
//! cargo run --manifest-path examples/example-dispatch-tree/Cargo.toml --quiet -- cmd5
//! ```
//!
//! Output:
//! ```plaintext
//! It's works!
//! ```
//!

use mingling::prelude::*;
use std::io::Write;

// --------- IMPORTANT ---------
// You have a large number of subcommands
dispatcher!("cmd1",         Entry1);
dispatcher!("cmd2.sub1",   Entry2Sub1);
dispatcher!("cmd2.sub2",   Entry2Sub2);
dispatcher!("cmd3.sub1.leaf1", Entry3Sub1Leaf1);
dispatcher!("cmd3.sub1.leaf2", Entry3Sub1Leaf2);
dispatcher!("cmd3.sub2",   Entry3Sub2);
dispatcher!("cmd4.sub1.subsub1.deep", Entry4Deep);
dispatcher!("cmd4.sub1.subsub2",      Entry4SubSub2);
dispatcher!("cmd5",        Entry5);
dispatcher!("cmd5.extra",  Entry5Extra);
dispatcher!("nested.a.b.c", EntryA);
dispatcher!("nested.a.b.d", EntryB);
dispatcher!("nested.a.e",   EntryC);
dispatcher!("nested.f",     EntryD);
// --------- IMPORTANT ---------

fn main() {
    let program = ThisProgram::new();
    program.exec_and_exit();
}

/// Renders the confirmation message for the `cmd5` command.
#[renderer]
fn render_cmd5(_: Entry5) -> RenderResult {
    let mut render_result = RenderResult::new();
    writeln!(render_result, "It's works!").ok();
    render_result
}

gen_program!();
