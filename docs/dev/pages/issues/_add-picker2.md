<h1 align="center">[Solved] The Picker2 Arguments Parser</h1>
<p align="center">
    A smarter, faster alternative to Picker
</p>

> [!NOTE]
>
> This issue has been fully resolved and is thus archived for preservation.

## Intro

Mingling's `parser` feature is a temporary argument parsing solution created in the early stages of the project. While it can handle basic argument parsing tasks, its functionality is incomplete and has many limitations.

This article aims to propose the design and development plan for the new Picker2 (feature name: `picker`).

### Picker2 Expected Syntax

1. **Pos args & flag args no longer depend on parsing order:** In `parser`, all flag args must be parsed before pos args. Picker2 removes this restriction.
2. **Declare flags with declarative macros:** Use `positional!()`, `flag![-X, --x]` etc. to declare flags — cleaner and more concise.
3. **One-shot `parse()` replaces step-by-step `unpack()`:** Defers the entire parsing flow to the final step, leaving more room for compile-time optimization.

```rust
#[chain]
fn handle_hello(args: EntryHello) {
    let parsed = args
        .pick::<String>(positional!())
        .pick::<String>(flag![--help, -h])
        .parse();
}
```
 
4. **Post-processing & routing control:** Use `after`, `after_route` to control post-processing logic. The route type is explicitly specified by the first `route` method.

```rust
#[chain]
fn handle_hello(args: EntryHello) {
    let parsed = args
        .pick::<String>(positional!())
        .after(|v| format!("\"{}\"", v)) // post-processing
        .parse();
}
```
 
```rust
#[chain]
fn handle_hello(args: EntryHello) -> Next {
    //                                         requires impl Grouped<_>
    //                                         |
    let parsed = args //                       vvvvvvvvvvvvvvvvvvv
        .pick_route::<String, _>(positional!(), ErrorNoNameProvided)
        .after_route(|v| Ok(format!("\"{}\"", v))) // post-process & route
        .parse();
    let routed_parsed = route!(parsed);
    empty_result!()
}
```
 
5. **Keep the `Pickable` Trait**
6. **Keep `PickableEnum` & provide a derive macro**
7. **Use `pick_optional`, `pick_vec` instead of implementing trait separately for `Option<T>`, `Vec<T>`**
8. **Use `pick_flag` instead of `pick::<bool>`**
9. **Provide `YesOrNo`, `TrueOrFalse` etc. that implement the `BoolFlag` trait for explicit bool extraction**
10. **Provide a `MultiFlag` derive macro, supporting arg modes like `pacman`**

```rust
#[derive(MultiFlag)]
pub struct SomeToggles {
    // default [-i]
    pub install: bool,
 
    #[flag('U')]
    pub uninstall: bool,
 
    #[flag('X')]
    pub execute: bool,
}
 
// Auto-implements Pickable, supports parsing args like `-iUX`
```
 
11. **Support multiple arg formats:** e.g. `--key=value`, `/Key=Value`, controlled by global config `PickerConfig` (or: `Picker::from_with_cfg(prev.inner, PickerConfig::default())`)

12. `.parse()` no longer returns a bare tuple, but instead returns:

```rust
pub struct PickerResult<Tuple> {
    pub result: Tuple,
    pub remains_argument: Arguments,
}
```
 
---

## 🕘 Progress

- [x] In Progress
    - [x] Added `Picker` struct and related call chain
    - [x] Added `parselib` providing parsing logic
    - [x] Added `Pickable` for extensibility
    - [x] Comprehensive testing!
    - [x] Improve documentation
    - [x] Add examples
    - [x] Update README
- [x] Complete
