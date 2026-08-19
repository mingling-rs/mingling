<h1 align="center">[Solved] [T0] Remove the parser Feature</h1>
<p align="center">
    Breaking: retire the legacy argument parsing in favor of <code>picker</code>
</p>

> [!NOTE]
>
> This is a **Breaking Change** planned for Mingling 0.5.0.

## Background

In 0.3.0, Mingling introduced the `picker` feature, which provides more powerful parameter parsing capabilities. At that point, the original `parser` feature became inadequate.

The `parser` feature was a temporary argument parsing solution created in the early stages of the project. While it can handle basic argument parsing tasks, its functionality is incomplete and has many limitations (see the [Picker2 issue](_add-picker2) for the full list).

## Plan

Completely remove the `parser` feature in 0.5.0. This will directly affect downstream users of the `parser` feature — they must migrate to `picker` (or handle parsing manually).

## Tasks

- [x] Identify all usages of the `parser` feature across the codebase, examples, and docs
- [x] Migrate internal usages (tests, examples, dev-dependencies) to `picker`
- [x] Remove the `parser` feature from `mingling` and its dependency (`size`)
- [x] Remove parser-related modules and public API
- [x] Update docs and helpdoc examples
- [x] Note the downstream migration path in the changelog

## 🕘 Progress

- [x] In Progress
- [x] Complete

<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
