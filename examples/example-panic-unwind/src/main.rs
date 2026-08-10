//! Example Panic Unwind
//!
//! > This example introduces how to catch Panic in the Mingling program loop
//!
//! Run:
//! ```bash
//! cargo run --manifest-path examples/example-panic-unwind/Cargo.toml --quiet -- panic
//! cargo run --manifest-path examples/example-panic-unwind/Cargo.toml --quiet -- panic OhMyGod
//! ```
//!
//! Output:
//! ```plaintext
//! Program not panic
//! Program panic: OhMyGod
//! OhMyGod
//! ```

use mingling::PanicSilence;
use mingling::{hook::ProgramHook, prelude::*};
use std::io::Write;

dispatcher!("panic", CMDPanic => EntryPanic);
pack!(NotPanic = ());

fn main() {
    let mut program = ThisProgram::new();
    program.with_dispatcher(CMDPanic);

    // --------- IMPORTANT ---------
    // Enable silence_panic to suppress automatic Panic output
    program.stdout_setting.silence_panic = PanicSilence::Silence;

    // Define a hook to output &ProgramPanic when a Panic occurs
    program.with_hook(
        ProgramHook::empty()
            .on_exec_panic::<_, ()>(|info| println!("Program panic: {}", info.panic)),
    );
    // --------- IMPORTANT ---------

    let _ = program.exec();
}

#[chain]
fn handle_panic(prev: EntryPanic) -> Next {
    let panic_info = prev.pick::<Option<String>>(()).unpack();
    match panic_info {
        Some(s) => {
            // Panic happens here, will be caught
            panic!("{}", s)
        }
        None => NotPanic::default().into(),
    }
}

/// Renders the message when no panic occurs.
#[renderer]
pub fn render(_: NotPanic) -> RenderResult {
    let mut render_result = RenderResult::new();
    writeln!(render_result, "Program not panic").ok();
    render_result
}

gen_program!();
