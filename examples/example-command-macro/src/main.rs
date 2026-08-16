//! Example Command Macro
//!
//! > Introduced how to use the `#[command]` macro to generate commands with minimal boilerplate
//!
//! Run:
//! ```base
//! cargo run --manifest-path examples/example-command-macro/Cargo.toml --quiet -- hello world
//! cargo run --manifest-path examples/example-command-macro/Cargo.toml --quiet -- greet-someone Alice
//! cargo run --manifest-path examples/example-command-macro/Cargo.toml --quiet -- goodbye
//! ```
//!
//! Output:
//! ```plaintext
//! Hello, World
//! Hello, Alice
//! Goodbye!
//! ```

use mingling::{macros::buffer, picker::IntoPicker, prelude::*};

fn main() {
    ThisProgram::new().exec_and_exit();
}

#[derive(Grouped, Wrap)]
pub struct ResultGreeting(String);

#[derive(Grouped)]
pub struct ResultGoodbye;

// --------- IMPORTANT ---------
// Auto-generates dispatcher!("hello.world", EntryHelloWorld);
#[command]
fn hello_world() -> ResultGreeting {
    ResultGreeting("World".to_string())
}

// Auto-generates dispatcher!("hello-world", EntryGreetSomeone);
#[command(node = "greet-someone")]
fn greet_someone(args: Vec<String>) -> ResultGreeting {
    let name = args.pick_or(&arg![String], || "World".to_string()).unwrap();
    ResultGreeting(name)
}

// Auto-generates dispatcher!("goodbye", EntryGoodBye);
#[command(entry = EntryGoodBye)]
fn goodbye() -> ResultGoodbye {
    ResultGoodbye
}
// --------- IMPORTANT ---------

#[renderer(buffer)]
fn render_greeting(result: ResultGreeting) {
    r_println!("Hello, {}", *result);
}

#[renderer(buffer)]
fn render_goodbye(_: ResultGoodbye) {
    r_println!("Goodbye!");
}

gen_program!();
