<h1 align="center">[Solved] [T1] Modify the dispatcher! Syntax</h1>
<p align="center">
    Breaking: drop the <code>CMD*</code> struct from the explicit form of <code>dispatcher!</code>
</p>

> [!NOTE]
>
> This is a **Breaking Change** planned for Mingling 0.5.0, and it depends on [Remove with_dispatcher and with_dispatchers](t0_remove-with-dispatcher).

## Background

Once `Dispatcher` registration is compile-time collected (see [Remove with_dispatcher and with_dispatchers](t0_remove-with-dispatcher)), the `CMD*` struct becomes unnecessary — it only existed to give `with_dispatcher` something to register.

## Plan

Simplify the `dispatcher!` syntax: the explicit form no longer creates a `CMD*` struct, so only the entry type needs to be given.

```rust
// Before
dispatcher!("command", CMDCommand => EntryCommand);
 
// After
dispatcher!("command", EntryCommand);
 
// NOTE: The implicit mode is not affected
```
 
## Tasks

- [x] Update `dispatcher!` to accept the simplified explicit form (`"name", EntryType`)
- [x] Decide whether the old `CMDType => EntryType` form should error with a helpful message or be removed outright
- [x] Remove the generated `CMD*` struct machinery
- [x] Update `#[command]` macro internals that depend on `CMD*`
- [x] Migrate examples, tests, and docs
- [x] Keep the implicit mode (`dispatcher!("name")`) working unchanged

## 🕘 Progress

- [x] In Progress
- [x] Complete

<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
