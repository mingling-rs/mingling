`<h1 align="center">[T0] Remove the pack! Family of Macros</h1>
<p align="center">
    Breaking: retire the entire <code>pack!</code> family in favor of the <code>Grouped</code> derive
</p>

> [!NOTE]
>
> This is a **Breaking Change** planned for Mingling 0.5.0.

## Background

Since the very first version of Mingling, the `pack!` macro has been around. Its purpose has gradually narrowed from "creating a type and registering it to Mingling" to "creating a newtype that derives Grouped". In other words, the functionality of `pack!` is gradually being replaced by the `Grouped derive`.

Furthermore, in 0.2.0, in order to accommodate `StructuralData derive`, Mingling introduced `pack_structural!` and `pack_err_structural!` variants all at once, which greatly increases the maintenance cost of the project.

## Plan

Remove the entire `pack!` family of macros (`pack!`, `pack_structural!`, `pack_err_structural!`) as a Breaking Change in 0.5.0.

All future type creation will be done as follows:

```rust
// Before
pack!(ResultNames = Vec<String>);
 
// After
#[derive(Grouped)]
pub struct ResultNames {
    names: Vec<String>
}
```
 
## Tasks

- [x] Identify all usages of `pack!` / `pack_structural!` / `pack_err_structural!` across the codebase, examples, and docs
- [x] Migrate internal usages to `#[derive(Grouped)]`
- [x] Remove the macro definitions and their re-exports
- [x] Update the docs / helpdoc examples that reference `pack!`
- [x] Update downstream feature docs (`structural_renderer` etc.) where `pack_structural!` was involved
- [x] Verify all tests pass

## 🕘 Progress

- [x] In Progress
- [x] Complete

<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
