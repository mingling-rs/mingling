<h1 align="center">Features</h1>
<p align="center">
    <b>Mingling</b>'s complete feature list
</p>

## Feature `all_serde_fmt`

**Description:**

Enables serde formatting support for all serialization formats (JSON, RON, TOML, YAML) in `structural_renderer`.

Enabling this feature will automatically enable the four sub-features: `json_serde_fmt`, `ron_serde_fmt`, `toml_serde_fmt`, `yaml_serde_fmt`.

## Feature `async`

**Description:**

Enables async runtime support, allowing `#[chain]` to bind `async` functions, e.g.:

```rust
// Features: ["async"]
 
pack!(StateFoo = ());
 
#[chain]
async fn handle_state_foo(foo: StateFoo) -> Next {
    StateFoo::new(()).into()
}
```
 
See [example](https://mingling-rs.github.io/mingling/docs/example-viewer.html?name=example-async-support)

## Feature `builds`

**Description:**

Enables scripts needed for use in `build.rs`, currently including:

1. Completion script generation under the `comp` feature:

```rust
// BUILD TIME
// Features: ["builds", "comp"]
use mingling::build::build_comp_scripts;
 
// Generate completion scripts for `myprogram`
build_comp_scripts("myprogram").unwrap();
```
 
## Feature `clap`

**Description:**

Enables integration with the [clap](https://crates.io/crates/clap) command-line argument parsing library, making it easy to build CLI apps.

With this feature enabled, you can use the `#[dispatcher_clap]` attribute macro to generate a dispatcher from a `clap::Parser` struct.

See [example](https://mingling-rs.github.io/mingling/docs/example-viewer.html?name=example-clap-binding)

## Feature `comp`

**Description:**

Enables code completion functionality, providing auto-completion support for interactive environments.

When enabled, you can use the `#[completion]` attribute macro to define dynamic completion logic, and generate completion scripts for shells such as bash, zsh, fish, and pwsh.

See [example](https://mingling-rs.github.io/mingling/docs/example-viewer.html?name=example-completion)

## Feature `debug`

**Description:**

Enables debugging-related features, providing more detailed error info and diagnostic output.

## Feature `dispatch_tree`

**Description:**

Enables the dispatch tree mechanism, supporting conditional dispatch and routing.

When enabled, Mingling **at compile time** hard-codes the subcommand structure as a prefix tree (Trie), achieving extremely fast subcommand lookup. Lookup complexity is **O(n)**, where _n_ is the input length, not the number of commands.

See [example](https://mingling-rs.github.io/mingling/docs/example-viewer.html?name=example-dispatch-tree)

## Feature `extra_macros`

**Description:**

Enables an additional set of macros, providing more convenient syntactic sugar and metaprogramming capabilities.

For example, allows the shorthand form `dispatcher!("greet")`, which auto-generates `CMDGreet` / `EntryGreet`.

| Macro                                                   | Description                                                     |
| ------------------------------------------------------- | --------------------------------------------------------------- |
| `empty_result!()`                                       | Shorthand for returning an empty result early in a chain        |
| `entry!(Type, ["a", "b"])`                              | Construct test data for an entry type                           |
| `group!(Type)`                                          | Register external types as group members without modifying them |
| `pack_err!(ErrorType)` / `pack_err!(ErrorType = Inner)` | Create error types with an automatic `name` field               |
| `#[program_setup]`                                      | Declare a program initialization function                       |
| `dispatcher!("cmd.path")` **shorthand**                 | Omit `CMDStruct => EntryStruct`, names are auto-derived         |

<details>
<summary> Details </summary>

### `empty_result!()`

```rust
// Features: ["extra_macros"]
 
pack!(StatePrev1 = ());
pack!(StatePrev2 = ());
 
pack!(StateNext = ());
 
#[chain]
fn handle_state_prev2(_p: StatePrev2) {
    // A #[chain] with no return type can simply omit the return value
}
 
#[chain]
fn handle_state_prev1(_p: StatePrev1) -> Next {
    let foo = 1;
    let bar = 2;
    if foo != bar {
        // When Next is needed but no return value is required, use this
        empty_result!()
    } else {
        StateNext::new(()).into()
    }
}
```
 
### `#[program_setup]`

```rust
// Features: ["extra_macros"]
use mingling::{macros::program_setup, Program};
 
fn main() {
    let mut program = ThisProgram::new();
    program.with_setup(NoErrorSetup);
    program.exec_and_exit();
}
 
#[program_setup]
fn no_error_setup(program: &mut Program<ThisProgram>) {
    program.global_flag(["--no-error"], |program| {
        program.stdout_setting.error_output = false;
    });
}
```
 
### `entry!`

```rust
// Features: ["extra_macros"]
use mingling::macros::entry;
 
pack!(EntryHello = Vec<String>);
 
fn main() {
    let result: Next = handle_hello(entry!("--name", "Bob")).into();
    // ... assertion logic here
}
 
#[chain]
fn handle_hello(args: EntryHello) {}
```
 
### `group!`

Registers an external type as a member of the program group without modifying its definition.
The type's simple name is used as the enum variant, just like `pack!` or `#[derive(Groupped)]`.

```rust
// Features: ["extra_macros"]
use mingling::macros::group;
use std::num::ParseIntError;
 
// Register std::num::ParseIntError as a group member.
// After this, ParseIntError can be used in #[chain] and #[renderer] functions.
group!(std::num::ParseIntError);
```
 
### `pack_err!`

Creates an error struct with an automatic `name: String` field set to the snake_case
of the struct name. Optionally wraps an inner type for additional context.

```rust
// Features: ["extra_macros"]
use std::path::PathBuf;
 
// Simple form — only a name field:
pack_err!(ErrorNotFound);
// Generates:
//   struct ErrorNotFound { pub name: String }
//   impl Default for ErrorNotFound { ... }
 
// Typed form — with additional info field:
pack_err!(ErrorNotDir = PathBuf);
// Generates:
//   struct ErrorNotDir { pub name: String, pub info: PathBuf }
//   impl ErrorNotDir { pub fn new(info: PathBuf) -> Self { ... } }
```
 
</details>

## Feature `structural_renderer`

**Description:**

Enables the structural renderer, providing basic content rendering capabilities. Enabling this feature will automatically enable `json_serde_fmt`.

When enabled, users can get structured output via flags like `--json` or `--yaml`.

See [example](https://mingling-rs.github.io/mingling/docs/example-viewer.html?name=example-structural-renderer)

## Feature `structural_renderer_empty`

**Description:**

Enables an empty implementation of the structural renderer, suitable for scenarios where no actual rendering is needed. This feature does not enable any serde formatting backend.

## Feature `structural_renderer_full`

**Description:**

Enables the full implementation of the structural renderer, including all rendering capabilities and serialization format support. Enabling this feature will automatically enable `all_serde_fmt`.

## Feature `json_serde_fmt`

**Description:**

Enables JSON serde serialization/deserialization formatting support.

## Feature `nightly`

**Description:**

Enables experimental features only available in the Nightly Rust compiler.

## Feature `pathf`

> [!IMPORTANT]
>
> This feature is **EXPERIMENTAL**, its API may change in future versions.

**Description:**

Enables the Module Pathfinder, which at build time automatically resolves the full module paths of all Mingling types and generates a `use` statement mapping file for `gen_program!()` to consume.

When enabled, types can be defined in any submodule, and `gen_program!()` can automatically identify them and generate the correct full-path references without requiring manual `use` imports.

```toml
# Cargo.toml
[dependencies.mingling]
features = ["pathf"]
 
[build-dependencies.mingling]
features = ["builds", "pathf"]
```
 
```rust
// BUILD TIME
// Features: ["pathf"]
analyze_and_build_type_mapping().unwrap();
```
 
See [example](https://mingling-rs.github.io/mingling/docs/example-viewer.html?name=example-pathfinder)

## Feature `parser`

**Description:**

Enables the parser module, providing text parsing and analysis capabilities.

When enabled, you can use `Picker` for zero-cost argument extraction, supporting methods like `pick()` and `pick_or()`.

See [example](https://mingling-rs.github.io/mingling/docs/example-viewer.html?name=example-argument-parse)

## Feature `repl`

**Description:**

Enables interactive REPL (Read-Eval-Print Loop) environment support.

When enabled, you can turn your CLI into an interactive shell via `program.exec_repl()`.

See [example](https://mingling-rs.github.io/mingling/docs/example-viewer.html?name=example-repl-basic)

## Feature `ron_serde_fmt`

**Description:**

Enables RON (Rusty Object Notation) serde serialization/deserialization formatting support.

## Feature `toml_serde_fmt`

**Description:**

Enables TOML serde serialization/deserialization formatting support.

## Feature `yaml_serde_fmt`

**Description:**

Enables YAML serde serialization/deserialization formatting support.

<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
