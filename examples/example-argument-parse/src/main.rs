//! Example Argument Parse
//!
//! > This example demonstrates how to use the `parser` feature to parse user input
//!
//! Run:
//! ```bash
//! cargo run --manifest-path examples/example-argument-parse/Cargo.toml --quiet -- transfer README.md --size 32kib
//! cargo run --manifest-path examples/example-argument-parse/Cargo.toml --quiet -- transfer src/ --dir
//! cargo run --manifest-path examples/example-argument-parse/Cargo.toml --quiet -- strict-transfer README.md
//! cargo run --manifest-path examples/example-argument-parse/Cargo.toml --quiet -- strict-transfer --dir
//! ```
//!
//! Output:
//! ```plaintext
//! file: README.md (32768)
//! dir: src/ (1048576)
//! file: README.md (1048576)
//! Error: name is not provided
//! ```

use mingling::{macros::route, prelude::*};

dispatcher!("transfer", CMDTransfer => EntryTransfer);
dispatcher!("strict-transfer", CMDStrictTransfer => EntryStrictTransfer);

pack!(ResultFile = (bool, usize, String)); // (IsDir, Size, Name)

#[chain]
fn handle_transfer_parse(args: EntryTransfer) -> Next {
    // --------- IMPORTANT ---------
    // First parse flag arguments (like --dir/-D), then positional arguments
    let result: ResultFile = args
        // Name --dir --size 20mib
        //            ^^^^^^^^^^^^_ first
        .pick::<bool>(["--dir", "-D"])
        // Name --dir
        //      ^^^^^_ second (or `-D`)
        .pick_or::<usize>("--size", 1024 * 1024_usize)
        // Name
        // ^^^^_ finally, pick positional arg
        .pick::<String>(())
        .after(|str| str.trim().replace(' ', ""))
        // Unpack to tuple (is_dir, size, name)
        .unpack()
        // Convert into ResultFile
        .into();
    // --------- IMPORTANT ---------
    result
}

pack!(ErrorNoNameProvided = ());

#[chain]
fn handle_strict_transfer_parse(args: EntryStrictTransfer) -> Next {
    // --------- IMPORTANT ---------
    // Strict parsing: error immediately if the name is not provided
    let result: ResultFile = route! { // Use `route!` to wrap a Picker that contains `or_route`
        args
            .pick::<bool>(["--dir", "-D"])
            .pick_or::<usize>("--size", 1024 * 1024_usize)
            // Finally parse the positional argument; if not found, route to `ErrorNoNameProvided`
            .pick_or_route::<String, _>((), ErrorNoNameProvided::default())
            .after(|str| str.trim().replace(' ', ""))
            .unpack()
    }
    // Convert into ResultFile
    .into();
    // --------- IMPORTANT ---------
    result.to_chain()
}

/// Renders the parsed transfer result (file/dir, size, name).
#[renderer]
fn render_result_file(result: ResultFile) {
    let (is_dir, size, name) = result.into();
    r_println!(
        "{}: {} ({})",
        if is_dir { "dir" } else { "file" },
        name,
        size
    )
}

/// Renders the error when no name is provided.
#[renderer]
fn render_error_no_name_provided(_: ErrorNoNameProvided) {
    r_println!("Error: name is not provided")
}

gen_program!();

fn main() {
    let mut program = ThisProgram::new();
    program.with_dispatcher(CMDTransfer);
    program.with_dispatcher(CMDStrictTransfer);
    program.exec_and_exit();
}
