//! Example Enum Tag
//!
//! > This example demonstrates how to use the `EnumTag` derive macro to tag enum variants with metadata,
//! > which can be used for autocompletion and parsing
//!
//! Run:
//! ```bash
//! cargo run --manifest-path examples/example-enum-tag/Cargo.toml --quiet -- lang-select OCaml
//! cargo run --manifest-path examples/example-enum-tag/Cargo.toml --quiet -- lang-select
//! ```
//!
//! Output:
//! ```plaintext
//! Selected: OCaml (A representative functional programming language with strong type inference)
//! Selected: Rust (A systems programming language focused on performance, safety, and concurrency)
//! ```

use mingling::{
    macros::suggest_enum, parser::PickableEnum, prelude::*, EnumTag, Grouped, ShellContext,
    Suggest,
};
use std::io::Write;

// Define the enum and derive the EnumTag trait
//                        ________ adds metadata to the enum, enabling it to:
//                       /         1. Be used by the `suggest_enum!(Enum)` macro under the `comp` feature for autocompletion
//                       vvvvvvv   2. Implement the `PickableEnum` trait
#[derive(Debug, Default, EnumTag, Grouped)]
pub enum ProgrammingLanguages {
    #[enum_desc("An efficient and flexible compiled language widely used for system programming")]
    C,

    #[enum_rename("C++")]
    #[enum_desc("A high-performance language extending C with object-oriented features")]
    CPlusPlus,

    #[enum_rename("C#")]
    #[enum_desc("Microsoft's object-oriented programming language running on the .NET platform")]
    Csharp,

    #[enum_desc(
        "A cross-platform object-oriented language widely used for enterprise application development"
    )]
    Java,

    #[enum_desc(
        "A dynamic scripting language for web development, supporting prototype chain inheritance"
    )]
    JavaScript,

    #[enum_desc("A modern statically typed language running on the JVM, concise and safe")]
    Kotlin,

    #[enum_desc("A representative functional programming language with strong type inference")]
    OCaml,

    #[enum_desc("A general-purpose programming language with clean syntax, known for readability")]
    Python,

    #[enum_desc(
        "An object-oriented scripting language, famous for its concise and elegant syntax"
    )]
    Ruby,

    #[default]
    #[enum_desc("A systems programming language focused on performance, safety, and concurrency")]
    Rust,
}

// --------- IMPORTANT ---------
// Implement the PickableEnum trait for ProgrammingLanguages,
// so that `Picker` can parse this enum
impl PickableEnum for ProgrammingLanguages {}
// --------- IMPORTANT ---------

dispatcher!("lang-select", CMDLanguageSelection => EntryLanguageSelection);

#[chain]
fn handle_language_selection(args: EntryLanguageSelection) -> Next {
    // You can use Picker to directly parse ProgrammingLanguages
    let lang: ProgrammingLanguages = args.pick(()).unpack();
    lang.into()
}

/// Renders the selected programming language with its name and description.
#[renderer]
pub fn render_programming_language(lang: ProgrammingLanguages) -> RenderResult {
    let mut render_result = RenderResult::new();
    let (name, desc) = lang.enum_info();
    writeln!(render_result, "Selected: {} ({})", name, desc).ok();
    render_result
}

#[completion(EntryLanguageSelection)]
fn complete_language_selection(_: &ShellContext) -> Suggest {
    // Use `suggest_enum!` directly to generate enum suggestions
    suggest_enum!(ProgrammingLanguages)
}

gen_program!();

fn main() {
    let mut program = ThisProgram::new();
    program.with_dispatcher(CMDCompletion);
    program.with_dispatcher(CMDLanguageSelection);
    program.exec_and_exit();
}
