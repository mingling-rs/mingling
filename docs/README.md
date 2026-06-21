<h1 align="center">Mingling Helpdoc</h1>

<p align="center">
    This is documentation for the <b>Mingling</b> command-line framework, maintained by <a href="https://github.com/Weicao-CatilGrass">Weicao-CatilGrass</a>
</p>

## Intro

If you're troubled by these issues —

1. Too many subcommands make `main.rs` bloat rapidly;
2. Handler logic mixed with side effects;
3. Logging, auth, and other cross-cutting concerns are hard to inject non-invasively;
4. Maintaining shell completion scripts for multiple platforms wears you out;
5. Global resources everywhere, making testing difficult.

… then you've come to the right place.

Of course, if you're just curious about "how to write maintainable CLIs in Rust," you've also come to the right place — it'll be fun.

## What is Mingling?

> **Mìng Lìng** is the pinyin of the Chinese word **命令**,
> which translates to **Command** in English.

Mingling is a CLI framework built with Rust. It's free, open-source, and licensed under the permissive MIT / Apache 2.0 license.

Mingling's design goals:

- **Extensible**: From 3 subcommands to 30, same pattern, no framework swap
- **Decoupled**: Parse args once, write business logic once, define output format once. Each independent.
- **Type-driven**: Clear, typed data flows through the pipeline — not `Vec<String>`
- **Lightweight deps**: Minimal core deps, low cost to pull in; advanced features on demand, no compile-time drag
- **Efficient**: Dispatch logic generated at compile time, no unnecessary runtime overhead

---

Alright, wanna give it a try?
