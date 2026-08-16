<h1 align="center">About ProgramCollect</h1>
<p align="center">
    Understand how gen_program!() builds a program
</p>

Every Mingling program ends with a `gen_program!()` call. Behind the scenes, it does three things to scaffold the entire program.

## The three tasks of gen_program!()

### 1. Generate an enum

Scans the current module for all types marked with `#[derive(Grouped)]`, `#[chain]`, `#[renderer]` and similar macros, then generates an enum variant for each type.

This enum is the type of `G` in `AnyOutput<G>` — the scheduler uses enum variants to distinguish different data flowing through the pipeline.

### 2. Generate a ProgramCollect impl

`ProgramCollect` is a trait that defines the mapping of **"which type each enum variant corresponds to and who handles it"**:

- **`do_chain`** — calls the corresponding `#[chain]` function by `member_id`, returns a new `AnyOutput` and `NextProcess`
- **`render`** — calls the corresponding `#[renderer]` function by `member_id`, writes to `RenderResult`
- **`render_help`** — calls the corresponding `#[help]` function by `member_id`
- **`has_chain` / `has_renderer`** — checks whether a variant has a corresponding handler
- **`build_entry_fallback` / `build_renderer_not_found` / `build_empty_result`** — three built-in fallback types for edge cases

This mapping is resolved at runtime via enum matching — only the enum and match branches are generated at compile time; actual function calls happen at runtime.

### 3. Generate ThisProgram

Generates the `ThisProgram` type alias, pointing to `Program<GeneratedEnum>`. That's why you can write `ThisProgram::new()` directly in `main` — it's the complete type of your whole program.

---

## Differences under `pathf` and `dispatch_tree`

The above describes the default behavior, which changes when specific features are enabled:

### 1. `dispatch_tree` feature

The Dispatcher no longer uses `Vec<Box<dyn Dispatcher>>` for linear matching. Instead, the subcommand structure is built as a prefix tree (Trie) at compile time.

Matching complexity drops from `O(n)` to `O(k)` — where `k` is input length, independent of the number of commands.

### 2. `pathf` feature (Module Pathfinder)

By default, all macro-marked types must be in the same module for `gen_program!()` to collect them.

With `pathf` enabled, the compiler automatically scans all sub-modules at compile time, finds all macro-marked types, and generates full module path references — types defined in deep sub-modules don't need a manual `use`.

<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
