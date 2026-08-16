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
    EnumTag, Grouped, ShellContext, Suggest,
    macros::suggest_enum,
    picker::{PickerArgResult, SinglePickable},
    prelude::*,
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

    #[enum_desc("An object-oriented scripting language, famous for its concise and elegant syntax")]
    Ruby,

    #[default]
    #[enum_desc("A systems programming language focused on performance, safety, and concurrency")]
    Rust,
}

// --------- IMPORTANT ---------
// NOTE: Due to the migration from the legacy `parser` to `picker`, the `EnumTag` -> `Picker` path
// is not yet complete, so a manual implementation is used for now.
// Once that path is complete, `#[derive(EnumTag)]` can automatically implement `SinglePickable`,
// replacing this manual implementation.
impl SinglePickable for ProgrammingLanguages {
    fn pick_single(str: Option<&str>) -> PickerArgResult<Self> {
        let Some(str) = str else {
            return PickerArgResult::NotFound;
        };
        let lang = match str.to_lowercase().as_str() {
            "c" => Self::C,
            "c++" | "cpp" => Self::CPlusPlus,
            "c#" | "csharp" => Self::Csharp,
            "java" => Self::Java,
            "javascript" | "js" => Self::JavaScript,
            "kotlin" => Self::Kotlin,
            "ocaml" => Self::OCaml,
            "python" => Self::Python,
            "ruby" => Self::Ruby,
            "rust" => Self::Rust,
            _ => return PickerArgResult::NotFound,
        };
        PickerArgResult::Parsed(lang)
    }
}
// --------- IMPORTANT ---------

dispatcher!("lang-select", EntryLanguageSelection);

#[chain]
fn handle_language_selection(args: EntryLanguageSelection) -> Next {
    // You can use Picker to directly parse ProgrammingLanguages
    let lang: ProgrammingLanguages = args.pick_or_default(&arg![ProgrammingLanguages]).unwrap();
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
    ThisProgram::new().exec_and_exit();
}
