//! Example: inline streaming render with `render!`
//!
//! Run:
//! ```bash
//! cargo run --manifest-path examples/example-render-stream/Cargo.toml --quiet -- stream
//! cargo run --manifest-path examples/example-render-stream/Cargo.toml --quiet -- stream 5 --json
//! cargo run --manifest-path examples/example-render-stream/Cargo.toml --quiet -- stream 2 --json-pretty
//! cargo run --manifest-path examples/example-render-stream/Cargo.toml --quiet -- stream 2 --yaml
//! ```
//!
//! Output (plain):
//! ```plaintext
//! task-0: 33%
//! task-1: 66%
//! task-2: 100%
//! finished: 100%
//! warning: 0%
//! owned: 42%
//! stderr: forced-stderr: 50%
//! ```
//!
//! Output (`--json`):
//! ```plaintext
//! {"name":"task-0","progress":33}
//! {"name":"task-1","progress":66}
//! {"name":"task-2","progress":100}
//! {"name":"finished","progress":100}
//! {"name":"warning","progress":0}
//! {"name":"owned","progress":42}
//! stderr: {"name":"forced-stderr","progress":50}
//! ```

use mingling::StructuralData;
use mingling::macros::buffer;
use mingling::prelude::*;
use mingling::setup::picker::StructuralRendererSetup;
use serde::Serialize;

dispatcher!("stream", EntryStream);

fn main() {
    let mut program = ThisProgram::new();
    program.with_setup(StructuralRendererSetup);
    program.exec_and_exit();
}

/// One streamed progress message.
///
/// `StructuralData` makes `--json` / `--yaml` render it structurally without a
/// dedicated structural renderer. `Clone` is required by `render!(&value)`.
#[derive(Grouped, StructuralData, Serialize, Clone)]
pub struct StreamMessage {
    /// The task name.
    pub name: String,
    /// Completion percentage.
    pub progress: u32,
}

/// Plain-text renderer; used when no structural format is selected.
///
/// In structural mode this renderer still runs for its side effects, but its
/// text output is replaced by the structural record.
#[renderer(buffer)]
pub fn render_stream_message(message: StreamMessage) {
    if message.progress == 0 {
        r_eprintln!("{}: {}%", message.name, message.progress);
    } else {
        r_println!("{}: {}%", message.name, message.progress);
    }
}

#[chain]
fn handle_stream(entry: EntryStream) {
    let count = entry
        .0
        .first()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(3)
        .max(1);

    let messages: Vec<StreamMessage> = (0..count)
        .map(|index| StreamMessage {
            name: format!("task-{index}"),
            progress: u32::try_from((index + 1) * 100 / count).unwrap_or(100),
        })
        .collect();

    // One record per item.
    render_each!(&messages);

    // Borrowed value, preserving the caller's ownership.
    let finished = StreamMessage {
        name: "finished".to_string(),
        progress: 100,
    };
    render!(&finished);

    // The plain renderer writes this one to stderr; flatten it to stdout.
    let warning = StreamMessage {
        name: "warning".to_string(),
        progress: 0,
    };
    render_stdout!(&warning);

    // The plain renderer writes this one to stdout; flatten it to stderr.
    let forced_stderr = StreamMessage {
        name: "forced-stderr".to_string(),
        progress: 50,
    };
    render_stderr!(&forced_stderr);

    // `render_owned!` is a wrapper around `render!`; it consumes the value.
    render_owned!(StreamMessage {
        name: "owned".to_string(),
        progress: 42,
    });
}

gen_program!();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capture_mode_collects_inline_records() {
        let mut program = mingling::Program::<ThisProgram>::new_with_args([
            "example-render-stream",
            "stream",
            "1",
            "--json",
        ]);
        program.with_setup(StructuralRendererSetup);

        let result = program.exec_without_render().unwrap();
        let output: String = result.into();

        assert!(output.contains("\"task-0\""));
        assert!(output.contains("\"finished\""));
        assert!(output.contains("\"warning\""));
        assert!(output.contains("\"forced-stderr\""));
        assert!(output.contains("\"owned\""));
    }
}
