<h1 align="center">[T1] Move structural_renderer from mingling_core to mingling</h1>
<p align="center">
    Breaking: inject <code>StructuralRenderer</code> via Hook instead of hardcoding it into the core loop
</p>

> [!NOTE]
>
> This is a **Breaking Change** planned for Mingling 0.5.0.

## Background

Mingling's Hook system is now complete, so there's no longer a need to hardcode `StructuralRenderer` into the core loop.

The plan is to remove it from `exec.rs` and instead inject the Hook implementation via `StructuralRendererSetup`.

## Plan

- Remove the hardcoded `StructuralRenderer` from the core execution loop (`exec.rs`).
- Implement the renderer as a Hook and inject it via `StructuralRendererSetup`.
- `mingling_core` no longer depends on the structural renderer; the wiring moves up to the `mingling` crate level.

## Tasks

- [ ] Audit where `StructuralRenderer` is hardcoded in `mingling_core` (`exec.rs` and related)
- [ ] Design `StructuralRendererSetup` as a Hook implementation
- [ ] Move / re-implement the renderer wiring in `mingling`
- [ ] Clean up `mingling_core`'s `structural_renderer` feature and its serde deps (if no longer needed there)
- [ ] Migrate examples and tests
- [ ] Verify structural renderer output is unchanged

## 🕘 Progress

- [ ] In Progress
- [ ] Complete

<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
