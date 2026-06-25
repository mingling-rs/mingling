<p align="center">
    <a href="https://github.com/mingling-rs/mingling">
        <img alt="Mingling" src="https://github.com/mingling-rs/mingling/raw/main/docs/res/icon2.png" width="50%">
    </a>
</p>
<h1 align="center">Mìng Lìng - 命令</h1>

<p align="center">
    <b>/mɪŋ lɪŋ/</b>
</p>

<p align="center">
    Macro magician in your CLI.
</p>
<p align="center">
	<img src="https://img.shields.io/github/stars/mingling-rs/mingling?style=flat"> 
	<a href="https://crates.io/crates/mingling">
	    <img src="https://img.shields.io/crates/v/mingling?style=flat">
	</a>
	<a href="https://docs.rs/mingling/latest/mingling/">
	    <img src="https://img.shields.io/docsrs/mingling?style=flat">
	</a>	
	<a href="https://mingling-rs.github.io/mingling/">
	    <img src="https://img.shields.io/badge/helpdoc-rewriting-yellow?style=flat">
	</a>
</p>

> [!WARNING]
>
> **Note**: Mingling is still under active development, and its API may change. Feel free to try it out and give us feedback!
> **Hint**: This note will be removed in version `0.5.0`

<h1 align="center">
  🤔 What is Mingling? 🤔 
</h1>

[`Mingling`](https://github.com/mingling-rs/mingling) is a **proc-macro and type system-based** Rust CLI framework, suitable for developing complex command-line programs with numerous subcommands.

> **BTW:** Its name comes from the Chinese Pinyin **"Mìng Lìng"**, meaning **"Command"**. 😄

### Mingling's Core Capabilities

1. **Separation of Concerns, Clear Logic**: Mingling decouples logic by responsibility, helping you organize your CLI program more clearly.
   See example: [Example](https://github.com/mingling-rs/mingling/blob/main/examples/example-basic/src/main.rs)
2. **"All Logic is Functions"**: Execution logic, rendering logic, completion logic, help logic — everything is a function. Just attach the corresponding attribute macro to bind them to your program.
3. **Fully Dynamic Completion System**: With the `comp` feature, you can flexibly implement dynamic completion logic for any subcommand.
   See examples: [Example](https://github.com/mingling-rs/mingling/blob/main/examples/example-completion/src/main.rs)
4. **Lightning-Fast Subcommand Dispatch**: With the `dispatch_tree` feature, Mingling hardens the subcommand structure into a prefix tree at **compile time**, enabling blazing-fast subcommand lookup.
   See examples: [Example](https://github.com/mingling-rs/mingling/blob/main/examples/example-dispatch-tree/src/main.rs)
5. **Lightweight Dependencies, On-Demand Importing**: Minimal core dependencies keep builds fast; enhanced features are imported on demand through fine-grained feature flags.
6. **Structured Output**: Enabling the `structural_renderer` feature adds support for flags like `--json` and `--yaml`, providing structured output capabilities.
   See examples: [Example](https://github.com/mingling-rs/mingling/blob/main/examples/example-structural-renderer/src/main.rs)

<h1 align="center">
    ✍️ Writing with Mingling ✍️
</h1>

### The Big Picture

Mingling organizes your CLI program into three distinct phases:

```
User Input → [Dispatcher] → Entry → [Chain(s)] → Result → [Renderer] → Output
```

The user's raw arguments flow in. A **Dispatcher** picks them up, wraps them into an **Entry** type, and hands it off to a **Chain** function. The chain processes the entry and produces a **Result**. A **Renderer** takes that result and writes it to the terminal.

Everything in this pipeline is a plain Rust function with an attribute macro on top. You never need to manually implement traits or construct boilerplate.

---

### 1. Defining Commands — `dispatcher!`

The entry point for every subcommand is the `dispatcher!` macro. It generates two structs for you: a **Dispatcher** (used to register the command with the program) and an **Entry** (a wrapper around `Vec<String>` that holds the raw arguments).

```rust
use mingling::prelude::*;

//            "command.name"  dispatcher  entry type
//            │               │           │
dispatcher!("greet",        CMDGreet => EntryGreet);

// Nested subcommand: `remote add`
dispatcher!("remote.add",   CMDRemoteAdd => EntryRemoteAdd);
```

Then in `main()`, register the dispatcher with the program:

```rust
use mingling::prelude::*;

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

use mingling::prelude::*;

// Auto-generates CMDGreet / EntryGreet from "greet"
dispatcher!("greet");
```

---

### 2. The Chain — "#[chain]" — Where Logic Lives

The `#[chain]` attribute turns a plain function into an execution step. Think of it as "the logic that transforms one typed value into another."

```rust
use mingling::prelude::*;

dispatcher!("greet", CMDGreet => EntryGreet);

pack!(ResultGreeting = String);

#[chain]
fn handle_greet(args: EntryGreet) -> Next {
    let greeting = args
        .inner
        .first()
        .cloned()
        .unwrap_or_else(|| "World".to_string());
    ResultGreeting::new(greeting)
}
```

Key points:

- The return type is `Next` — a type alias for `ChainProcess<ThisProgram>`.
- You chain results by calling `.to_chain()` on any `pack!`-ed type.
- You can have **multiple chain functions** for the same command, each transforming the data further.
- With the `async` feature, chain functions can be `async fn`.

---

### 3. The Renderer — "#[renderer]" — How Output Works

The `#[renderer]` attribute turns a function into an output handler. It receives the final result of a chain and writes it to the terminal.

```rust
use mingling::prelude::*;

pack!(ResultGreeting = String);

#[renderer]
fn render_greeting(greeting: ResultGreeting) {
    r_println!("Hello, {}!", *greeting);
}
```

Inside a renderer, use `r_print!` / `r_println!` to write to the output buffer. This is not `println!` — it writes into Mingling's internal `RenderResult` buffer, which is flushed at the end of the pipeline.

You can write renderers for **any type** in your program, including error types:

```rust
use mingling::prelude::*;

#[renderer]
fn render_dispatcher_not_found(err: ErrorDispatcherNotFound) {
    r_println!("Command not found: [{}]", err.join(" "));
}
```

---

### 4. Parsing Arguments — The Picker

Mingling provides a **Picker** for zero-cost argument extraction. You use `pick()` or `pick_or()` on an entry to extract typed values, then `unpack()` to get the final tuple.

```rust
// Features: ["parser"]

use mingling::prelude::*;
use mingling::parser::Picker;

dispatcher!("greet", CMDGreet => EntryGreet);
pack!(ResultGreeting = String);

#[chain]
fn handle_greet(args: EntryGreet) -> Next {
    let (name, count) = Picker::new(args.inner)
        .pick::<String>(())                   // positional: first string
        .pick_or::<u8>(["-r", "--repeat"], 1) // optional flag with default
        .unpack();
    ResultGreeting::new(format!("{} x{}", name, count))
}
```

With the `parser` feature, the `AsPicker` trait provides a shorthand directly on entries:

```rust
// Features: ["parser"]

use mingling::prelude::*;

dispatcher!("greet", CMDGreet => EntryGreet);
pack!(ResultGreeting = String);

#[chain]
fn handle(args: EntryGreet) -> Next {
    let (name, count) = args
        .pick::<Option<String>>(())
        .pick_or::<u8>(["-r", "--repeat"], 1)
        .unpack();
    ResultGreeting::new(format!("{} x{}", name.unwrap_or_default(), count))
}
```

For enums, derive `EnumTag` and implement `PickableEnum` to parse enum variants from strings:

```rust
// Features: ["parser", "extra_macros"]

use mingling::prelude::*;
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
    lang
}
```

---

### 5. The Help System — "#[help]"

Help is just another attribute macro. When the user passes `--help` or `-h`, the program skips the normal chain/render pipeline and routes directly to your `#[help]` function.

Enable it by adding `BasicProgramSetup`:

```rust
use mingling::{macros::help, prelude::*, setup::BasicProgramSetup};

dispatcher!("greet", CMDGreet => EntryGreet);

#[help]
fn help_greet(_prev: EntryGreet) {
    r_println!("Usage: greet <NAME>");
    r_println!("Greets the user with the given name.");
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

### 6. Completion — "#[completion]" — Dynamic Shell Completions

With the `comp` feature, Mingling provides a fully dynamic completion system. You write a function that returns `Suggest` based on the current shell context, and Mingling generates the completion scripts for bash, zsh, fish, and pwsh.

```rust
// Features: ["comp", "extra_macros"]

use mingling::{macros::suggest, prelude::*, ShellContext, Suggest};

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
// Features: ["comp", "builds"]

fn main() {
    mingling::build::build_comp_scripts(env!("CARGO_PKG_NAME")).unwrap();
}
```

For enum-based completions, use `suggest_enum!`:

```rust
// Features: ["comp", "extra_macros"]

use mingling::prelude::*;
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

### 7. Error Handling

Mingling doesn't use `?` operator propagation. Instead, errors are just **alternative results** that flow through the same chain/render pipeline. Create error types with `pack!` and route to them with `.to_render()`:

```rust
use mingling::prelude::*;

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
fn render_no_name(_: ErrorNoNameProvided) {
    r_println!("No name provided");
}

#[renderer]
fn render_too_long(len: ErrorNameTooLong) {
    r_println!("Name too long: {} > 10", *len);
}
```

Two built-in fallback types are always available:

- `ErrorDispatcherNotFound` — rendered when no dispatcher matches the input
- `ErrorRendererNotFound` — rendered when no renderer is found for a result type

---

### 8. Resource Injection

Chain and renderer functions can accept **additional parameters** for the program's global state. Resources are singleton values registered with `program.with_resource(...)`.

```rust
// Features: ["parser", "extra_macros"]

use std::path::PathBuf;
use mingling::prelude::*;

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

dispatcher!("current", CMDCurrent => EntryCurrent);

#[derive(Default, Clone)]
struct ResCurrentDir {
    current_dir: std::path::PathBuf,
}

#[renderer]
fn render_current(_: EntryCurrent, current_dir: &ResCurrentDir) {
    r_println!("Current directory: {}", current_dir.current_dir.display());
}
```

---

### 9. Dispatch Tree — Compile-Time Command Trie

As your program grows to dozens or hundreds of subcommands, linear dispatcher lookup becomes slow. Enable the `dispatch_tree` feature to convert the command structure into a **prefix tree (Trie)** at compile time.

```rust
// Features: ["dispatch_tree"]

use mingling::prelude::*;

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

### 10. Clap Binding — Using Clap's Parser

If you prefer clap's powerful argument parsing, use `#[dispatcher_clap]`. It generates a dispatcher from a `clap::Parser` struct.

```rust
// Features: ["clap"]
// Dependencies:
// clap = "4"

use mingling::macros::dispatcher_clap;
use mingling::prelude::*;
use mingling::Groupped;

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
fn render_greet(greet: EntryGreet) {
    r_print!("Hello, ");
    for _ in 0..greet.repeat { r_print!("{}", greet.name); }
    r_println!("!");
}

#[renderer]
fn render_parse_error(err: ErrorGreetParsed) {
    r_println!("{}", *err);
}
```

You can control how clap help is displayed:

```rust
// Features: ["clap"]

use mingling::prelude::*;

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

### 11. REPL Mode

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
    prelude::*,
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

### 12. Hooks — Observing the Pipeline

Mingling provides a `ProgramHook` system for observing every stage of the execution pipeline. Useful for debugging, logging, or telemetry.

```rust
use mingling::{
    hook::{ProgramControlUnit, ProgramHook},
    prelude::*,
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

### 13. Structural Renderer — Structured Output (JSON/YAML)

With the `structural_renderer` feature, users can add `--json` or `--yaml` flags to get structured output instead of human-readable text.

```rust
// Features: ["structural_renderer", "parser"]
// Dependencies:
// serde = "1"

use mingling::{prelude::*, setup::StructuralRendererSetup};
use mingling::Groupped;
use mingling::StructuralData;
use serde::Serialize;

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
fn render_info_result(info: ResultInfo) {
    r_println!("{} is {} years old", info.name, info.age);
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

### 14. Async Support

Enable the `async` feature to use `async fn` inside `#[chain]`:

```rust
// Features: ["async", "parser"]
// Dependencies:
// tokio = { version = "1", features = ["full"] }

use mingling::prelude::*;
use std::time::Duration;

dispatcher!("download", CMDDownload => EntryDownload);
pack!(ResultDownloaded = String);

#[chain]
pub async fn handle_download(args: EntryDownload) -> Next {
    let file = args.pick(()).unpack();
    download_file(file).await
}

async fn download_file(name: String) -> ResultDownloaded {
    tokio::time::sleep(Duration::from_secs(1)).await;
    ResultDownloaded::new(name)
}

#[renderer]
fn render_downloaded(result: ResultDownloaded) {
    r_println!("\"{}\" downloaded.", *result);
}
```

> [!NOTE]
>
> `#[renderer]` functions cannot be async. When `async` is enabled, `program.exec_and_exit().await` returns a Future.

---

### 15. Wrapping Up — `gen_program!()`

At the very end of your crate root (main.rs / lib.rs), call `gen_program!()` to generate the `ThisProgram` struct, the `Next` type alias, and all internal plumbing.

```rust
use mingling::macros::gen_program;

gen_program!();
```

It must be placed **after** all your `dispatcher!`, `pack!`, `#[chain]`, `#[renderer]`, and `#[help]` declarations.

---

### Putting It All Together

Here's a complete, runnable program:

```rust
use mingling::prelude::*;

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
    ResultGreeting::new(greeting)
}

#[renderer]
fn render_greeting(greeting: ResultGreeting) {
    r_println!("Hello, {}!", *greeting);
}

gen_program!();
```

```bash
$ myapp greet
Hello, World!

$ myapp greet Alice
Hello, Alice!
```

<h1 align="center">
    🚀 Getting Started 🚀
</h1>

Add Mingling to your `Cargo.toml`:

```toml
[dependencies.mingling]
version = "0.2.0"
features = []
```

Or use the github version

```toml
[dependencies.mingling]
git = "https://github.com/mingling-rs/mingling.git"
tag = "unreleased"
features = []
```

Or use the [template project](https://github.com/mingling-rs/mingling-template):

```bash
cargo generate --git mingling-rs/mingling-template
```

Then check out:

- 📖 [Mingling Helpdoc](https://mingling-rs.github.io/mingling/)
- 📖 [Examples on docs.rs](https://docs.rs/mingling/latest/mingling/_mingling_examples/index.html)
- 📖 [Full API docs](https://docs.rs/mingling/latest/mingling/)

<h1 align="center">
    🗺️ Roadmap 🗺️
</h1>

- [ ] Milestone.1 "MVP"
  - [x] \[[0.1.4](https://docs.rs/mingling/0.1.4/mingling/)\] \[`core`\] \[`structural_renderer`\] **Mingling** can render data into serializable formats via `--json` and `--yaml` flags
  - [x] \[[0.1.5](https://docs.rs/mingling/0.1.5/mingling/)\] \[`core`\] \[`comp`\] **Mingling** can dynamically invoke itself to provide completions for shells like `bash`, `zsh`, `fish`, and `pwsh`
  - [x] \[[0.1.6](https://docs.rs/mingling/0.1.6/mingling/)\] \[`core`\] \[`comp`\] **Mingling** can gather more context for smarter completions
  - [x] \[[0.1.7](https://docs.rs/mingling/0.1.7/mingling/)\] \[`clap`\] Provides a **Clap** compatibility layer, allowing **Mingling** to reuse its powerful parsing capabilities
  - [x] \[[0.1.7](https://docs.rs/mingling/0.1.7/mingling/)\] \[`core`\] **Mingling** can intercept `-h` or `--help` flags to display custom help text for each subcommand
  - [x] \[[0.1.7](https://docs.rs/mingling/0.1.7/mingling/)\] \[`mling`\] Provides a basic scaffolding tool (`mling`) for rapid development and debugging
  - [x] \[[0.1.8](https://docs.rs/mingling/0.1.8/mingling/)\] \[`core`\] \[`dispatch_tree`\] Converts the subcommand list into a prefix tree to improve command matching speed
  - [x] \[[0.1.9](https://docs.rs/mingling/0.1.9/mingling/)\] \[`core`\] \[`dev_toolkits`\] Provides debugging interfaces for developers to capture invocation information when issues arise (`InvokeStackDisplay`) (indirectly implemented via `ProgramHook`)
  - [x] \[[0.1.9](https://docs.rs/mingling/0.1.9/mingling/)\] \[`core`\] \[`repl`\] Provides REPL capability (`program.exec_repl();`)
  - [ ] \[**0.2.0**\] Complete documentation, tests, and examples

- [ ] Milestone.2 "More Comfortable Dev and User Experience"
  - [ ] \[**0.2.1**\] \[`macros`\] `r_println!` in `#[chain]` support.
  - [ ] \[**0.2.5**\] \[`mling`\] Helpdoc Maker
  - [ ] \[**0.2.8**\] \[`picker`\] A more efficient and intelligent argument parser

- [ ] Milestone.3 "Unplanned"
  - [ ] ...

<h1 align="center">
    🚫 Unplanned Features 🚫
</h1>

While Mingling has several common CLI features that are **NOT PLANNED** to be directly included in the framework.
This is because the Rust ecosystem already has excellent and mature crates to handle these issues, and Mingling's design is intended to be used in combination with them.

- **Colored Output**: To add color and styles (bold, italic, etc.) to terminal output, consider using crates like [`colored`](https://crates.io/crates/colored) or [`owo-colors`](https://crates.io/crates/owo-colors). You can integrate their types directly into your renderers.
- **I18n**: To translate your CLI application, the [`rust-i18n`](https://crates.io/crates/rust-i18n) crate provides a powerful internationalization solution that you can use in your command logic and renderers.
- **Progress Bars**: To display progress indicators, the [`indicatif`](https://crates.io/crates/indicatif) crate is the standard choice.
- **TUI**: To build full-screen interactive terminal applications, it is recommended to use a framework like [`ratatui`](https://crates.io/crates/ratatui) (formerly `tui-rs`).

<h1 align="center">
    📄 Open Source License 📄
</h1>

This project is licensed under the MIT License.

See [LICENSE-MIT](LICENSE-MIT) or [LICENSE-APACHE](LICENSE-APACHE) file for details.
