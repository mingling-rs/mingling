<h1 align="center">[T1] The pathf_export Attribute Macro</h1>
<p align="center">
    Feature: an escape hatch for <code>pathf</code>'s path inference
</p>

## Background

`pathf` has been around since 0.2.0 and has worked well for a long time, with many edge cases resolved. However, it still lacks an escape hatch — "when certain indirect expansions cannot be recognized by `pathf`, how can we assist its inference?"

For example, when a Mingling type is created through a user-defined `macro_rules!` wrapper, `pathf` cannot see through the indirect expansion:

```rust
#[macro_export]
macro_rules! repack {
    ($name:ident) => {
        // Ignored! This section cannot be parsed by pathf.
        #[mingling::macros::pathf_ignore]
        #[derive(mingling::Grouped)]
        pub struct $name;
    };
}
 
// The expansion contains macros that need to be parsed by pathf
#[pathf_export(MyType)] // Explicitly specified to assist pathf's inference
repack!(MyType);
```
 
> [!Note]
> Haha, hopefully we'll never have to use it.

## Plan

Introduce a new attribute macro, `#[pathf_export(type::TypePath)]`, to supplement `pathf`'s path inference. When applied to an item whose expansion contains Mingling types that `pathf` cannot recognize, it explicitly records the resulting type paths so the build-time analyzer can pick them up.

The example above also shows `#[mingling::macros::pathf_ignore]`, which marks an item to be skipped by `pathf` (used inside macro bodies that `pathf` otherwise cannot parse).

## Tasks

- [ ] Design the `pathf_export` syntax and semantics (attribute position, multiple type paths, interplay with `pathf_ignore`)
- [ ] Implement `pathf_export` in `mingling_macros`
- [ ] Implement `pathf_ignore` support in `mingling_macros`
- [ ] Teach `mingling-pathf` to consume the exported mappings
- [ ] Add tests covering indirect macro expansions
- [ ] Update docs and examples

## 🕘 Progress

- [ ] In Progress
- [ ] Complete

<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
