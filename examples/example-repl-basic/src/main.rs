//! Example REPL Basic
//!
//! > This example demonstrates how to develop a REPL program using the `repl` feature
//!
//! Run:
//! ```bash
//! cargo run --manifest-path examples/example-repl-basic/Cargo.toml --quiet
//! ```

use mingling::{
    hook::ProgramHook,
    prelude::*,
    res::ResREPL,
    setup::{BasicREPLOutputSetup, BasicREPLPromptSetup, BasicREPLReadlineSetup},
    this,
};
use std::{env::current_dir, path::PathBuf};

// Resource to store the current directory
#[derive(Clone)]
struct ResCurrentDir {
    dir: PathBuf,
}

impl Default for ResCurrentDir {
    fn default() -> Self {
        Self {
            dir: current_dir().unwrap(),
        }
    }
}

fn main() {
    let mut program = ThisProgram::new();

    // Resource
    program.with_resource(ResCurrentDir::default());

    // Dispatchers
    program.with_dispatcher(CMDCd);
    program.with_dispatcher(CMDLs);
    program.with_dispatcher(CMDExit);
    program.with_dispatcher(CMDClear);

    // Setups
    // Enable basic std::io::stdin().read_line(&mut input)
    program.with_setup(BasicREPLReadlineSetup);

    // Enable basic output, using println! after Renderer finishes drawing
    program.with_setup(BasicREPLOutputSetup);

    // Enable basic Prompt display, with custom display logic
    program.with_setup(BasicREPLPromptSetup::func(|| {
        // Get the ResCurrentDir resource from the program
        let res = this::<ThisProgram>().res::<ResCurrentDir>().unwrap();
        let dir_str: String = res.dir.to_string_lossy().into();
        let prompt = format!(
            "{}> ",
            dir_str
                .replace(&['/', '\\'][..], ">")
                .trim_start_matches('>')
                .trim_end_matches('>')
        );
        prompt
    }));

    // Add hooks to handle REPL-related events
    program.with_hook(ProgramHook::empty().on_repl_begin(|_| {
        // Print welcome message
        println!("Welcome!");
    }));

    // Start the REPL loop
    program.exec_repl();
}

// Create error route
pack!(ErrorDirectoryNotExist = PathBuf);

// Create commands: cd ls exit
dispatcher!("cd", CMDCd => EntryCd);
dispatcher!("ls", CMDLs => EntryLs);
dispatcher!("exit", CMDExit => EntryExit);
dispatcher!("clear", CMDClear => EntryClear);

// Define data needed for the cd command's execution phase
pack!(StateChangeDirectory = String);

// Define data needed for the ls command's rendering phase
pack!(ResultList = Vec<String>);

// Parse cd command arguments
#[chain]
fn parse_cd_args(prev: EntryCd) -> Next {
    let join = prev.pick(()).unpack();
    StateChangeDirectory::new(join)
}

// Execute directory change
#[chain]
fn handle_cd(prev: StateChangeDirectory, current_dir: &mut ResCurrentDir) -> Next {
    use just_fmt::fmt_path::fmt_path;

    let join = prev.inner;
    let new_dir = fmt_path(current_dir.dir.join(join)).unwrap_or_default();

    // If the path is not found, route to error handling
    if !new_dir.exists() {
        return ErrorDirectoryNotExist::new(new_dir).to_render();
    }

    current_dir.dir = new_dir;
    empty_result!()
}

// Get directory contents via the CurrentDir resource
#[chain]
fn handle_ls(_prev: EntryLs, current_dir: &ResCurrentDir) -> Next {
    let dir = &current_dir.dir;
    let entries: Vec<String> = std::fs::read_dir(dir)
        .into_iter()
        .flat_map(|rd| rd.filter_map(std::result::Result::ok))
        .map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            if e.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                format!("{name}/")
            } else {
                name
            }
        })
        .collect();

    // Render ResultList
    ResultList::new(entries).to_render()
}

/// Render ResultList data
#[renderer]
fn render_list(list: ResultList) {
    for item in list.inner {
        r_println!("{}", item);
    }
}

// Handle exit command event
#[chain]
fn handle_exit(
    _prev: EntryExit,
    repl: &mut ResREPL, // Import REPL resource, registered in `exec_repl`, usable directly
) {
    // Set the REPL exit flag; REPL will exit after this loop iteration
    repl.exit = true;
}

/// Handle clear command event
#[chain]
fn handle_clear(_prev: EntryClear) {
    // Clear the terminal screen
    print!("\x1B[2J\x1B[1;1H");
}

/// Handle path not found event
#[renderer]
fn render_error_directory_not_exist(err: ErrorDirectoryNotExist) {
    r_println!("Directory not found: {}", err.inner.display())
}

/// Handle dispatcher not found event
/// Renders the error when a command is not found.
#[renderer]
fn dispatcher_not_found(prev: ErrorDispatcherNotFound) {
    r_println!("Command not found: \"{}\"", prev.join(", "))
}

gen_program!();
