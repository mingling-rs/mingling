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
  <img src="https://img.shields.io/github/license/mingling-rs/mingling">
	<img src="https://img.shields.io/github/stars/mingling-rs/mingling?style=flat"> 
	<img src="https://img.shields.io/crates/size/mingling">
	<img src="https://img.shields.io/crates/v/mingling?style=flat">
	<img src="https://img.shields.io/docsrs/mingling?style=flat">
	<img src="https://img.shields.io/github/actions/workflow/status/mingling-rs/mingling/ci.yml">
</p>

## What is Mingling?

[`Mingling`](https://github.com/mingling-rs/mingling) is a **state-driven and data-driven** CLI workflow orchestration framework built in Rust.

💡 Its name comes from the Chinese pinyin **"Mìng Lìng"**, which means **"command"**.

## WARNING

Mingling is currently usable at a basic level, but it is still under active development, so many APIs are not yet mature. Any changes to the public API will be documented in detail in the [Changelog](https://github.com/mingling-rs/mingling/blob/main/CHANGELOG.md).

Additionally, the project is currently developed by me alone ([Weicao-CatilGrass](https://github.com/Weicao-CatilGrass)). If you are interested in this project, I warmly welcome your [contributions](https://github.com/mingling-rs/mingling/blob/main/CONTRIBUTING.md). You can reach me directly via [Github Issue](https://github.com/mingling-rs/mingling/issues).

## About Mingling's Design

Mingling abstracts the behavior of a program's lifecycle into three phases: **Dispatch**, **Execution**, and **Rendering**. Each phase is connected by types — the output of the current phase becomes the input of the next phase. For example:

```rust
dispatcher!("current", CMDCurrent => EntryCurrent);
pack!(StateNext = ());

#[chain]
fn handle_current(_: EntryCurrent) -> StateNext {
    // 1. The first phase outputs the StateNext value
    StateNext::default()  //          ^^^^^^^^^
}                 //                  |
                  //                  |
                  // 2. The second phase takes StateNext as input
#[chain]          //    |
fn handle_state_next(_: StateNext) {
    todo!()
}
```

See? `handle_current` and `handle_state_next` have no direct connection!

They are bridged by `StateNext` and automatically linked by the framework.

You can use this approach to separate computation from result rendering, like this:

```rust
// Features: ["picker"]
dispatcher!("calc", CMDCalculate => EntryCalculate);
pack!(StateSumNumbers = Vec<i32>);
pack!(ResultNumber = i32);

// Entry: parse arguments and pass state to the calculation step
#[chain]
fn handle_calc(args: EntryCalculate) -> StateSumNumbers {
    let numbers = args.pick(&arg![Vec<i32>]).unwrap();
    StateSumNumbers::new(numbers)
}

// Calculate: pass the result to the rendering step
#[chain]
fn handle_state_sum_numbers(sum: StateSumNumbers) -> ResultNumber {
    let numbers = sum.inner;
    let total: i32 = numbers.iter().sum();
    ResultNumber::new(total)
}

// Renderer: return the render result and let the framework handle output
#[renderer]
fn render_number(number: ResultNumber) -> RenderResult {
    let mut result = RenderResult::new();
    writeln!(result, "Number: {}", *number).ok();
    result
}
```

Although this may make your program slightly more verbose, each step is a **pure function**, making it extremely easy to test!

## Getting Started

Add Mingling to your `Cargo.toml`:

```toml
[dependencies.mingling]
version = "0.3.0"
features = []
```

Or use the github version

```toml
[dependencies.mingling]
git = "https://github.com/mingling-rs/mingling.git"
tag = "unreleased"
features = []
```

> [!NOTE]
> To learn more, check out [Writing with Mingling](https://github.com/mingling-rs/mingling/blob/main/GETTING_STARTED.md)

## Roadmap

- [x] Milestone.1 "MVP" 🎉
  - [x] [[0.1.4](https://docs.rs/mingling/0.1.4/mingling/)] [`core`] [`structural_renderer`] **Mingling** can render data into serializable formats via `--json` and `--yaml` flags
  - [x] [[0.1.5](https://docs.rs/mingling/0.1.5/mingling/)] [`core`] [`comp`] **Mingling** can dynamically invoke itself to provide completions for shells like `bash`, `zsh`, `fish`, and `pwsh`
  - [x] [[0.1.6](https://docs.rs/mingling/0.1.6/mingling/)] [`core`] [`comp`] **Mingling** can gather more context for smarter completions
  - [x] [[0.1.7](https://docs.rs/mingling/0.1.7/mingling/)] [`clap`] Provides a **Clap** compatibility layer, allowing **Mingling** to reuse its powerful parsing capabilities
  - [x] [[0.1.7](https://docs.rs/mingling/0.1.7/mingling/)] [`core`] **Mingling** can intercept `-h` or `--help` flags to display custom help text for each subcommand
  - [x] [[0.1.7](https://docs.rs/mingling/0.1.7/mingling/)] [`mling`] Provides a basic scaffolding tool (`mling`) for rapid development and debugging
  - [x] [[0.1.8](https://docs.rs/mingling/0.1.8/mingling/)] [`core`] [`dispatch_tree`] Converts the subcommand list into a prefix tree to improve command matching speed
  - [x] [[0.1.9](https://docs.rs/mingling/0.1.9/mingling/)] [`core`] [`dev_toolkits`] Provides debugging interfaces for developers to capture invocation information when issues arise (`InvokeStackDisplay`) (indirectly implemented via `ProgramHook`)
  - [x] [[0.1.9](https://docs.rs/mingling/0.1.9/mingling/)] [`core`] [`repl`] Provides REPL capability (`program.exec_repl();`)
  - [x] [[0.2.0](https://docs.rs/mingling/0.2.0/mingling/)] Complete documentation, tests, and examples
- [ ] Milestone.2 "More Comfortable Dev and User Experience"
    - [ ] [`mling` / `mingling-cli`]
        - [ ] **Mingling** Linter
        - [ ] **Mingling** Project Generator
        - [ ] **Mingling** Program Installer & Manager (For development)
        - [ ] Helpdoc Editor
    - [ ] [`picker`] A more efficient and intelligent argument parser
    - [x] [`macros`] Remove `r_print!` / `r_println!` macros
- [ ] Milestone.3 "Unplanned"
    - [ ] ...

## Unplanned Features

While Mingling has several common CLI features that are **NOT PLANNED** to be directly included in the framework.
This is because the Rust ecosystem already has excellent and mature crates to handle these issues, and Mingling's design is intended to be used in combination with them.

- **Colored Output**: To add color and styles (bold, italic, etc.) to terminal output, consider using crates like [`colored`](https://crates.io/crates/colored) or [`owo-colors`](https://crates.io/crates/owo-colors). You can integrate their types directly into your renderers.
- **I18n**: To translate your CLI application, the [`rust-i18n`](https://crates.io/crates/rust-i18n) crate provides a powerful internationalization solution that you can use in your command logic and renderers.
- **Progress Bars**: To display progress indicators, the [`indicatif`](https://crates.io/crates/indicatif) crate is the standard choice.
- **TUI**: To build full-screen interactive terminal applications, it is recommended to use a framework like [`ratatui`](https://crates.io/crates/ratatui) (formerly `tui-rs`).

## License

This project is licensed under the MIT License.

See [LICENSE-MIT](LICENSE-MIT) or [LICENSE-APACHE](LICENSE-APACHE) file for details.

## Learn More

**To learn more, check out the following links:**

- 📦 Repo - [Github](https://github.com/mingling-rs/mingling) | [Gitee](https://gitee.com/mingling-rs/mingling) | [Origin](https://catilgrass.cn/mingling.git)
- 🚪 Mainpage - [Github](https://mingling-rs.github.io/mingling/) | [crates.io](https://crates.io/crates/mingling)
- 💡 Examples - [Github](https://mingling-rs.github.io/mingling/docs/examples.html)
- 📖 Help Doc - [EN](https://mingling-rs.github.io/mingling/docs/doc.html#/) | [中文](https://mingling-rs.github.io/mingling/docs/_zh_CN/index.html#/)
- 📖 API Doc - [docs.rs](https://docs.rs/mingling/latest/mingling/) | [latest](https://mingling-rs.github.io/mingling/docs/api-docs/mingling/)
- 📖 Dev Doc - [Github](https://mingling-rs.github.io/mingling/docs/dev/)
