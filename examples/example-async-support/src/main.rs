//! Example Async Runtime Support
//!
//! > This example shows how to drive an async runtime using the `async` feature
//!
//! ## Note
//!
//! When the `async` feature is enabled, **Mingling** provides a different framework implementation,
//! allowing you to use the `async` keyword directly within `#[chain]`.
//!
//! However, you will lose some capabilities:
//!
//! 1. The program will not be able to use panic unwind functionality
//!
//! Run:
//! ```bash
//! cargo run --manifest-path examples/example-async-support/Cargo.toml --quiet -- download README.md
//! ```
//!
//! Output:
//! ```plaintext
//! Download begin
//! # (Will pause for 1 second here)
//! "README.md" downloaded.
//! ```

use mingling::{hook::ProgramHook, prelude::*};
use std::io::Write;

#[tokio::main]
async fn main() {
    let mut program = ThisProgram::new();

    program.with_dispatcher(CMDDownload);

    // Add a hook to display when the download begins
    program.with_hook(ProgramHook::empty().on_begin::<_, ()>(|_| println!("Download begin")));

    // --------- IMPORTANT ---------
    // The return values of `exec_*()` related functions have been replaced with Futures
    program.exec_and_exit().await;
    // --------- IMPORTANT ---------
}

dispatcher!("download", CMDDownload => EntryDownload);

pack!(ResultDownloaded = String);

// --------- IMPORTANT ---------
#[chain]
//  vvvvv_ `async` keyword can be used directly here
pub async fn handle_download(args: EntryDownload) -> Next {
    let file_name = args.pick(()).unpack();
    fake_download(file_name).await
}

/// Renders the downloaded file name.
#[renderer]
// But renderers cannot use the `async` keyword
pub fn render_downloaded(result: ResultDownloaded) -> RenderResult {
    let mut render_result = RenderResult::new();
    writeln!(render_result, "\"{}\" downloaded.", *result).ok();
    render_result
}
// --------- IMPORTANT ---------

gen_program!();

async fn fake_download(file_name: String) -> ResultDownloaded {
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    ResultDownloaded::new(file_name)
}
