<h1 align="center">Features</h1>
<p align="center">
    <b>Mingling</b>'s complete feature list
</p>

# Preset Feature Groups

Mingling provides a set of **preset feature groups** that make it easy to enable features in whatever combination you need.

## `mini`

**Enables features:** `picker`, `comp`

**Positioning:** Minimal mode, suitable for small CLI tools or projects that need to get started quickly. Includes only the most essential argument parsing and code completion capabilities.

## `advanced`

**Enables features:** `picker`, `repl`, `comp`, `structural_renderer`, `pathf`

**Positioning:** Advanced mode, builds on `mini` by adding an interactive REPL environment, structured output capabilities, and the experimental path analyzer. Suitable for medium-sized command-line applications with a fuller feature set.

## `full`

**Enables features:** `picker`, `repl`, `clap`, `comp`, `structural_renderer_full`

**Positioning:** Full mode, enables all of Mingling's core functionality. In addition to `advanced`, it includes clap integration and the full structural renderer (with all serialization formats). Suitable for large, feature-complete command-line applications.

# Feature Details

## Feature `all_serde_fmt`

**Description:**

Enables serde formatting support for all serialization formats (JSON, RON, TOML, YAML) in `structural_renderer`.

Enabling this feature will automatically enable the four sub-features: `json_serde_fmt`, `ron_serde_fmt`, `toml_serde_fmt`, `yaml_serde_fmt`.

## Feature `async`

**Description:**

Enables async runtime support, allowing `#[chain]` to bind `async` functions, e.g.:

```rust
// Features: ["async"]
 
#[derive(Grouped, Wrap)]
pub struct StateFoo(());
 
#[chain]
async fn handle_state_foo(foo: StateFoo) -> Next {
    StateFoo(()).into()
}
```
 
See [example](https://mingling-rs.github.io/mingling/docs/example-viewer.html?name=example-async-support)

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
```
 
With the `pathf` feature enabled, `gen_program!()` automatically invokes `build_pathf!()` at compile time to run the type mapping analysis.

See [example](https://mingling-rs.github.io/mingling/docs/example-viewer.html?name=example-pathfinder)

## Feature `picker`

**Description:**

Introduces the `arg-picker` dependency, providing advanced argument parsing capabilities for Mingling.

`picker` is an argument parser independent of Mingling and does not rely on the built-in argument extraction API of `mingling_core`.

See [example](https://mingling-rs.github.io/mingling/docs/example-viewer.html?name=example-argument-picker)

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
