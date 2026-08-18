<p align="center">
    <a href="https://github.com/mingling-rs/mingling">
        <img alt="Mingling" src="https://github.com/mingling-rs/mingling/raw/main/docs/res/icon3.png" width="30%">
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
    <img alt="License" src="https://img.shields.io/github/license/mingling-rs/mingling?style=for-the-badge&color=yellow">
    <img alt="GitHub stars" src="https://img.shields.io/github/stars/mingling-rs/mingling?style=for-the-badge&color=yellow">
		<img alt="Crate size" src="https://img.shields.io/crates/size/mingling?style=for-the-badge">
		<img alt="Crates.io version" src="https://img.shields.io/crates/v/mingling?style=for-the-badge">
		<img alt="CI" src="https://mingling-rs.github.io/badges/badge-build.png" height="36">
</p>

## What is Mingling?

[`Mingling`](https://github.com/mingling-rs/mingling) is a **state-driven and data-driven** CLI workflow orchestration framework built in Rust.

Its name comes from the Chinese pinyin **"Mìng Lìng"**, which means **"command"**.

## WARNING

Mingling is currently usable at a basic level, but it is still under active development, so many APIs are not yet mature. Any changes to the public API will be documented in detail in the [Changelog](https://github.com/mingling-rs/mingling/blob/main/CHANGELOG.md).

Additionally, the project is currently developed by me alone ([Weicao-CatilGrass](https://github.com/Weicao-CatilGrass)). If you are interested in this project, I warmly welcome your [contributions](https://github.com/mingling-rs/mingling/blob/main/CONTRIBUTING.md). You can reach me directly via [Github Issue](https://github.com/mingling-rs/mingling/issues).

## About Mingling's Design

Mingling aims to organize and manage the architectural concerns of command-line programs through reasonable abstractions: it breaks a program down into the following concepts:

|        Concept | Description                                                      |
| -------------: | :--------------------------------------------------------------- |
|    **Command** | A combination of **Dispatcher** and **Chain**                    |
| **Dispatcher** | Maps user input to entry types                                   |
|      **Chain** | Provides behavioral logic for any type and returns the next type |
|   **Renderer** | Renders any type into output-ready text                          |
|   **Resource** | Provides global **data** for the program                         |
|       **Hook** | Provides global **behavior** for the program                     |

## Example

Below is a typical Mingling program that demonstrates how to implement a simple adder:

```rust
// Features: ["mini"]

#[derive(Grouped)]
struct ResultNumber(f32);

#[command]
fn sum(args: Entry) -> ResultNumber {
    let (a, b) = args
        .pick(&arg![f32])
        .pick(&arg![f32])
        .unwrap();
    ResultNumber(a + b)
}

#[renderer]
fn render_number(n: ResultNumber) -> String {
    format!("Result is {}", n.0)
}
```

Output:

```bash
~# my-cli sum 5 10
Result is 15
```

If we add full error handling, it would look like this:

```rust
// Features: ["mini"]

use mingling::macros::routeify;
use mingling::setup::ExitCodeSetup;
use mingling::res::ResExitCode;

fn main() {
    let mut program = ThisProgram::new();
    program.with_setup(ExitCodeSetup);
    program.exec_and_exit();
}

#[derive(Grouped)]
struct ResultNumber(f32);

#[derive(Grouped)]
struct ErrorNoNumber;

#[command(routeify)]
fn sum(args: Entry) -> Next {
    let (a, b) = args
        .pick_or_route(&arg![f32], || ErrorNoNumber.into())
        .pick_or_route(&arg![f32], || ErrorNoNumber.into())
        .to_result()?;
    ResultNumber(a + b).into()
}

#[renderer]
fn render_error_no_num(n: ErrorNoNumber, ec: &mut ResExitCode) -> String {
    ec.exit_code = 1;
    format!("Error: No number provided.")
}

#[renderer]
fn render_number(n: ResultNumber) -> String {
    format!("Result is {}", n.0)
}
```

Output:

```bash
~# my-cli sum 5 10
Result is 15

~# my-cli sum
Error: No number provided. << 1
```

See! By assembling `ExitCodeSetup` and modifying `ResExitCode`, we **explicitly** mark the side effect of changing the exit code on the `render_error_no_num` function. This is exactly the problem Mingling aims to solve: **separating concerns by separating side effects through architecture**.

Of course, by combining the concepts above, you can elegantly separate the side effects of your program into independent resources, keeping your execution functions pure.

## Getting Started

Add Mingling to your `Cargo.toml`:

```toml
[dependencies.mingling]
version = "0.5.0"
features = []
```

Or use the github version

```toml
[dependencies.mingling]
git = "https://github.com/mingling-rs/mingling.git"
tag = "unreleased"
features = []
```

To learn more, check out [Writing with Mingling](https://github.com/mingling-rs/mingling/blob/main/GETTING-STARTED.md)

> [!Note]
> You can also use the `mling` scaffolding tool to build, check, and manage your project [Download](https://mingling-rs.github.io/mingling/dist) | [About](https://github.com/mingling-rs/mingling/tree/main/mingling_cli)

## Unplanned Features

While Mingling has several common CLI features that are **NOT PLANNED** to be directly included in the framework.
This is because the Rust ecosystem already has excellent and mature crates to handle these issues, and Mingling's design is intended to be used in combination with them.

- **Colored Output**: To add color and styles (bold, italic, etc.) to terminal output, consider using crates like [`colored`](https://crates.io/crates/colored) or [`owo-colors`](https://crates.io/crates/owo-colors). You can integrate their types directly into your renderers.
- **I18n**: To translate your CLI application, the [`rust-i18n`](https://crates.io/crates/rust-i18n) crate provides a powerful internationalization solution that you can use in your command logic and renderers.
- **Progress Bars**: To display progress indicators, the [`indicatif`](https://crates.io/crates/indicatif) crate is the standard choice.
- **TUI**: To build full-screen interactive terminal applications, it is recommended to use a framework like [`ratatui`](https://crates.io/crates/ratatui) (formerly `tui-rs`).

## Learn More

**To learn more, check out the following links:**

- ⚡ Mingling CLI - [About Mling](https://mingling-rs.github.io/mingling/dist)
- 📦 Repo - [Github](https://github.com/mingling-rs/mingling) | [Gitee](https://gitee.com/mingling-rs/mingling) | [Origin](https://catilgrass.cn/mingling.git)
- 🚪 Mainpage - [Github](https://mingling-rs.github.io/mingling/) | [crates.io](https://crates.io/crates/mingling)
- 💡 Examples - [Github](https://mingling-rs.github.io/mingling/docs/examples.html)
- 📖 Help Doc - [EN](https://mingling-rs.github.io/mingling/docs/index.html#/) | [中文](https://mingling-rs.github.io/mingling/docs/_zh_CN/index.html#/)
- 📖 API Doc - [docs.rs](https://docs.rs/mingling/latest/mingling/) | [latest](https://mingling-rs.github.io/mingling/docs/api-docs/mingling/)
- 📖 Coverage Test - [LLVM Coverage](https://mingling-rs.github.io/mingling/docs/cov-test/)
- 📖 Dev Doc - [Github](https://mingling-rs.github.io/mingling/docs/dev/)

- 📖 Contribution - [CONTRIBUTING.md](./CONTRIBUTING.md)
- 🗺 Roadmap - [ROADMAP.md](./ROADMAP.md)

## License

This project is licensed under the MIT License.

See [LICENSE-MIT](LICENSE-MIT) or [LICENSE-APACHE](LICENSE-APACHE) file for details.
