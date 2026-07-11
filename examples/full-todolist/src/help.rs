//! This module provides help information for the `todolist` command line program

use crate::{EntryAdd, EntryClean, EntryComplete, EntryList, ErrorDispatcherNotFound};
use mingling::{RenderResult, macros::help};
use std::io::Write;

/// Shows the global help message.
#[help]
pub fn help_global(_p: ErrorDispatcherNotFound) -> RenderResult {
    let mut render_result = RenderResult::new();
    writeln!(
        render_result,
        r"Usage: todolist [command] [args]

Commands:
  add            -- Add a new task
  list           -- List all tasks
  complete       -- Mark a task as complete
  clean          -- Clean up completed tasks

Args:
  -h, --help     -- Show this help message
  -V, --version  -- Show the version
  -A, --all      -- All tasks (Clean all / List all)"
    )
    .ok();
    render_result
}

/// Shows help for the `add` command.
#[help]
pub fn help_add(_p: EntryAdd) -> RenderResult {
    let mut render_result = RenderResult::new();
    writeln!(
        render_result,
        r"Usage: todolist add [task description]

Add a new task to the todo list.

Example:
  todolist add 'Buy groceries'
  todolist add 'Finish Rust project'"
    )
    .ok();
    render_result
}

/// Shows help for the `list` command.
#[help]
pub fn help_list(_p: EntryList) -> RenderResult {
    let mut render_result = RenderResult::new();
    writeln!(
        render_result,
        r"Usage: todolist list

List all tasks.

Example:
  todolist list"
    )
    .ok();
    render_result
}

/// Shows help for the `complete` command.
#[help]
pub fn help_complete(_p: EntryComplete) -> RenderResult {
    let mut render_result = RenderResult::new();
    writeln!(
        render_result,
        r"Usage: todolist complete [task_id]

Mark a task as complete by its ID.

Example:
  todolist complete 1
  todolist complete 3"
    )
    .ok();
    render_result
}

/// Shows help for the `clean` command.
#[help]
pub fn help_clean(_p: EntryClean) -> RenderResult {
    let mut render_result = RenderResult::new();
    writeln!(
        render_result,
        r"Usage: todolist clean

Remove all completed tasks from the list.

Example:
  todolist clean"
    )
    .ok();
    render_result
}
