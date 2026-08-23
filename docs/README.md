<h1 align="center">Mingling Documentation 🇨🇳</h1>

<p align="center">
    This page contains the documentation for the <b>Mingling</b> CLI framework, maintained by <a href="https://github.com/Weicao-CatilGrass">Weicao-CatilGrass</a>
</p>

## Introduction

If you want to write a monolithic CLI program with many subcommands that requires long-term maintenance,
and you also need context-aware command-line completion, then Mingling is definitely worth learning!

## About Mingling

Mingling's slogan is: "Macro magician in your CLI.",
It uses a macro system to construct a clear paradigm for describing CLI programs.

Mingling is free to use forever, and you can use it under either the [MIT](https://github.com/mingling-rs/mingling/blob/main/LICENSE-MIT) or [APACHE 2.0](https://github.com/mingling-rs/mingling/blob/main/LICENSE-APACHE) license.

Mingling's design goals are:

- Type and state driven: types describe state, functions handle behavior, macros mark them!
- Compile-time focus: all command lists, states, and types are fully baked in at compile time, keeping runtime lightweight.
- Side-effect isolation: isolate side effects of behavior handling as much as possible through framework mechanisms, creating a clean context.
- Testability: all steps are functions, making it easy to assert and inject context.
- Easy ecosystem composition: capabilities can be integrated through the Rust ecosystem, with the framework providing pure scheduling logic.

## Things you must know before using:

### Single-Crate Architecture

You can split crates using external framework-agnostic logic,
and Mingling also provides extensibility points like Setup, Resource, and Hook that can be split out.

However, all the **binding layers** (i.e., the layer connecting commands to actual behavior) of the final CLI are strictly confined to the same crate.

If your business requires splitting that binding layer, please consider this carefully.

### About Stability

Mingling is still under active development, and its API changes frequently. If the program you need to develop is **production-grade**,
it is highly recommended not to use Mingling.

### Parser VS Framework

Mingling is not a CLI Parser, but rather a CLI Framework. It does not have built-in argument parsing capabilities.

However, Mingling provides two features that allow you to integrate the [`clap`](https://github.com/clap-rs/clap) parser or the [`arg-picker`](https://github.com/catilgrass/arg-picker) external parser, which is better suited for Mingling.

## Getting Started

Of course, if your motivation for using Mingling is "Just for fun," then that's the best state to be in!
