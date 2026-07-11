//! Data structures, read and write logic for the todo list

use mingling::{Groupped, RenderResult, macros::renderer};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::{env::current_dir, path::PathBuf};

use crate::ResProgramFlags;

#[derive(Debug, Serialize, Deserialize, Clone, Default, Groupped)]
pub struct ResTodoList {
    pub items: Vec<Todo>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Todo {
    pub item: String,
    pub completed: bool,
}

fn get_todo_list_path() -> PathBuf {
    current_dir().unwrap().join(".todo.json")
}

pub fn read_todo_list() -> ResTodoList {
    let path = get_todo_list_path();
    if !path.exists() {
        return ResTodoList::default();
    }
    let file = match std::fs::File::open(&path) {
        Ok(f) => f,
        Err(_) => return ResTodoList::default(),
    };
    let reader = std::io::BufReader::new(file);
    serde_json::from_reader(reader).unwrap_or_default()
}

/// Renders the todo list.
#[renderer]
pub fn render_res_todo_list(
    todo_list: ResTodoList,
    program_flags: &ResProgramFlags,
) -> RenderResult {
    let mut render_result = RenderResult::new();
    let mut idx = 0;
    writeln!(render_result, "TODO: ").ok();
    for item in &todo_list.items {
        if item.completed && !program_flags.all {
            idx += 1;
            continue;
        }
        writeln!(
            render_result,
            "  {idx}. [{}] - \"{}\"",
            if item.completed { "x" } else { " " },
            item.item
        )
        .ok();
        idx += 1;
    }
    render_result
}

pub fn write_todo_list(todo_list: ResTodoList) {
    let path = get_todo_list_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    let file = std::fs::File::create(path).unwrap();
    let writer = std::io::BufWriter::new(file);
    serde_json::to_writer(writer, &todo_list).unwrap();
}
