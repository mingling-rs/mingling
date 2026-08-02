<h1 align="center">Mìng Lìng - 命令</h1>

<p align="center">
    <b>/mɪŋ lɪŋ/</b>
</p>

<p align="center">
    Macro magician in your CLI.
</p>

## Intro

[`Mingling`](https://github.com/mingling-rs/mingling) is a **state-driven and data-driven** CLI workflow orchestration framework built in Rust.

## Use

Here is a basic project written using **Mingling**:

- When the user types `greet`, the program outputs `Hello, World!`
- When the user types `greet Alice`, the program outputs `Hello, Alice!`

```rust
use mingling::prelude::*;

dispatcher!("greet", CMDGreet => EntryGreet);

fn main() {
    let mut program = ThisProgram::new();
    program.with_dispatcher(CMDGreet);
    program.exec_and_exit();
}

pack!(ResultName = String);

#[chain]
fn handle_greet(args: EntryGreet) -> Next {
    let name: ResultName = args
        .inner
        .first()
        .cloned()
        .unwrap_or_else(|| "World".to_string())
        .into();
    name.into()
}

#[renderer]
fn render_name(name: ResultName) -> RenderResult {
    let mut result = RenderResult::default();
    result.println(&format!("Hello, {}!", *name));
    result
}

#[renderer]
fn render_dispatcher_not_found(err: ErrorDispatcherNotFound) -> RenderResult {
    let mut result = RenderResult::default();
    if err.len() > 0 {
        result.println(&format!("Command not found: [{}]", err.join(" ")));
    }
    result
}

gen_program!();
```

Output:

```text
> mycmd greet
Hello, World!
> mycmd greet Alice
Hello, Alice!
> mycmd great
Command not found: [great]
```

## Examples

`Mingling` provides detailed usage examples for your reference.
See [Examples](EXAMPLES/index.html) or [Helpdoc](https://mingling-rs.github.io/mingling/docs/examples.html)

## About Features

All features of `Mingling` are opt-in. To learn what each feature provides, see [Features](feature/index.html) or [Helpdoc](https://mingling-rs.github.io/mingling/docs/doc.html#/pages/other/features)

## Use unreleased version

If you want to use the latest development version, you can pull from the [Unreleased](https://github.com/mingling-rs/mingling/tree/unreleased) branch on [GitHub](https://github.com/mingling-rs/mingling) by adding the following to your `Cargo.toml`:

```toml
[dependencies.mingling]
git = "https://github.com/mingling-rs/mingling.git"
tag = "unreleased"
features = []
```

**Please note** that the documentation for the latest version may differ from this article. It is recommended to refer to the [latest documentation](https://mingling-rs.github.io/mingling/docs/api-docs/mingling/).
