<h1 align="center">[T0] Generalize the REPL System</h1>
<p align="center">
    Breaking: remove the <code>repl</code> feature and expose execution interfaces on the Program
</p>

> [!NOTE]
>
> This is a **Breaking Change** planned for Mingling 0.5.0.

## Background

The current REPL is merely _usable_, but far from _user-friendly_. The `repl` feature locks a specific interactive front-end into the framework, while the underlying execution model is what actually provides value.

## Plan

- Remove the `repl` feature in 0.5.0.
- By default, expose more execution-related interfaces for the `Program`, so that users can extend functionality beyond the REPL by leveraging Mingling's execution model.

The goal is to separate the execution model from any particular interactive front-end, letting users build their own REPL (or other execution drivers) on top of Mingling's public interfaces.

## Tasks

- [ ] Audit the current `repl` implementation and identify which behaviors belong to the execution model vs. the interactive front-end
- [ ] Design and expose the execution-related interfaces on `Program` (e.g. per-input execution, result handling, exit semantics)
- [ ] Remove the `repl` feature from `mingling`, `mingling_core`, and `mingling_macros`
- [ ] Remove or migrate the built-in REPL front-end
- [ ] Update examples and docs that enable `repl`

## 🕘 Progress

- [ ] In Progress
- [ ] Complete

<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
