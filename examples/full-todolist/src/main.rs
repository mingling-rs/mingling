//! Full Example - Todo List
//!
//! This example introduces how to use Mingling's features to build a complete CLI program
//!
//! > HAHA:
//! >
//! > This is truly a cliché example, as common as `Hello World`!

use mingling::{
    LazyInit, LazyRes,
    macros::route,
    prelude::*,
    res::ResExitCode,
    setup::{ExitCodeSetup, HelpFlagSetup, StructuralRendererSetup},
};
use std::io::Write;

mod help;
pub use help::*;

mod todolist;
pub use todolist::*;

#[derive(Default, Clone)]
pub struct ResProgramFlags {
    pub all: bool,
}

// Define dispatchers

dispatcher!("add");
dispatcher!("list");
dispatcher!("complete");
dispatcher!("clean");

// Define states

pack!(StateAddTodo = String);
pack!(StateCompleteTodo = i32);
pack!(StateListTodo = bool);

// Define errors

pack!(ErrorNoTaskDescriptionProvided = ());
pack!(ErrorNoIndexProvided = ());
pack!(ErrorIndexOutOfBounds = ());

fn main() {
    let mut program = ThisProgram::new();

    // Setups
    program.with_setup(ExitCodeSetup::default());
    program.with_setup(StructuralRendererSetup);
    program.with_setup(HelpFlagSetup::new(["--help", "-h"]));

    // Flags
    let all = program.pick_global_flag(["-A", "--all"]);

    // Resources
    program.with_resource(
        // Load on use
        ResTodoList::lazy_init(read_todo_list)
            // Write on drop
            .with_on_drop(write_todo_list),
    );
    program.with_resource(ResProgramFlags { all });

    // Dispatchers
    program.with_dispatcher(CMDAdd);
    program.with_dispatcher(CMDComplete);
    program.with_dispatcher(CMDList);
    program.with_dispatcher(CMDClean);

    program.exec_and_exit();
}

// Command `add`

#[chain]
fn handle_add(args: EntryAdd) -> Next {
    let task: String = route! {
        args
            .pick_or_route((), ErrorNoTaskDescriptionProvided::new(()))
            .unpack()
    };
    StateAddTodo::new(task).to_chain()
}

#[chain]
fn handle_state_add_todo(
    state: StateAddTodo,
    // Inject ResTodoList resource
    todolist: &mut LazyRes<ResTodoList>,
) -> Next {
    let todolist = todolist.get_mut();

    // Unpack state and read description
    let description = state.inner;

    todolist.items.push(Todo {
        item: description,
        completed: false,
    });

    // Clone todolist to render
    let todolist = todolist.clone();
    todolist.to_render()
}

// Command `list`

#[chain]
fn handle_list(_args: EntryList, todolist: &mut LazyRes<ResTodoList>) -> Next {
    let todolist = todolist.get_mut().clone();
    todolist.to_render()
}

// Command `complete`

#[chain]
fn handle_complete(args: EntryComplete) -> Next {
    let index: i32 = route! {
        args.pick_or_route((), ErrorNoIndexProvided::new(())).unpack()
    };
    StateCompleteTodo::new(index).to_chain()
}

#[chain]
fn handle_state_complete_todo(
    state: StateCompleteTodo,
    todolist: &mut LazyRes<ResTodoList>,
) -> Next {
    let todolist = todolist.get_mut();
    let index = state.inner as usize;
    if index < todolist.items.len() {
        todolist.items[index].completed = true;
        todolist.clone().to_render()
    } else {
        ErrorIndexOutOfBounds::new(()).to_render()
    }
}

// Command `clean`

#[chain]
fn handle_clean(
    _args: EntryClean,
    todolist: &mut LazyRes<ResTodoList>,
    program_flags: &ResProgramFlags,
) -> Next {
    let todolist = todolist.get_mut();
    if program_flags.all {
        todolist.items.clear();
    } else {
        todolist.items.retain(|item| !item.completed);
    }
    if todolist.items.is_empty() {
        empty_result!()
    } else {
        todolist.clone().to_render()
    }
}

/// Renders error when no task description is provided.
#[renderer]
pub fn render_error_no_task_description_provided(
    _err: ErrorNoTaskDescriptionProvided,
    // ExitCode
    ec: &mut ResExitCode,
) -> RenderResult {
    let mut render_result = RenderResult::new();
    writeln!(render_result, "No task description provided!").ok();
    writeln!(render_result).ok();
    writeln!(render_result, "Use `todolist add <desc>` to add a task").ok();
    ec.exit_code = 1;
    render_result
}

/// Renders error when no index is provided.
#[renderer]
pub fn render_error_no_index_provided(
    _err: ErrorNoIndexProvided,
    // ExitCode
    ec: &mut ResExitCode,
) -> RenderResult {
    let mut render_result = RenderResult::new();
    writeln!(render_result, "No index provided!").ok();
    writeln!(render_result).ok();
    writeln!(render_result, "Use `todolist list` to query indexes").ok();
    ec.exit_code = 2;
    render_result
}

/// Renders error when index is out of bounds.
#[renderer]
pub fn render_error_index_out_of_bounds(
    _err: ErrorIndexOutOfBounds,
    // ExitCode
    ec: &mut ResExitCode,
) -> RenderResult {
    let mut render_result = RenderResult::new();
    writeln!(render_result, "Index out of bounds!").ok();
    ec.exit_code = 3;
    render_result
}

gen_program!();
