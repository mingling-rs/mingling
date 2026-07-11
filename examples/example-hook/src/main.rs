//! Example Hook
//!
//! > This example demonstrates how to use Mingling's hook system to obtain debugging information during program execution
//!
//! Run:
//! ```base
//! cargo run --manifest-path examples/example-hook/Cargo.toml --quiet -- greet Alice
//! ```
//!
//! Output:
//! ```plaintext
//! [DEBUG] Program is begin
//! [DEBUG] Pre dispatch: ["greet", "Alice"]
//! [DEBUG] Post dispatch: EntryGreet
//! [DEBUG] Pre chain: EntryGreet
//! [DEBUG] Post chain: ResultName
//! [DEBUG] Pre render: ResultName
//! [DEBUG] Post render
//! [DEBUG] Program end
//! Hello, Alice!
//! ```

use mingling::{
    hook::{ProgramControlUnit, ProgramHook},
    prelude::*,
};
use std::io::Write;

dispatcher!("greet", CMDGreet => EntryGreet);

fn main() {
    let mut program = ThisProgram::new();

    // --------- IMPORTANT ---------
    program.with_hook(
        ProgramHook::<ThisProgram>::empty()
            .on_begin::<_, ()>(|_| println!("[DEBUG] Program is begin"))
            .on_pre_dispatch(|info| println!("[DEBUG] Pre dispatch: {}", info.arguments.join(" ")))
            .on_post_dispatch(|info| println!("[DEBUG] Post dispatch: {}", info.entry))
            .on_pre_chain(|info| {
                println!("[DEBUG] Pre chain: {}", info.input);
            })
            .on_post_chain(|info| println!("[DEBUG] Post chain: {}", info.output.member_id))
            .on_finish(|_| {
                println!("[DEBUG] Loop end");
                ProgramControlUnit::OverrideExitCode(0) // Override exit code
            })
            .on_pre_render(|info| println!("[DEBUG] Pre render: {}", info.input))
            .on_post_render(|_| println!("[DEBUG] Post render")),
    );
    // --------- IMPORTANT ---------

    program.with_dispatcher(CMDGreet);
    program.exec_and_exit();
}

pack!(ResultName = String);

#[chain]
fn handle_greet(args: EntryGreet) -> Next {
    let name: ResultName = args
        .inner
        .first()
        .cloned()
        .unwrap_or_else(|| "World".to_string())
        .into();
    name
}

/// Renders the greeting message with the provided name.
/// Renders the greeting message with the provided name.
#[renderer]
fn render_name(name: ResultName) -> RenderResult {
    let mut render_result = RenderResult::new();
    writeln!(render_result, "Hello, {}!", *name).ok();
    render_result
}

gen_program!();
