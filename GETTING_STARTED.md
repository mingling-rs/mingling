# Writing with Mingling

## The Big Picture

Mingling organizes your CLI program into three distinct phases:

```
Input → [Dispatcher] → Entry → [Chain(s)] → Result → [Renderer] → Output
```

**Step 1: Input** — The user's raw arguments flow in.
**Step 2: Dispatch** — The **Dispatcher** receives them and wraps them into an **Entry** type.
**Step 3: Chain** — The Entry is handed to a **Chain** function for processing.
**Step 4: Render** — The **Renderer** receives the result and writes it to the terminal.

> [!NOTE]
> A Chain can produce a **State** type, passed to the next Chain for further processing,
>
> or it can produce a **Result** type, handed to the Renderer for output.

Every step in this pipeline is a **pure function** annotated with an attribute macro, and the framework recognizes them as long as they meet the requirements.

---

## 1. Defining Commands — `dispatcher!`

The entry point for every subcommand is the `dispatcher!` macro. It generates two structs for you: a **Dispatcher** (used to register the command with the program) and an **Entry** (a wrapper around `Vec<String>` that holds the raw arguments).

```rust
use mingling::prelude::*;

//           command.name   Dispatcher  EntryType
//           │              │           │
dispatcher!("greet",        CMDGreet => EntryGreet);

// Nested subcommand: `remote add`
dispatcher!("remote.add",   CMDRemoteAdd => EntryRemoteAdd);
```

Then in `main()`, register the dispatcher with the program:

```rust
dispatcher!("greet", CMDGreet => EntryGreet);

fn main() {
    let mut program = ThisProgram::new();
    program.with_dispatcher(CMDGreet);
    program.exec_and_exit();
}
```

Mingling also supports an abbreviated form (with the `extra_macros` feature):

```rust
// Features: ["extra_macros"]

// Auto-generates CMDGreet / EntryGreet from "greet"
dispatcher!("greet");
```

---

## 2. The Chain — "#[chain]" — Where Logic Lives

The `#[chain]` attribute turns a plain function into an execution step. Think of it as "the logic that transforms one typed value into another."

```rust
dispatcher!("greet", CMDGreet => EntryGreet);

pack!(ResultGreeting = String);

#[chain]
fn handle_greet(args: EntryGreet) -> Next {
    let greeting = args
        .inner
        .first()
        .cloned()
        .unwrap_or_else(|| "World".to_string());
    ResultGreeting::new(greeting).into()
}
```

Key points:

- The return type is `Next` — a type alias for `ChainProcess<ThisProgram>`.
- You chain results by calling `.to_chain()` on any `pack!`-ed type.
- You can have **multiple chain functions** for the same command, each transforming the data further.
- With the `async` feature, chain functions can be `async fn`.

---

## 3. The Renderer — "#[renderer]" — How Output Works

The `#[renderer]` attribute turns a function into an output handler. It receives the final result of a chain and returns a `RenderResult`.

```rust
use mingling::macros::pack;
use mingling::prelude::*;
use std::io::Write;

pack!(ResultGreeting = String);

#[renderer]
fn render_greeting(greeting: ResultGreeting) -> RenderResult {
    let mut result = RenderResult::new();
    writeln!(result, "Hello, {}!", *greeting).ok();
    result
}
```

Inside a renderer, create a `RenderResult`, write to it using `write!` / `writeln!` (from [`std::io::Write`](https://doc.rust-lang.org/std/io/trait.Write.html)), and return it. The output is captured in the buffer and flushed by the framework at the end of the pipeline.

You can write renderers for **any type** in your program, including error types:

```rust
use mingling::prelude::*;
use std::io::Write;

#[renderer]
fn render_dispatcher_not_found(err: ErrorDispatcherNotFound) -> RenderResult {
    let mut result = RenderResult::new();
    writeln!(result, "Command not found: [{}]", err.join(" ")).ok();
    result
}
```

---

## 4. Parsing Arguments — The Picker

Mingling provides a **Picker** for zero-cost argument extraction. You use `pick()` or `pick_or()` on an entry to extract typed values, then `unpack()` to get the final tuple.

```rust
// Features: ["parser"]

use mingling::parser::Picker;

dispatcher!("greet", CMDGreet => EntryGreet);
pack!(ResultGreeting = String);

#[chain]
fn handle_greet(args: EntryGreet) -> Next {
    let (name, count) = Picker::new(args.inner)
        .pick::<String>(())                   // positional: first string
        .pick_or::<u8>(["-r", "--repeat"], 1) // optional flag with default
        .unpack();
    ResultGreeting::new(format!("{} x{}", name, count)).into()
}
```

With the `parser` feature, the `AsPicker` trait provides a shorthand directly on entries:

```rust
// Features: ["parser"]

dispatcher!("greet", CMDGreet => EntryGreet);
pack!(ResultGreeting = String);

#[chain]
fn handle(args: EntryGreet) -> Next {
    let (name, count) = args
        .pick::<Option<String>>(())
        .pick_or::<u8>(["-r", "--repeat"], 1)
        .unpack();
    ResultGreeting::new(format!("{} x{}", name.unwrap_or_default(), count)).into()
}
```

For enums, derive `EnumTag` and implement `PickableEnum` to parse enum variants from strings:

```rust
// Features: ["parser", "extra_macros"]

use mingling::{EnumTag, Groupped};
use mingling::parser::PickableEnum;

dispatcher!("lang.select", CMDLang => EntryLang);

#[derive(Debug, Default, EnumTag, Groupped)]
pub enum Language {
    #[default]
    Rust,
    #[enum_rename("C++")]
    CPlusPlus,
}

impl PickableEnum for Language {}

#[chain]
fn handle(args: EntryLang) -> Next {
    let lang: Language = args.pick(()).unpack();
    lang.into()
}
```

---

## 5. The Help System — "#[help]"

Help is just another attribute macro. When the user passes `--help` or `-h`, the program skips the normal chain/render pipeline and routes directly to your `#[help]` function.

Enable it by adding `BasicProgramSetup`:

```rust
use mingling::{macros::help, prelude::*, setup::BasicProgramSetup};
use std::io::Write;

dispatcher!("greet", CMDGreet => EntryGreet);

#[help]
fn help_greet(_prev: EntryGreet) -> RenderResult {
    let mut result = RenderResult::new();
    writeln!(result, "Usage: greet <NAME>").ok();
    writeln!(result, "Greets the user with the given name.").ok();
    result
}

fn main() {
    let mut program = ThisProgram::new();
    program.with_setup(BasicProgramSetup);  // enables --help / -h
    program.with_dispatcher(CMDGreet);
    program.exec_and_exit();
}

gen_program!();
```

The flow is:

- User types `greet --help`
- `BasicProgramSetup` sets `program.user_context.help = true`
- The dispatcher sees this flag and routes to the `#[help]` function instead of the `#[chain]`

---

## 6. Completion — "#[completion]" — Dynamic Shell Completions

With the `comp` feature, Mingling provides a fully dynamic completion system. You write a function that returns `Suggest` based on the current shell context, and Mingling generates the completion scripts for bash, zsh, fish, and pwsh.

```rust
// Features: ["comp", "extra_macros"]

use mingling::{macros::suggest, ShellContext, Suggest};

dispatcher!("greet", CMDGreet => EntryGreet);
pack!(ResultName = (u8, String));

#[completion(EntryGreet)]
fn complete_greet(ctx: &ShellContext) -> Suggest {
    // Suggest positional arguments
    if ctx.previous_word == "greet" {
        return suggest! {
            "Alice": "Likes to receive messages",
            "Bob":   "Likes to pass messages",
            "World"
        };
    }

    // Suggest flag arguments
    if ctx.typing_argument() {
        return suggest! {
            "-r":        "Number of repetitions",
            "--repeat":  "Number of repetitions",
        }
        .strip_typed_argument(ctx);
    }

    suggest!()  // no suggestions
}
```

You also need to register the built-in completion dispatcher:

```rust
// Features: ["comp"]

fn main() {
    let mut program = ThisProgram::new();
    program.with_dispatcher(crate::CMDCompletion);
    program.exec_and_exit();
}
```

In your `build.rs`, generate the shell scripts:

```rust
// BUILD TIME
// Features: ["comp", "builds"]
mingling::build::build_comp_scripts(env!("CARGO_PKG_NAME")).unwrap();
```

For enum-based completions, use `suggest_enum!`:

```rust
// Features: ["comp", "extra_macros"]

use mingling::{ShellContext, Suggest};
use mingling::macros::suggest_enum;
use mingling::EnumTag;

dispatcher!("lang.select", CMDLang => EntryLang);

#[derive(EnumTag)]
pub enum ProgrammingLanguages {
    Rust,
    Python,
    JavaScript,
}

#[completion(EntryLang)]
fn complete_lang(_: &ShellContext) -> Suggest {
    suggest_enum!(ProgrammingLanguages)
}
```

---

## 7. Error Handling

Mingling doesn't use `?` operator propagation. Instead, errors are just **alternative results** that flow through the same chain/render pipeline. Create error types with `pack!` and route to them with `.to_render()`:

```rust
use mingling::macros::pack;
use mingling::prelude::*;
use std::io::Write;

dispatcher!("hello", CMDHello => EntryHello);
pack!(ResultName = String);
pack!(ErrorNoNameProvided = ());
pack!(ErrorNameTooLong = u16);

#[chain]
fn handle(args: EntryHello) -> Next {
    let Some(name) = args.inner.first().cloned() else {
        return ErrorNoNameProvided::default().to_render();  // ← early return to error renderer
    };

    if name.len() > 10 {
        return ErrorNameTooLong::new(name.len() as u16).to_render();
    }

    ResultName::new(name).to_render()  // ← success path
}

#[renderer]
fn render_no_name(_: ErrorNoNameProvided) -> RenderResult {
    let mut result = RenderResult::new();
    writeln!(result, "No name provided").ok();
    result
}

#[renderer]
fn render_too_long(len: ErrorNameTooLong) -> RenderResult {
    let mut result = RenderResult::new();
    writeln!(result, "Name too long: {} > 10", *len).ok();
    result
}
```

Two built-in fallback types are always available:

- `ErrorDispatcherNotFound` — rendered when no dispatcher matches the input
- `ErrorRendererNotFound` — rendered when no renderer is found for a result type

---

## 8. Resource Injection

Chain and renderer functions can accept **additional parameters** for the program's global state. Resources are singleton values registered with `program.with_resource(...)`.

```rust
// Features: ["parser", "extra_macros"]

use std::path::PathBuf;

dispatcher!("current", CMDCurrent => EntryCurrent);
dispatcher!("cd", CMDCd => EntryCd);

#[derive(Default, Clone)]
struct ResCurrentDir {
    current_dir: PathBuf,
}

fn main() {
    let mut program = ThisProgram::new();
    program.with_resource(ResCurrentDir {
        current_dir: std::env::current_dir().unwrap(),
    });
    program.with_dispatcher(CMDCurrent);
    program.with_dispatcher(CMDCd);
    program.exec_and_exit();
}

// Read-only access (shared reference):
#[chain]
fn show_current(_prev: EntryCurrent, current_dir: &ResCurrentDir) -> Next {
    println!("Current: {}", current_dir.current_dir.display());
    empty_result!()
}

// Mutable access:
#[chain]
fn change_dir(prev: EntryCd, current_dir: &mut ResCurrentDir) -> Next {
    let path: String = prev.pick(()).unpack();
    current_dir.current_dir = current_dir.current_dir.join(path);
    empty_result!()
}
```

Resources can also be injected into `#[renderer]`:

```rust
use mingling::prelude::*;
use std::io::Write;

dispatcher!("current", CMDCurrent => EntryCurrent);

#[derive(Default, Clone)]
struct ResCurrentDir {
    current_dir: std::path::PathBuf,
}

#[renderer]
fn render_current(_: EntryCurrent, current_dir: &ResCurrentDir) -> RenderResult {
    let mut result = RenderResult::new();
    writeln!(result, "Current directory: {}", current_dir.current_dir.display()).ok();
    result
}
```

---

## 9. Dispatch Tree — Compile-Time Command Trie

As your program grows to dozens or hundreds of subcommands, linear dispatcher lookup becomes slow. Enable the `dispatch_tree` feature to convert the command structure into a **prefix tree (Trie)** at compile time.

```rust
// Features: ["dispatch_tree"]

dispatcher!("cmd1",              CMD1 => Entry1);
dispatcher!("cmd2.sub1",         CMD2Sub1 => Entry2Sub1);
dispatcher!("cmd2.sub2",         CMD2Sub2 => Entry2Sub2);
dispatcher!("cmd3.sub1.leaf1",   CMD3Sub1Leaf1 => Entry3Sub1Leaf1);
dispatcher!("cmd3.sub1.leaf2",   CMD3Sub1Leaf2 => Entry3Sub1Leaf2);
// ... dozens more

fn main() {
    let program = ThisProgram::new();
    // No more with_dispatcher calls — it's all compile-time!
    program.exec_and_exit();
}
```

With `dispatch_tree` enabled:

- Dispatchers are auto-collected at compile time
- `Program` no longer stores a dispatcher list
- `program.with_dispatcher(...)` is not compiled
- Lookup is **O(n)** where _n_ is input length, not number of commands

---

## 10. Clap Binding — Using Clap's Parser

If you prefer clap's powerful argument parsing, use `#[dispatcher_clap]`. It generates a dispatcher from a `clap::Parser` struct.

```rust
// Features: ["clap"]
// Dependencies:
// clap = "4"

use mingling::macros::dispatcher_clap;
use mingling::prelude::*;
use std::io::Write;

#[derive(Default, clap::Parser, Groupped)]
#[dispatcher_clap(
    "greet", CMDGreet,
    help = true,              // auto-generate #[help] from clap
    error = ErrorGreetParsed, // capture parse errors as a renderable type
)]
pub struct EntryGreet {
    #[clap(default_value = "World")]
    name: String,

    #[arg(short, long, default_value_t = 1)]
    repeat: i32,
}

#[renderer]
fn render_greet(greet: EntryGreet) -> RenderResult {
    let mut result = RenderResult::new();
    write!(result, "Hello, ").ok();
    for _ in 0..greet.repeat { write!(result, "{}", greet.name).ok(); }
    writeln!(result, "!").ok();
    result
}

#[renderer]
fn render_parse_error(err: ErrorGreetParsed) -> RenderResult {
    let mut result = RenderResult::new();
    writeln!(result, "{}", *err).ok();
    result
}
```

You can control how clap help is displayed:

```rust
// Features: ["clap"]

dispatcher!("greet", CMDGreet => EntryGreet);

fn main() {
    let mut program = ThisProgram::new();
    program.with_dispatcher(CMDGreet);
    program.stdout_setting.clap_help_print_behaviour =
        mingling::ClapHelpPrintBehaviour::WriteToRenderResult;
    // or: PrintDirectly — writes clap help straight to stdout
    program.exec_and_exit();
}
```

---

## 11. REPL Mode

With the `repl` feature, turn your CLI into an interactive shell with one method call:

```rust
// Features: ["repl"]

fn main() {
    ThisProgram::new().exec_repl();
}
```

Mingling provides built-in REPL setups:

```rust
// Features: ["repl", "extra_macros"]

use mingling::{
    res::ResREPL,
    setup::{BasicREPLReadlineSetup, BasicREPLOutputSetup, BasicREPLPromptSetup},
};

dispatcher!("cd", CMDCd => EntryCd);
dispatcher!("exit", CMDExit => EntryExit);

fn main() {
    let mut program = ThisProgram::new();

    program.with_dispatcher(CMDCd);
    program.with_dispatcher(CMDExit);

    // Enable line reading from stdin
    program.with_setup(BasicREPLReadlineSetup);

    // Enable output flushing after each render
    program.with_setup(BasicREPLOutputSetup);

    // Custom prompt
    program.with_setup(BasicREPLPromptSetup::func(|| "> ".to_string()));

    program.exec_repl();  // ← interactive loop
}

// Exit the REPL via the ResREPL resource:
#[chain]
fn handle_exit(_prev: EntryExit, repl: &mut ResREPL) {
    repl.exit = true;
}
```

---

## 12. Hooks — Observing the Pipeline

Mingling provides a `ProgramHook` system for observing every stage of the execution pipeline. Useful for debugging, logging, or telemetry.

```rust
use mingling::{
    hook::{ProgramControlUnit, ProgramHook},
};

dispatcher!("greet", CMDGreet => EntryGreet);

fn main() {
    let mut program = ThisProgram::new();

    program.with_hook(
        ProgramHook::<ThisProgram>::empty()
            .on_begin::<_, ()>(|_| println!("[DEBUG] Program is begin"))
            .on_pre_dispatch(|info| println!("[DEBUG] Pre dispatch: {}", info.arguments.join(" ")))
            .on_post_dispatch(|info| println!("[DEBUG] Post dispatch: {}", info.entry))
            .on_pre_chain(|info| {
                println!("[DEBUG] Pre chain: {}", info.input);
            })
            .on_post_chain(|info| println!("[DEBUG] Post chain: {}", info.output.member_id))
            .on_finish(|_| {
                println!("[DEBUG] Loop end");
                ProgramControlUnit::OverrideExitCode(0) // Override exit code
            })
            .on_pre_render(|info| println!("[DEBUG] Pre render: {}", info.input))
            .on_post_render(|_| println!("[DEBUG] Post render")),
    );

    program.with_dispatcher(CMDGreet);
    program.exec_and_exit();
}
```

---

## 13. Structural Renderer — Structured Output (JSON/YAML)

With the `structural_renderer` feature, users can add `--json` or `--yaml` flags to get structured output instead of human-readable text.

```rust
// Features: ["structural_renderer", "parser"]
// Dependencies:
// serde = "1"

use mingling::{prelude::*, setup::StructuralRendererSetup};
use mingling::Groupped;
use mingling::StructuralData;
use serde::Serialize;
use std::io::Write;

dispatcher!("render", CMDRender => EntryRender);

#[derive(Default, StructuralData, Serialize, Groupped)]
struct ResultInfo {
    name: String,
    age: i32,
}

#[chain]
fn render_info(args: EntryRender) -> Next {
    let (name, age) = args.pick::<String>(()).pick::<i32>(()).unpack();
    ResultInfo { name, age }.to_chain()
}

#[renderer]
fn render_info_result(info: ResultInfo) -> RenderResult {
    let mut result = RenderResult::new();
    writeln!(result, "{} is {} years old", info.name, info.age).ok();
    result
}

fn main() {
    let mut program = ThisProgram::new();
    program.with_setup(StructuralRendererSetup);  // enables --json / --yaml
    program.with_dispatcher(CMDRender);
    let _ = program.exec();
}
```

Then users can do:

```bash
$ myapp render Bob 22
Bob is 22 years old

$ myapp render Bob 22 --json
{"name":"Bob","age":22}

$ myapp render Bob 22 --yaml
name: Bob
age: 22
```

---

## 14. Async Support

Enable the `async` feature to use `async fn` inside `#[chain]`:

```rust
// Features: ["async", "parser"]
// Dependencies:
// tokio = { version = "1", features = ["full"] }

use std::io::Write;
use std::time::Duration;

dispatcher!("download", CMDDownload => EntryDownload);
pack!(ResultDownloaded = String);

#[chain]
pub async fn handle_download(args: EntryDownload) -> Next {
    let file = args.pick(()).unpack();
    download_file(file).await.into()
}

async fn download_file(name: String) -> ResultDownloaded {
    tokio::time::sleep(Duration::from_secs(1)).await;
    ResultDownloaded::new(name)
}

#[renderer]
fn render_downloaded(result: ResultDownloaded) -> RenderResult {
    let mut r = RenderResult::new();
    writeln!(r, "\"{}\" downloaded.", *result).ok();
    r
}
```

> [!NOTE]
>
> `#[renderer]` functions cannot be async. When `async` is enabled, `program.exec_and_exit().await` returns a Future.

---

## 15. Wrapping Up — `gen_program!()`

At the very end of your crate root (main.rs / lib.rs), call `gen_program!()` to generate the `ThisProgram` struct, the `Next` type alias, and all internal plumbing.

```rust
use mingling::macros::gen_program;

gen_program!();
```

It must be placed **after** all your `dispatcher!`, `pack!`, `#[chain]`, `#[renderer]`, and `#[help]` declarations.

---

## Putting It All Together

Here's a complete, runnable program:

```rust
use mingling::macros::pack;
use mingling::prelude::*;
use std::io::Write;

dispatcher!("greet", CMDGreet => EntryGreet);

fn main() {
    let mut program = ThisProgram::new();
    program.with_dispatcher(CMDGreet);
    program.exec_and_exit();
}

pack!(ResultGreeting = String);

#[chain]
fn handle_greet(args: EntryGreet) -> Next {
    let greeting = args
        .inner
        .first()
        .cloned()
        .unwrap_or_else(|| "World".to_string());
    ResultGreeting::new(greeting).into()
}

#[renderer]
fn render_greeting(greeting: ResultGreeting) -> RenderResult {
    let mut result = RenderResult::new();
    writeln!(result, "Hello, {}!", *greeting).ok();
    result
}

gen_program!();
```

```bash
$ myapp greet
Hello, World!

$ myapp greet Alice
Hello, Alice!
```
