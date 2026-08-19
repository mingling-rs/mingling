<h1 align="center">[Solved] [T0] Remove with_dispatcher and with_dispatchers</h1>
<p align="center">
    Breaking: make <code>Dispatcher</code> registration compile-time collected in all modes
</p>

> [!NOTE]
>
> This is a **Breaking Change** planned for Mingling 0.5.0.

## Background

Mingling's commands must be registered through `with_dispatcher` in order to be usable when `dispatcher_tree` is disabled. This has always been a strange semantic: `chain`, `renderer`, `help`, `completion`, and `metadata` are all collected at compile time, so why is `Dispatcher` the exception?

In fact, during usage of `dispatcher` from 0.1.0 to 0.4.0, no scenario has ever been encountered where **dynamic registration** was necessary. It is considered unnecessary.

## Plan

Make `Dispatcher` registration also compile-time collected in non-`dispatcher_tree` states starting from 0.5.0, and remove `with_dispatcher` / `with_dispatchers` (and the related `#[program_setup]` registration path if it becomes obsolete).

## Tasks

- [x] Design how dispatchers are collected at compile time when `dispatcher_tree` is disabled (consistent with how `chain` / `renderer` / `completion` / `metadata` are collected)
- [x] Remove `with_dispatcher` and `with_dispatchers` from the `Program` API
- [x] Update `gen_program!` and the macros so registration happens automatically
- [x] Migrate examples, tests, and docs that call `with_dispatcher` / `with_dispatchers`
- [x] Verify both `dispatcher_tree`-enabled and disabled modes behave identically

## 🕘 Progress

- [x] In Progress
- [x] Complete

<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
