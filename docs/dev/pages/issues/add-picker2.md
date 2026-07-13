<h1 align="center">The Picker2 Arguments Parser</h1>
<p align="center">
    A smarter, faster alternative to Picker
</p>

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
    //                                         requires impl Groupped<_>
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

## Current Progress (2026-07-14)

Picker2 has been significantly implemented. Here is the current status:

### Completed

#### Core Type System

- **`PickerArgs`** — Three-tier ownership enum for CLI arguments:
  - `Slice(&'a [&'a str])` — zero-copy borrow from `std::env::args()`
  - `Vec(Vec<&'a str>)` — owned container, borrowed elements
  - `Owned(Vec<String>)` — fully owned (constructed / manipulated args)

- **`PickerResult`** — Four-state parse result:
  - `Unparsed` — not yet parsed (default)
  - `Parsed(Type)` — successfully parsed
  - `NotFound` — requested value not found
  - `FormatError` — input could not be parsed

- **`PickerFlag`** — Const-compatible parameter constraint definition:
  ```rust
  pub struct PickerFlag<'a, Type: Pickable> {
      pub full: &'a [&'a str],     // flag names & aliases
      pub short: Option<char>,      // short name, e.g. 'n'
      pub positional: bool,         // positional or named
      pub internal_type: PhantomData<Type>,
  }
  ```
  All fields are `pub`, enabling direct struct literal construction.

#### `internal_repeat!` Template Engine

A custom proc-macro template engine that generates repetitive code without nightly features or external dependencies.

**Syntax:**

```rust
internal_repeat!(1..=32 => {
    // $   — current counter (1, 2, ..., 32)
    // $+  — current + 1
    // $-  — current - 1
    // $^  — loop minimum
    // ^$  — loop maximum
    // ident$     → ident{current}
    // ident$+    → ident{current+1}
    // ident$-    → ident{current-1}
    // ident$^    → ident{min}
    // (group,+)  — repeat * times (current), `,` separator
    // (group,+)+ — repeat *+1 times (current + 1)
    // (group,+)- — repeat *-1 times (current - 1)
});
```
 
Generates `PickerPattern1` through `PickerPattern32` structs, each with `N` typed flag/result fields:

```rust
pub struct PickerPattern3<'a, T1, T2, T3> {
    pub args: PickerArgs<'a>,
    pub flag_1: &'a PickerFlag<'a, T1>,
    pub result_1: PickerResult<T1>,
    pub flag_2: &'a PickerFlag<'a, T2>,
    pub result_2: PickerResult<T2>,
    pub flag_3: &'a PickerFlag<'a, T3>,
    pub result_3: PickerResult<T3>,
}
```
 
#### `flag!` Macro

Declarative parameter definition macro. Expands to a const-compatible struct literal:

```rust
// Syntax variants:
flag![name: String]                     // named, no short
flag![name: String, 'n']                // named, with short
flag![name: String, 'n', "alias"]       // named, with short & alias
flag![String]                           // positional, no name
flag![String, 'F']                      // positional, with short
flag![String, 'o', "output"]            // positional, with short & alias
 
// Expansion (all const-compatible):
::mingling::picker::PickerFlag::<String> {
    full: &["name"],
    short: ::std::option::Option::Some('n'),
    positional: false,
    internal_type: ::std::marker::PhantomData,
}
```
 
The type part preserves original token spans for IDE support (Rust Analyzer).

#### `Pickable` Trait

Implemented for all numeric types, `String`, `PathBuf`, `Option<T>`.

```rust
pub trait Pickable {
    fn pick(raw_str: &str) -> PickerResult<Self>;
}
```
 
#### Type-Safe Chaining

```rust
let p4: PickerPattern4<String, i32, String, String> = args
    .pick(&flag![name: String])
    .pick(&flag![age: i32])
    .pick(&flag![food: String, 'F'])
    .pick(&flag![kind: String]);
```
 
Each `.pick()` call adds one type parameter to the pattern. Parameter count is compile-time enforced — writing 3 `.pick()` calls produces `PickerPattern3`, and trying to assign to `PickerPattern4` fails to compile.

### ❌ Pending

#### `.parse()` — Runtime Parsing Engine

Not yet implemented. The proposed parsing flow:

```rust
let result: (String, i32) = args
    .pick(&flag![name: String])
    .pick(&flag![age: i32])
    .parse();
```
 
Procedural steps:

1. Allocate a `Vec<Box<dyn Any>>` indexed by type position
2. Sort all flags: named flags first, positional flags after
3. Each `PickerFlag`'s `Pickable` scans the argument list and marks its tokens
4. Each `Pickable::pick()` resolves its value into the `Vec`
5. Downcast each `Box<dyn Any>` to the concrete type
6. Convert via `Into<(...)>` into the final tuple

Planned companion code generation via `internal_repeat!`:

```rust
internal_repeat!(1..=32 => {
    impl<'a, (T$,)+> PickerPattern$<'a, (T$,)+> {
        pub fn parse(self) -> ((T$,)+) {
            // ...
        }
    }
});
```
 
#### `.pick_route()` — Error Routing

Post-processing with route-on-error:

```rust
let parsed = args
    .pick_route::<String, _>(flag![path: PathBuf], ErrorNotFound)
    .parse();
```
 
#### `MultiFlag` — Combined Flags

```rust
#[derive(MultiFlag)]
pub struct Toggles {
    pub install: bool,      // -i
    #[flag('U')]
    pub uninstall: bool,    // -U
    #[flag('X')]
    pub execute: bool,      // -X
}
// Supports: `-iUX`
```
 
#### `PickerConfig` — Arg Format Configuration

Support for `--key=value`, `/Key=Value`, etc.

### Summary

| Component                          | Status |
| ---------------------------------- | ------ |
| `PickerArgs` (3-tier ownership)    | ✅     |
| `PickerResult` (4-state)           | ✅     |
| `PickerFlag` (const-compatible)    | ✅     |
| `Pickable` trait + impls           | ✅     |
| `internal_repeat!` template engine | ✅     |
| `flag!` macro                      | ✅     |
| `PickerPattern1..32` structs       | ✅     |
| Chainable `.pick()` (2→31)         | ✅     |
| **`.parse()` runtime**             | ❌     |
| `.pick_route()` error routing      | ❌     |
| `MultiFlag` derive                 | ❌     |
| `PickerConfig`                     | ❌     |
| Integration with `#[chain]`        | ❌     |
