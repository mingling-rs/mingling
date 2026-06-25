//! Example `pack_err!`
//!
//! > This example demonstrates how to use the `pack_err!` macro to define error types
//! > with automatic `name` field (set to snake_case at compile time) and optional `info` field.
//! > Also demonstrates `--json` serialization when `general_renderer` is enabled.
//!
//! Run:
//! ```bash
//! cargo run --manifest-path examples/example-pack-err/Cargo.toml --quiet -- find
//! cargo run --manifest-path examples/example-pack-err/Cargo.toml --quiet -- find Cargo.toml
//! cargo run --manifest-path examples/example-pack-err/Cargo.toml --quiet -- find src
//! cargo run --manifest-path examples/example-pack-err/Cargo.toml --quiet -- find-structural --json
//! cargo run --manifest-path examples/example-pack-err/Cargo.toml --quiet -- find-structural Cargo.toml --json
//! cargo run --manifest-path examples/example-pack-err/Cargo.toml --quiet -- find-structural src --json
//! ```
//!
//! Output:
//! ```plaintext
//! Search path not provided
//! Not a directory: Cargo.toml
//! Found directory: src
//! {"name":"error_not_found"}
//! {"name":"error_not_dir","info":"Cargo.toml"}
//! {"inner":"src"}
//! {"name":"error_not_found_structural"}
//! {"name":"error_not_dir_structural","info":"Cargo.toml"}
//! ```

use mingling::prelude::*;
use mingling::setup::GeneralRendererSetup;
use std::path::PathBuf;

dispatcher!("find", CMDFind => EntryFind);
dispatcher!("find-structural", CMDFindStructural => EntryFindStructural);

// --------- IMPORTANT ---------
// `pack_err!` is a convenient macro for defining error types.
//
//     Simple form:         pack_err!(ErrorNotFound);
//     Typed form:          pack_err!(ErrorNotDir = PathBuf);
//
// The simple form generates a struct with `name: String` and `impl Default`.
//   name = "error_not_found"  (automatically snake_cased at compile time)
//
// The typed form additionally generates `pub fn new(info)`.
//   name = "error_not_dir"
//
// When `general_renderer` is enabled, the struct also gets
// `#[derive(serde::Serialize)]` for --json / --yaml output.
// --------- IMPORTANT ---------

// Simple form — name = "error_not_found"
pack_err!(ErrorNotFound);

// Typed form — name = "error_not_dir"
pack_err!(ErrorNotDir = PathBuf);

// Simple form — with StructuralData support for --json / --yaml
pack_err_structural!(ErrorNotFoundStructural);

// Typed form — with StructuralData support for --json / --yaml
pack_err_structural!(ErrorNotDirStructural = PathBuf);

// Success type with StructuralData support
pack_structural!(ResultPath = PathBuf);

#[chain]
fn handle_find(args: EntryFind) -> Next {
    let Some(path_str) = args.inner.first().cloned() else {
        // No path provided → use the simple error form (Default)
        return ErrorNotFound::default().to_render();
    };

    let path = PathBuf::from(&path_str);
    if path.is_dir() {
        // Is a directory → success
        ResultPath::new(path).to_render()
    } else {
        // Not a directory (or doesn't exist) → use the typed error form
        ErrorNotDir::new(path).to_render()
    }
}

#[chain]
fn handle_find_structural(args: EntryFindStructural) -> Next {
    let Some(path_str) = args.inner.first().cloned() else {
        // No path provided → use the simple error form (Default)
        return ErrorNotFoundStructural::default().to_render();
    };

    let path = PathBuf::from(&path_str);
    if path.is_dir() {
        // Is a directory → success
        ResultPath::new(path).to_render()
    } else {
        // Not a directory (or doesn't exist) → use the typed error form
        ErrorNotDirStructural::new(path).to_render()
    }
}

/// Renders the successful result with the found directory path.
#[renderer]
fn render_result_path(path: ResultPath) {
    r_println!("Found directory: {}", path.display());
}

/// Renders the error when no search path is provided.
#[renderer]
fn render_error_not_found(_: ErrorNotFound) {
    r_println!("Search path not provided");
}

/// Renders the error when the given path is not a directory.
#[renderer]
fn render_error_not_dir(err: ErrorNotDir) {
    r_println!("Not a directory: {}", err.info.display());
}

/// Renders the structural error when no search path is provided.
#[renderer]
fn render_error_not_found_structural(_: ErrorNotFoundStructural) {
    r_println!("Search path not provided");
}

/// Renders the structural error when the given path is not a directory.
#[renderer]
fn render_error_not_dir_structural(err: ErrorNotDirStructural) {
    r_println!("Not a directory: {}", err.info.display());
}

gen_program!();

fn main() {
    let mut program = ThisProgram::new();
    // Add GeneralRendererSetup to support --json / --yaml flags
    program.with_setup(GeneralRendererSetup);
    program.with_dispatcher(CMDFind);
    program.with_dispatcher(CMDFindStructural);
    let _ = program.exec();
}
