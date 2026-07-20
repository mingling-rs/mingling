# Changelogs

This file tracks all notable changes to the Mingling project. Each release entry documents new features, bug fixes, optimizations, and breaking changes, helping developers and users understand the evolution of the framework.

The format follows a human-readable changelog convention, with sections organized by release version and change type.

Any contributor making changes to the project must record their changes in this file under the appropriate release section, using the established format and change type categories _(Features, Fixes, Optimizations, Tests, BREAKING CHANGES, etc.)_.

## TOC

**- Milestone.1 "MVP" -**

- [Unreleased](#unreleased)
- [Release 0.3.0 (Unreleased)](#release-030-unreleased)
- [Release 0.2.2 (2026-07-10)](#release-022-2026-07-10)
- [Release 0.2.1 (2026-07-01)](#release-021-2026-07-01)
- [Release 0.2.0 (2026-06-30)](#release-020-2026-06-30)
- [Release 0.1.9 (2026-05-29)](#release-019-2026-05-29)
- [Release 0.1.8 (2026-05-18)](#release-018-2026-05-18)
- [Release 0.1.7 (2026-05-04)](#release-017-2026-05-04)
- [~~Yanked 0.1.6 (2026-04-24)~~](#yanked-016-2026-04-24)
- [Release 0.1.5 (2026-04-12)](#release-015-2026-04-12)
- [Release 0.1.4 (2026-04-06)](#release-014-2026-04-06)
- [Release 0.1.3 (2026-04-01)](#release-013-2026-04-01)
- [Release 0.1.2 (2026-03-31)](#release-012-2026-03-31)
- [Release 0.1.1 (2026-03-29)](#release-011-2026-03-29)
- [Release 0.1.0 (2026-03-29)](#release-010-2026-03-29)

---

## Contents

### Unreleased

#### Fixes:

None

#### Optimizations:

None

#### Features:

None

#### **BREAKING CHANGES** (API CHANGES):

None

---

### Release 0.3.0 (Unreleased)

#### Fixes:

None

#### Optimizations:

1. **[`macros`]** Updated `route!` macro to use `Routable` trait instead of `Grouped` trait for error conversion, making the semantics clearer. The `route!` macro now calls `::mingling::Routable::to_chain(e)` on the error branch instead of `::mingling::Grouped::to_chain(e)`.

    Additionally, `ChainProcess<ThisProgram>` now implements `Routable<ThisProgram>`, allowing `route!` to work with `Result<Ok, ChainProcess<ThisProgram>>` patterns — where the error side is already a `ChainProcess` value that should be routed directly:

    When `ChainProcess` implements `Routable`, `to_chain()` re-routes the inner `AnyOutput` to the chain pipeline (preserving the existing `NextProcess::Chain`/`Renderer` flag), while `to_render()` re-routes it to the render pipeline. This enables seamless propagation of already-routed chain process values through the `route!` macro without double-wrapping.

    The `Routable` trait is defined in `mingling_core::asset::routable` and provides unified routing capabilities (`to_chain` / `to_render`) for any type that can be dispatched into the program's pipeline. A blanket implementation is provided for all `T: Grouped<C> + Send`, ensuring backward compatibility — existing types that implement `Grouped` automatically implement `Routable`.

2. **[`macros`]** Restructured the `mingling_macros` crate's internal module hierarchy. The previously flat module structure has been reorganized into a logical directory-based layout:

    - **`attr/`** — Attribute macro implementations (e.g., `#[chain]`, `#[renderer]`, `#[help]`, `#[completion]`, `#[dispatcher_clap]`, `#[program_setup]`)
    - **`derive/`** — Derive macro implementations (e.g., `#[derive(Grouped)]`, `#[derive(EnumTag)]`)
    - **`func/`** — Function-like macro implementations (e.g., `pack!`, `group!`, `dispatcher!`, `suggest!`, `entry!`, `node!`, `gen_program!` and its sub-macros)
    - **`systems/`** — Cross-cutting systems (e.g., resource injection, dispatch tree generation, structural data derive support)
    - **`extensions/`** — Extension point mechanism for attribute macros (unchanged)
    - **`utils.rs`** — Shared utility module for future common helpers

    All public API items (`#[proc_macro]`, `#[proc_macro_derive]`, `#[proc_macro_attribute]`) remain at the crate root (`lib.rs`) with identical signatures. Internal function visibility has been tightened from `pub fn` to `pub(crate) fn` for all module-internal functions that were previously publicly accessible only within the crate.

    _No migration is required for downstream code — all macros are re-exported with the same names and signatures as before._

3. **[`macros`]** Refactored the code generation strategy for `#[chain]`, `#[renderer]`, `#[help]`, and `#[completion]` attribute macros. Instead of inlining the user's function body directly into the generated trait implementation, these macros now **preserve the original user function as a standalone item** and generate trait method implementations that **call the original function by name**, injecting resources from the application context.

    This approach provides several benefits:

    - **Better debugging and error messages** — The user's function exists as a named, callable item. Stack traces, profiler output, and error messages reference the user's function name (e.g., `handle_greet`) rather than an anonymous closure or inlined block, making debugging more intuitive.

    - **Clearer macro expansion** — The generated code maintains a clear separation between the original user function and the trait glue code, reducing the cognitive load when inspecting macro-expanded output.

    _No migration is required for downstream code — the behavior of all four macros is unchanged. This is purely an internal code generation refactoring._

4. **[`core`]** Changed the `exec_without_render` and `exec` methods' behavior: the `print!`-based output with manual stdout flushing has been replaced by a call to `result.std_print()`, which handles buffered output internally via the new `RenderResult` buffer system (introduced in **BREAKING CHANGE #6**). The `render_output` gating still applies — when `stdout_setting.render_output` is `false`, nothing is printed — but the `!result.is_empty()` guard has been removed: `std_print()` now handles empty results appropriately.

    Previously, when `render_output` was true but `result.is_empty()` was also true, the old code would skip printing entirely and return the exit code from the result. Now, `std_print()` is called unconditionally when `render_output` is true, and the exit code is read from the result regardless of whether any output was printed. This ensures that programs which set a non-zero exit code without producing renderable output (e.g., via `ExitCodeSetup` or `ProgramControlUnit::OverrideExitCode`) will exit with the correct code.

5. **[`core`]** Made `with_resource`, `with_dispatcher`, `with_dispatchers`, `with_hook`, and `with_setup` methods return `&mut Self` instead of `()`. These methods now return a mutable reference to the program instance, enabling ergonomic chaining:

    ```rust
    // Before — required separate statements
    program.with_resource(ResConfig::new());
    program.with_resource(ResDatabase::new());
    program.with_setup(BasicProgramSetup);
    program.with_hook(logging_hook);

    // After — supports method chaining
    program
        .with_resource(ResConfig::new())
        .with_resource(ResDatabase::new())
        .with_setup(BasicProgramSetup)
        .with_hook(logging_hook);
    ```

    Affected methods:
    - `Program::with_resource(&mut self, res: Res) -> &mut Self` — Insert a resource into the program's global resource store.
    - `Program::with_dispatcher(&mut self, dispatcher: Disp) -> &mut Self` — Add a single dispatcher to the program.
    - `Program::with_dispatchers(&mut self, dispatchers: D) -> &mut Self` — Add multiple dispatchers to the program.
    - `Program::with_hook(&mut self, hook: ProgramHook<C>) -> &mut Self` — Add a lifecycle hook to the program.
    - `Program::with_setup(&mut self, setup: S) -> &mut Self` — Load and execute a program setup.

    _No behavioral changes — all existing functionality is preserved. Downstream code that ignores the return value continues to work without modification._

#### Features:

1. **[`core`]** Added `RenderResult::new()` method for creating a new `RenderResult` with default values (empty text and exit code 0). This provides a more explicit and discoverable constructor compared to `RenderResult::default()`, making it clearer when a fresh result is being created for use with `write!`/`writeln!`.

    ```rust
    let mut result = RenderResult::new();
    writeln!(result, "Hello!").ok();
    result
    ```

    The method is equivalent to `RenderResult::default()` but serves as a more idiomatic entry point for renderer functions.

2. **[`core`]** **[`macros`]** Added `core` and `macros` features to the `mingling` crate, both enabled by default. These features allow selective exclusion of `mingling_core` and `mingling_macros` dependencies respectively.

    - When `core` is **disabled** (`default-features = false, features = ["macros"]`), the `mingling` crate only re-exports proc-macros and macro-related items, without linking `mingling_core`.
    - When `macros` is **disabled** (`default-features = false, features = ["core"]`), the `mingling` crate only provides runtime types and traits without any proc-macro re-exports.
    - When **both are disabled** (`default-features = false`), the `mingling` crate provides a minimal re-export surface.

    This enables downstream projects to fine-tune dependency weight — e.g., a library crate that only uses `mingling_macros` can disable `core` to avoid pulling in unused runtime types.

3. **[`picker`]** Added the new `picker` system — a **non‑breaking companion** to the existing `parser` feature, not a replacement. The `picker` feature provides a chained argument parsing API and a parsing function library (`parselib`) for analyzing command-line argument structure. Key components:

    - **Chained Argument Parser with `.pick()`** — A chained-call API for declaring and extracting arguments:

        ```rust
        use arg_picker::prelude::*;

        let args: Vec<&str> = vec!["--name", "Bob", "--age", "24"];

        let (name, age) = args
            .pick(&arg![name: String])
            .or(|| "Alice".to_string())
            .pick(&arg![age: i32])
            .or(|| 24)
            .post(|num| num.clamp(0, 120))
            .unwrap();
        ```

    - **Pure function library `parselib`** — Provides utility functions for analyzing command-line argument structure:

        ```rust
        use arg_picker::parselib::*;
        ```

    The `picker` system is **additive** — the old `parser` feature remains fully functional and is **not deprecated**. Users may choose either or both systems in the same project. The `picker` feature is available both as the `mingling/picker` feature (enabled by default) and as a standalone `arg_picker` crate.

    _No migration is required for existing `parser` users — the old API continues to work unchanged._

4. **[`core`]** Added multiple `From` implementations for `RenderResult`:

    - **From `()`** — Allows constructing an empty `RenderResult` from a unit value, enabling ergonomic returns like `fn my_renderer() -> RenderResult { }` (via `}` → `}` with implicit `()` return).
    - **From integer types** (`i32`, `i16`, `i8`, `u32`, `u16`, `u8`, `usize`) — Allows constructing a `RenderResult` with a specific exit code and empty text, enabling `fn my_renderer() -> RenderResult { 0 }` or `42.into()`.
    - **From `String`**, **`&String`**, and **`&str`** — Allows constructing a `RenderResult` with the given text and exit code `0`, enabling `fn my_renderer() -> RenderResult { "Hello".into() }` or passing a `String` directly.

    These implementations make `RenderResult` more flexible as a return type, allowing renderer functions to return simple values without manually constructing a `RenderResult` via `new()` and `write!`/`writeln!`.

5. **[`macros:renderer`]** Removed the restriction that `#[renderer]` functions must return `RenderResult`. The `#[renderer]` macro now accepts any return type (including no return type), and automatically converts the return value to `RenderResult` via `Into::into`.

    - Functions returning `RenderResult` work as before.
    - Functions returning other types (e.g., `String`, `i32`, `()`) are converted via the `Into<RenderResult>` trait.
    - Functions with no return type (`-> ()` or omitted) return `()` which is converted to an empty `RenderResult` via `From<()>`.

    This makes `#[renderer]` more flexible and consistent with the ergonomic `From` implementations added in item 4 above.

    ```rust
    #[renderer]
    fn render_greeting(prev: ResultGreeting) -> String {
        format!("Hello, {}!", *prev)
    }

    #[renderer]
    fn render_exit_code(prev: ResultExit) -> i32 {
        42
    }

    #[renderer]
    fn render_void(prev: ResultVoid) {
        // side effects only, returns empty RenderResult
    }
    ```

6. **[`macros:help`]** Removed the restriction that `#[help]` functions must return `::mingling::RenderResult`. The `#[help]` macro now accepts any return type (including no return type), and automatically converts the return value to `RenderResult` via `Into::into`.

    - Functions returning `RenderResult` work as before.
    - Functions returning other types (e.g., `String`, `i32`, `()`) are converted via `Into<RenderResult>`.
    - Functions with no return type (`-> ()` or omitted) return `()` which is converted to an empty `RenderResult` via `From<()>`.

    This makes `#[help]` consistent with the `#[renderer]` macro's ergonomic return type handling introduced in item 5 above.

    ```rust
    #[help]
    fn help_greeting(prev: EntryGreeting) -> String {
        format!("Displaying help for greeting: {}", *prev)
    }

    #[help]
    fn help_void(prev: EntryVoid) {
        // side effects only, returns empty RenderResult
    }
    ```

7. **[`setups`]** Refactored `BasicProgramSetup`, `HelpFlagSetup`, `QuietFlagSetup`, `ConfirmFlagSetup`, `StructuralRendererSetup`, and `StructuralRendererSimpleSetup` into the `picker` subsystem under `mingling::setups::picker`. These setups now use the `arg_picker` (`picker`) chained argument parsing API internally instead of directly manipulating global arguments.

    - The `BasicProgramSetup`, `HelpFlagSetup`, `QuietFlagSetup`, and `ConfirmFlagSetup` structs now use `PickerArg<Flag>` and chained `.pick()` calls to detect flags from the argument list, replacing the previous `global_argument`-based approach.
    - The `StructuralRendererSetup` struct now uses `PickerArg<Flag>` constants (e.g., `JSON_FLAG`, `YAML_FLAG`) and chained `.pick()` calls to detect format-specifying flags, replacing the previous `global_argument` approach.
    - The `StructuralRendererSimpleSetup` struct still uses the legacy `global_argument("--renderer", ...)` approach, preserving backward compatibility with the `--renderer <FORMAT>` syntax.
    - New `PickerArg<Flag>` constants have been added in `mingling::setups::picker::consts`: `HELP_FLAG`, `QUIET_FLAG`, `CONFIRM_FLAG`, `JSON_FLAG`, `JSON_PRETTY_FLAG`, `YAML_FLAG`, `TOML_FLAG`, `RON_FLAG`, and `RON_PRETTY_FLAG`. The format-specific flags are feature-gated behind their respective `json_serde_fmt`, `yaml_serde_fmt`, `toml_serde_fmt`, and `ron_serde_fmt` features.
    - The module structure is:
        - `mingling::setups::picker` — re-exports all picker-based setup types
        - `mingling::setups::picker::basic` — `BasicProgramSetup`, `HelpFlagSetup`, `QuietFlagSetup`, `ConfirmFlagSetup`
        - `mingling::setups::picker::consts` — reusable `PickerArg<Flag>` constants
        - `mingling::setups::picker::structural_renderer` — `StructuralRendererSetup`, `StructuralRendererSimpleSetup`
    - All setup types remain available from `mingling::setups::*` as before — this is purely an internal refactoring; no public API surface changes.

    The `picker` feature must be enabled for these refactored setups to be available. When the feature is disabled, the original implementations (using `global_argument`) remain in effect.

8. **[`core`]** Added `get_args_mut()`, `take_args()`, and `replace_args()` methods to `Program` for more flexible argument manipulation:

    - **`get_args_mut(&mut self) -> &mut [String]`** — Returns a mutable reference to the program's command-line arguments, allowing in-place modification of individual arguments.
    - **`take_args(&mut self) -> Vec<String>`** — Takes ownership of the program's command-line arguments, replacing them with an empty `Vec`. Useful for transferring arguments to another context or processing them with ownership.
    - **`replace_args(&mut self, args: Vec<String>) -> Vec<String>`** — Replaces the program's command-line arguments with a new set and returns the old ones. Enables swapping argument sets during program execution.

    These methods complement the existing read-only `get_args(&self)` method, providing full control over argument mutation and ownership.

9. **[`macros:chain`]** Relaxed the `#[chain]` return type validation. Previously, `#[chain]` functions were restricted to returning `Next`, `ChainProcess<ThisProgram>`, `()`, or omitting the return type. Now, any return type is accepted, and the generated `proc` function performs an explicit `Into<ChainProcess<ThisProgram>>` conversion using a fully-qualified turbofish based on the user-declared return type.

    This means `#[chain]` functions can now return any pack type directly, without needing an explicit `.into()` call in the function body:

    ```rust
    // Before — required explicit .into()
    #[chain]
    fn handle_greet(args: EntryGreet) -> Next {
        let name = /* ... */;
        ResultGreeting::new(name).into()
    }

    // After — return any pack type directly
    #[chain]
    fn handle_greet(args: EntryGreet) -> ResultGreeting {
        let name = /* ... */;
        ResultGreeting::new(name)
    }
    ```

    The generated `proc` function now wraps the body result in `<UserReturnType as Into<ChainProcess<ThisProgram>>>::into(...)`, which:
    - Works for `Next` / `ChainProcess` via the identity `From<T> for T` implementation.
    - Works for any pack type (`ResultGreeting`, etc.) via the `.into()` conversion generated by `pack!` / `#[derive(Grouped)]`.
    - Works for `()` via the `From<()>` implementation on `ChainProcess`.

    The return type validation has been removed entirely — any valid Rust return type is accepted. If the type does not implement `Into<ChainProcess<ThisProgram>>`, a standard Rust compilation error will be produced at the call site.

10. **[`macros`]** **[`extensions`]** Added the `extensions` module to `mingling_macros`, providing an extension point mechanism for attribute macros (`#[chain]`, `#[renderer]`, `#[help]`, `#[completion]`). The extension system allows identifiers in the attribute argument to be extracted and processed before the main macro logic runs.

    Each attribute macro now attempts to redispatch through `extensions::try_redispatch_simple()` (or `try_redispatch_completion` for `#[completion]`) before executing its standard logic. If extension identifiers are detected, the call is re-routed so that extensions are applied via additional `#[...]` attributes stacked on top of the inner core attribute. New extensions can be added without modifying the attribute macros themselves — only the `extensions` module needs to be updated to register new identifiers.

    This system is designed for future extensibility: as new cross-cutting concerns (e.g., logging, metrics, validation) are identified, they can be added as simple extension identifiers without touching the core macro logic.

11. **[`extensions`]** **[`macros`]** Added the `#[routeify]` extension attribute macro that transforms `expr?` into `route!(expr)`, enabling concise error routing in chain functions using the `?` operator syntax.

    The `#[routeify]` macro can be used:
    - **Standalone** — as a direct attribute: `#[routeify] fn handle(...) { ... }`
    - **As an extension** — via the extension point system: `#[chain(routeify)] fn handle(...) { ... }`

    When used as a `#[chain]` extension, the `routeify` identifier is detected by the extension point mechanism, stripped from the `#[chain]` attribute arguments, and `#[routeify]` is applied as an outer attribute on top of `#[chain]`. The re-dispatch token stream now correctly generates `#[#exts]*` (i.e., `#[routeify] #[chain]`) instead of the previous bare `#exts` — fixing a bug where extension identifiers were emitted without the `#[...]` attribute delimiter, producing invalid token streams.

    ```rust
    use mingling::macros::routeify;

    #[chain(routeify)]
    fn handle_calc(args: EntryCalculate) -> Next {
        let a = args.pick(&arg![f32]).to_result()?;
        let op = args.pick(&arg![Operator]).to_result()?;
        StateCalculate { number_a: a, operator: op, ... }.to_chain()
    }
    ```

    The `#[routeify]` macro is feature-gated behind `extra_macros` and re-exported as `mingling::macros::routeify`.

    Internal changes:
    - Added `mingling_macros/src/extensions/routeify.rs` with `routeify_impl` implementation.
    - Updated `try_redispatch_simple` and `try_redispatch_completion` to emit `#[#exts]` instead of bare `#exts`, ensuring proper attribute syntax in the re-dispatched token stream.
    - Registered `#[proc_macro_attribute] pub fn routeify` in `mingling_macros/src/lib.rs`.

12. **[`core`]** **[`macros`]** Added the `r_append!` macro and `RenderResult::append_other()` method for appending the contents of one `RenderResult` to another. The `append_other()` method on `RenderResult` merges the buffered content (text and output modes) from another `RenderResult` into the current one. If the destination result has `immediate_output` enabled but the source does not, the source's content will be immediately flushed to the appropriate output stream (stdout/stderr) while also being appended to the render buffer. The `exit_code` of the source result is **not** transferred — only the buffered content and the `immediate_output` flag are merged.

    The `r_append!` macro supports two usage forms:
    - **Explicit buffer** — `r_append!(dst, src)` appends the contents of `src` into the `dst` `RenderResult`.
    - **Implicit buffer** — `r_append!(src)` appends the contents of `src` into the implicit `__render_result_buffer` (available inside `#[buffer]` functions).

    The macro is re-exported as `mingling::macros::r_append` and included in `mingling::prelude::*`.

#### **BREAKING CHANGES** (API CHANGES):

1. **[`macros:renderer`]** **[`macros:help`]** Removed `r_println!` and `r_print!` macros from being implicitly injected by `#[renderer]` and `#[help]` macros. These macros still exist, but must now be used **explicitly** — either with an explicit buffer argument, or via the `#[buffer]` extension attribute that re-enables implicit buffer injection.

    **Option A — Explicit buffer:** Pass a `RenderResult` variable as the first argument:

    ```rust
    use mingling::macros::r_println;
    use mingling::prelude::*;

    #[renderer]
    fn render_greeting(greeting: ResultGreeting) -> RenderResult {
        let mut result = RenderResult::new();
        r_println!(result, "Hello, {}!", *greeting);
        result
    }
    ```

    **Option B — Implicit buffer via `#[buffer]` extension:** Use `#[renderer(buffer)]` to re-enable the old implicit injection behavior, where a `RenderResult` buffer is automatically created and `r_println!`/`r_print!` write to it without an explicit argument. This requires the function to return `()` (unit); the expanded function will return `RenderResult`:

    ```rust
    use mingling::macros::r_println;

    #[renderer(buffer)]
    fn render_greeting(greeting: ResultGreeting) {
        r_println!("Hello, {}!", *greeting);
        // Returns RenderResult implicitly
    }
    ```

    The `#[buffer]` extension is also available standalone as `#[buffer]` for use outside of `#[renderer]`/`#[help]` functions.

2. **[`macros:chain`]** The `#[chain]` macro's return type requirement has been relaxed. Previously, chain functions were required to return `Next` or `()` (with `()` auto-converting to `ResultEmpty`). Now, chain functions can also return `ChainProcess<ThisProgram>` directly, or omit the return type entirely (which defaults to `()` → `ResultEmpty`).

    The return value of chain functions is now wrapped in an explicit `.into()` call inside the generated `proc` function, ensuring consistent conversion to `ChainProcess<ProgramType>`. As a result, **all downstream code that previously relied on implicit conversion from packed types to `Next`/`ChainProcess` must now call `.into()` explicitly**.

    ```rust
    // Before — implicit conversion worked because the generated proc
    // function was `fn proc(...) -> impl Into<ChainProcess<...>>`
    #[chain]
    fn handle_greet(args: EntryGreet) -> Next {
        let name = /* ... */;
        ResultGreeting::new(name) // implicitly converted
    }

    // After — the generated proc function is `fn proc(...) -> ChainProcess<...>`,
    // so the body must produce ChainProcess explicitly
    #[chain]
    fn handle_greet(args: EntryGreet) -> Next {
        let name = /* ... */;
        ResultGreeting::new(name).into() // explicit conversion required
    }
    ```

    The key advantage of this design is that **the original function body and the expanded `proc` function body are now identical** — the macro only adjusts the function signature and inserts an outermost `.into()` wrapper, without rewriting the internal return expressions. This means the semantics of the original code are perfectly preserved: there is no invisible type coercion happening mid-body, and the behavior you write in the source is exactly what executes at runtime. If a bug arises, the expanded code mirrors the source almost one-to-one, making debugging straightforward.

    This change also applies to:
    - Chain functions returning `()` (unit), where the body's final expression with `.into()` is replaced by an explicit `ResultEmpty::to_chain()` call.
    - Chain functions using `&mut` resource injection with non-unit returns: the inner closure now calls `__modify_res_and_return_route` (which returns `ChainProcess<C>` directly) instead of relying on `.into()` conversion.
    - The `__modify_res_and_return_route` method signature changed from accepting `impl Into<ChainProcess<C>>` to returning `ChainProcess<C>` directly.

    All examples, docs, and test cases across the repository have been updated to use `.into()` where packed types are returned from chain functions.

3. **[`core`]** **[`ExitCodeSetup`]** Updated `ExitCodeSetup` to only override the exit code when `ResExitCode` has been modified (i.e., `exit_code != 0`). Previously, it unconditionally overrode the exit code, which could interfere with exit codes set by other hooks or the program's default exit flow. The `on_finish` hook now returns `ProgramControlUnit::OverrideExitCode(...)` only when the exit code is non-zero, and `ProgramControls::Empty` otherwise. The import of `ProgramControls` has been added accordingly.

4. **[`core`]** **[`macros`]** Renamed `Groupped` (typo) to `Grouped`. All references to the trait, derive macro, module files, and related types have been corrected throughout the codebase:

    - Trait: `Groupped<Group>` → `Grouped<Group>`
    - Derive macro: `#[derive(Groupped)]` → `#[derive(Grouped)]`
    - Serialize variant: `GrouppedSerialize` → `GroupedSerialize`
    - Source files: `groupped.rs` → `grouped.rs`
    - Pattern matcher: `GrouppedDerivePattern` → `GroupedDerivePattern`
    - All `use` imports, type annotations, and trait bound references updated accordingly.

    This is a pure rename — no behavioral changes. All functionality remains identical. Downstream code using the old `Groupped` name must migrate to `Grouped`.

5. **[`core`]** **[`macros`]** Removed `to_chain()` and `to_render()` default methods from the `Grouped` trait. These methods are now exclusively provided by the `Routable` trait. All code that previously called `to_chain()` or `to_render()` via `Grouped` must now call them via `Routable`:

    ```rust
    // Before (via Grouped — removed)
    use mingling::Grouped;
    my_value.to_chain();

    // After (via Routable)
    use mingling::Routable;
    my_value.to_chain();
    ```

    - The `Routable` trait is re-exported in `mingling::prelude` alongside `Grouped`.
    - The blanket implementation `impl<T: Grouped<C> + Send> Routable<C> for T` remains, so all types that implement `Grouped` still have `to_chain()` and `to_render()` — they just need to import `Routable` instead of relying on `Grouped` for those methods.
    - Internal macro-generated code (in `#[chain]`, `#[renderer]`, `#[dispatcher_clap]`, `gen_program!`, `empty_result!`, etc.) has been updated to reference `::mingling::Routable::<C>::to_chain(...)` / `::mingling::Routable::<C>::to_render(...)` instead of `::mingling::Grouped::<C>::to_chain(...)` / `::mingling::Grouped::<C>::to_render(...)`.
    - Downstream crates using `mingling` macros are automatically migrated — the macro output now references `Routable`. Only manual `.to_chain()` / `.to_render()` calls in user code need updating (add `use mingling::Routable;`).

    _No behavioral changes — this is a pure API migration from `Grouped` to `Routable` for routing methods._

6. **[`core`]** **[BREAKING]** Removed the `Deref<Target = str>` implementation from `RenderResult`. Previously, `RenderResult` implemented `Deref<Target = str>`, delegating to the internal `render_text: String` field. This implementation has been removed as part of the internal refactoring of `RenderResult` from a single `String` field to a `Vec<(String, RenderResultMode)>` buffer.

    The `RenderResult` struct has been restructured internally:

    - **Removed:** `render_text: String` — replaced by a buffered storage format
    - **Added:** `render_buffer: Vec<(String, RenderResultMode)>` — a list of (text, output-mode) pairs
    - **Added:** `immediate_output: bool` — flag for real-time flushing

    **New `RenderResultMode` enum** (`Stdout` / `Stderr`) has been added to distinguish output streams at the buffer level. Both variants are re-exported as `mingling::RenderResultMode`.

    **New methods added to `RenderResult`:**

    - `append_to_buffer(text, mode)` / `append_line_to_buffer(text, mode)` — Append text with an explicit output mode (`Stdout` or `Stderr`)
    - `eprint(text)` / `eprintln(text)` — Append text marked for stderr output
    - `immediate_output()` — Enable real-time output flushing
    - `std_print()` — Flush all buffered content to stdout/stderr
    - `len()` / `is_empty()` — Character count and emptiness check (based on the buffer)
    - `trim_buffer(self) -> RenderResult` — Trim whitespace from the buffer ends, returning a new `RenderResult`

    **`r_eprint!` and `r_eprintln!` macros** have been added to `mingling_macros` and re-exported via `mingling::macros::r_eprint` / `mingling::macros::r_eprintln` and `mingling::prelude::*`. These work analogously to `r_print!` / `r_println!` but target stderr output (calling `RenderResult::eprint` / `RenderResult::eprintln` under the hood). Both support implicit buffer mode (inside `#[buffer]` functions) and explicit buffer mode (passing a `RenderResult` as the first argument).

        The `Display` implementation for `RenderResult` is now: `write!(f, "{}", render_result_to_string(self).trim())` — this trims leading and trailing whitespace from the rendered output, making formatting more predictable and avoiding stray newlines or spaces in display contexts.

    **Migration guide:**

    Code that relied on `Deref<Target = str>` (e.g., `&*result`, `result.as_ref()`, or passing `&RenderResult` where `&str` was expected) must be updated to use one of the following:

    ```rust
    // Before — relied on Deref
    fn takes_str(s: &str) { /* ... */ }
    takes_str(&result);

    // After — convert explicitly
    let result_string: String = result.to_string();
    takes_str(&result_string);

    // Or use the Display impl directly
    println!("{}", result);
    ```

    The `is_empty()` method now checks the buffer length (in characters) rather than checking `render_text.is_empty()`. The `Display` implementation no longer adds a trailing newline (previously `writeln!` was used; now `write!` is used) — existing code that relied on the trailing newline in `Display` may need adjustment. Additionally, the `to_string()` call on `RenderResult` now trims leading and trailing whitespace from the rendered text via the `Display` implementation, whereas previously the raw content was preserved without trimming.

    All examples and internal usages have been updated across the codebase to reflect these changes (e.g., `repl_basic_setup` now calls `println!("{}", r.result)` instead of `println!("{}", r.result.trim())`, since `Display` no longer adds a trailing newline).

---

## Release 0.2.2 (2026-07-10)

### Unreleased

#### Fixes:

1. **[`macros:structural_data`]** Fixed `group_structural!` macro to correctly generate `use` statements for multi-segment type paths (e.g., `crate::MyType`). Previously, when an aliased type had only one segment, the macro used the alias name (`type_name`) instead of the original type's last segment for `super::` imports, which could cause compilation errors. Now correctly extracts the last segment from the original `type_path` for single-segment fallback imports.

#### Optimizations:

None

#### Features:

None

#### **BREAKING CHANGES** (API CHANGES):

None

---

### Release 0.2.1 (2026-07-01)

#### Fixes:

1. **[`macros`]** Fixed false positives in `entry_has_variant` caused by bare substring matching in the third `contains` check.
   When a longer variant (e.g., `EntryListAlias`) is registered first, followed by a shorter variant that shares the same prefix (e.g., `EntryList`),
   `"=> EntryList"` would incorrectly match as a substring of `"=> EntryListAlias,"`, causing a false duplicate registration detection.
   Now changed to use `find` + trailing character boundary validation, ensuring the character immediately after the match is not an identifier character (letter/digit/underscore).

    Affected scope: Deduplication logic for `#[chain]`, `#[renderer]`, `#[help]`, and `#[completion]` registration.

#### Optimizations:

None

#### Features:

None

#### **BREAKING CHANGES** (API CHANGES):

None

---

### Release 0.2.0 (2026-06-30)

> [!IMPORTANT]
> Starting from 0.2.0, Mingling's GitHub repository has been migrated from [catilgrass/mingling](https://github.com/catilgrass/mingling) to [mingling-rs/mingling](https://github.com/mingling-rs/mingling).
>
> Please note the change in repository address;
>
> the old address is no longer maintained, and all new Issues, PRs, and Releases will be conducted in the new repository.

#### Tests:

1. **[`core`]** - **Added complete unit test coverage**, adding `#[cfg(test)]` test modules for 23 modules in `mingling_core` that previously lacked tests, covering:

    - **Core types** (`any.rs`): `AnyOutput` creation, downcast, type judgment, route routing, restore deserialization; `ChainProcess` type conversion; `NextProcess` formatting
    - **Dispatcher** (`dispatcher.rs`): Conversion of `Dispatchers` from 1~7 tuples, Vec, Box; Deref dereferencing; clone behavior
    - **Node** (`node.rs`): Construction, join, kebab-case conversion, equality comparison, sorting
    - **Global resource** (`global_resource.rs`): `GlobalResource` new, Deref, AsRef; three default implementations of `ResourceMarker`
    - **Lazy resource** (`lazy_resource.rs`): Coverage of all 18+ methods of `LazyRes`, including initialization triggering, get_ref/get_mut/get_clone, into_inner/unwrap, Drop callback, `ResourceMarker` integration
    - **Error types** (`chain/error.rs`, `program/error.rs`): All Display, Error source, From conversions
    - **Configuration structs** (`config.rs`): Default values for `ProgramStdoutSetting`, `ProgramUserContext`; FromStr parsing and Display output of `StructuralRendererSetting` (feature-gated)
    - **Flag system** (`flag.rs`): Added 8 From conversions, Deref, AsRef for `Flag`
    - **String wrapper** (`string_vec.rs`): 6 From conversions, Deref, Into\<Vec\>
    - **Render result** (`render_result.rs`): print/println/clear/is_empty, Write trait, Display, Deref, From conversions
    - **Render error** (`structural/error.rs`): Construction, From, Deref, Into\<String\>
    - **Structural renderer** (`structural.rs`): Rendering in Disable/JSON/YAML/TOML/RON formats (feature-gated)
    - **Completion suggestions** (`suggest.rs`): All construction, access, modification, sorting, and conversion methods for `Suggest` and `SuggestItem`
    - **Shell context** (`shell_ctx.rs`): Added `filling_argument`, `filling_argument_first`, `typing_argument`, `strip_typed_argument`, `get_typed_arguments`
    - **Hook system** (`hook.rs`): `ProgramHook::empty` and all 8 builder methods
    - **Singleton management** (`single_instance.rs`): `ProgramCell` set/get_raw/take/double-set-panic
    - **Program setup** (`setup.rs`): Verification of `with_setup` invocation
    - **Completion detection** (`comp_ctx.rs`): Three scenarios for `is_completing`
    - **Build script** (`builds/comp.rs`): `get_tmpl` for four Shells and Other fallback

2. **[`core`]** - **Added 6 integration test crates**, testing public APIs under different feature combinations:

    - `test-basic`: Basic type tests with default features (Node, Flag, RenderResult, NextProcess, StringVec)
    - `test-comp`: ShellContext, Suggest, SuggestItem, is_completing with `comp + builds` features
    - `test-structural-renderer`: StructuralRenderer output in various formats with `structural_renderer_full + parser` features
    - `test-repl`: ResREPL and basic types with `repl + extra_macros` features
    - `test-dispatch-tree`: Basic types with `dispatch_tree` feature
    - `test-all`: Comprehensive testing with all feature combinations (ShellContext, Suggest, ResREPL, StructuralRenderer, Hooks, basic types, etc.)

    These crates are located in `mingling_core/tests/test-*/`, each marked as an independent workspace via `[workspace]`, isolated from the main workspace.

3. **[`workspace`]** - **Added workspace exclude rules for the 6 test crates in the root `Cargo.toml`**, ensuring that integration test crates are not captured by the workspace's implicit member rules.

#### Fixes:

1. **[`core:comp`]** Fixed `default_completion` incorrectly handling multi-level subcommand suggestions when the cursor is after a trailing space. `all_words.get(1..word_index)` could go out of bounds because Zsh's `$CURRENT` (`word_index`) may exceed `all_words.len()` when trailing whitespace is present. The range end is now capped with `.min(all_words.len())`

2. **[`core:comp`]** Fixed `default_completion` jumping to the next subcommand level on partial input (e.g. typing `b` for `bind` would skip `bind` and directly suggest third-level commands `add`/`ls`/`rm`). Now if the last input word is only a partial match (`starts_with` but not equal), the current-level word is suggested instead of skipping ahead

3. **[`core`]** Replaced `OnceLock<Option<Box<dyn Any>>>` with a custom `ProgramCell` type backed by `UnsafeCell` and `AtomicBool`. The new `ProgramCell` replaces `OnceLock`'s `get_or_init` / `get` / `as_ref` calls with a direct `set` / `get_raw` / `take` API. This change:
    - Eliminates the double indirection (`OnceLock<Option<Box<...>>>` → `UnsafeCell<Option<Box<...>>>`)
    - Allows the program instance to be **taken** (moved out) via an `unsafe fn take()` after execution completes, enabling proper cleanup before `std::process::exit()` in `exec_and_exit`
    - Is paired with corresponding simplifications in `once_exec.rs` and `repl_exec.rs` that switch from `THIS_PROGRAM.get().unwrap().as_ref()` to `THIS_PROGRAM.get_raw().unwrap()`

4. **[`macros:dispatcher_clap`]** Added `dispatch_tree` feature integration for `#[dispatcher_clap]`. When the `dispatch_tree` feature is enabled, `#[dispatcher_clap]` will now automatically register the dispatcher and entry in the dispatch tree via `register_dispatcher!`, matching the behavior already present in the `dispatcher!` macro. When the feature is disabled, no additional code is generated.

5. **[`macros`]** The four macros `#[chain]`, `#[renderer]`, `#[help]`, and `#[completion]` now support using fully qualified type paths with `::` (e.g. `crate::EntryFine`) as type inputs. Previously, these macros required types to be bare single-segment idents (e.g. `EntryFine`), rejecting reasonable paths like `crate::EntryFine`. Specific changes:

    - `res_injection::extract_args_info` (shared by `#[chain]` and `#[renderer]`): Removed the single-segment validation for the first parameter type
    - `#[renderer]` / `#[help]`: Removed respective `check_single_segment_type` calls
    - `#[completion]`: Attribute parameter parsing changed from `Ident` to `TypePath`, supporting `#[completion(crate::EntryFine)]`
    - Fixed code generation in `build_chain_arm`, `build_chain_exist_arm`, `build_renderer_entry`, `build_renderer_exist_entry`, `build_general_renderer_entry`, and completion entry: `Self::#variant` match arms now only take the last segment ident of the type path (e.g. `Self::EntryFine`), rather than concatenating the full path directly (which would generate invalid syntax like `Self::crate::EntryFine`), while `downcast::<T>()` and `type Previous = T` still use the full path to ensure correct type resolution

6. **[`macros:register`]** Added compile-time duplicate variant detection for chain, renderer, help, and completion registrations. When two `#[chain]` (or `#[renderer]`, `#[help]`, `#[completion]`) functions register the same type variant, the compiler now emits a clear error at the registration site (e.g. `fn handle_state_prev1(_p: StatePrev1)`) instead of silently producing an unreachable match arm that only manifests as dead code in the generated `do_chain()`/`render()` dispatch.

    Affected registration points:
    - `register_chain` — checks `CHAINS` set for existing entries with the same variant
    - `register_renderer` — checks `RENDERERS` set
    - `help_attr` (via `#[help]`) + `register_help` — checks `HELP_REQUESTS`; `register_help` also serves as a public escape hatch for manual help registration, automatically skipping the duplicate check when the exact same entry was pre-inserted by `#[help]`
    - `completion_attr` (via `#[completion]`) — checks `COMPLETIONS` set

7. **[`macros:dispatch_tree`]** Fixed the static name generation for dispatch tree nodes to use `snake_case` conversion instead of simple `.` → `_` replacement, and fixed the `__comp` completion dispatcher static name from `__internal_dispatcher___comp` (triple underscore) to `__internal_dispatcher_comp` (double underscore), resolving a mismatch between the name generated by `register_dispatcher!` and the name used in `program_comp_gen`.

8. **[`core`]** Changed the `exec_without_render` and `exec` methods' behavior: when `stdout_setting.render_output` is `false` or the result is empty, the exit code from the result is now returned instead of hardcoded `0`. This ensures that programs which set a non-zero exit code without producing renderable output (e.g., via `ExitCodeSetup` or `ProgramControlUnit::OverrideExitCode`) will exit with the correct code. Specific changes in `once_exec.rs`:

    - `exec()` (async) and `exec_without_render_and_print()` (sync): The `exit_code` is now read from the result before the render check, and returned as the fallback value instead of `0` when output is not printed.
    - This means `ExitCode` / `ProgramControls` overrides are now respected regardless of whether any output is rendered.

#### Optimizations:

1. **[`core:flag`]** Refactored the `special_argument!` and `special_arguments!` macros to replace index‑based `while` loops with iterator `position` and `drain`, improving both performance and readability.

2. **[`core:comp`]** Changed the completion system's node filtering to exclude all hidden nodes (names starting with `_`) instead of only the specific `__comp` node. This makes the completion script generation more general — any node prefixed with an underscore is now treated as internal/hidden and excluded from suggestions.

3. **[`macros`]** Consolidated `__dispatch_program_renderers!` and `__dispatch_program_chains!` from `macro_rules!` into the `program_final_gen` proc-macro (`mingling_macros/src/lib.rs`), removing them from `mingling_core/src/program.rs`. The `render()` and `do_chain()` match dispatch is now generated directly by the proc-macro, using a compile-time `ASYNC_ENABLED` constant (via `#[cfg(feature = "async")]`) to select the correct sync/async signature at proc-macro compilation time, replacing the previous `#[cfg]`-gated `macro_rules!` dispatch that relied on per-crate feature resolution.

4. **[`macros`]** Added global registry cleanup at the end of `program_final_gen`, clearing all `OnceLock<Mutex<BTreeSet>>` registries after consuming them. This prevents stale state accumulation across compilation sessions.

#### Features:

1. **[`core`]** Added the `unpack_chain_process!` macro for ergonomically extracting the inner value from a `ChainProcess` result.

This macro wraps `::mingling::test::unpack_chain_process_result` to downcast a `ChainProcess::Ok` result to the specified type. It panics if the result is `ChainProcess::Err` or if the downcast fails.

```rust
let result = some_chain_function(args).into();
let value: MyType = unpack_chain_process!(result, MyType);
```

2. **[`core`]** Refactored the built-in flag system in `BasicProgramSetup` into individual, reusable setup structs (`HelpFlagSetup`, `QuietFlagSetup`, `ConfirmFlagSetup`). These setups are now separate implementations of `ProgramSetup`, each with customizable flag aliases and `Default` implementations. `BasicProgramSetup` now composes them via `with_setup` instead of defining flags inline.

```rust
// Customize individual flags
program.with_setup(HelpFlagSetup::new(["-h", "--help"]));
program.with_setup(QuietFlagSetup::new(["-q", "--quiet"]));
program.with_setup(ConfirmFlagSetup::new(["-C", "--confirm"]));

// Or use defaults via BasicProgramSetup
program.with_setup(BasicProgramSetup);
```

3. **[`core`]** Added `verbose`, `quiet`, `debug`, `color`, and `progress` fields to `ProgramStdoutSetting`, and `dry_run`, `force`, `interactive`, and `assume_yes` fields to `ProgramUserContext`. These fields are annotated as conventions only, meaning the framework does not enforce any particular behavior — it is up to the application to read and act on them.

4. **[`core`]** Added `LazyRes<T>` for lazy resource initialization. Resources wrapped in `LazyRes<T>` are only initialized when first accessed via `get_ref()` or `get_mut()`, rather than immediately when added to the program. This is useful for resources that are expensive to initialize and may not always be needed.

```rust
use std::collections::BTreeMap;
use mingling::{LazyInit, LazyRes, prelude::*};

#[derive(Default, Clone)]
pub struct ResLargeData {
    pub data: BTreeMap<String, String>,
}

fn init_res_large_data() -> ResLargeData {
    // Expensive initialization here
    ResLargeData { data: BTreeMap::new() }
}

fn main() {
    let mut program = ThisProgram::new();
    program.with_resource(ResLargeData::lazy_init(init_res_large_data));
    // ...
}

// Injected as &mut LazyRes<T> instead of &T
#[renderer]
fn render_entry_show(_args: EntryShow, res: &mut LazyRes<ResLargeData>) {
    let res = res.get_ref(); // Initialization happens here
    // use res...
}
```

5. **[`core`]** Added `Program::get_args(&self)` method to expose the program's command-line arguments as a `&[String]` slice, providing public read access to the internal `args` field.

6. **[`core:comp`]** Added `COMPLETION_SUBCOMMAND` constant to `mingling_core::comp` with the value `"__comp"`, providing a single canonical reference for the completion subcommand name used internally. This replaces hardcoded string literals across the codebase.

7. **[`core:comp`]** Added `Program::is_completing()` method to check whether the program is currently running in completion mode. This provides a convenient way to conditionally skip certain logic during completion generation, where those operations may be unnecessary or undesirable.

8. **[`macros`]** Added the `pack_err!` macro for creating error structs with automatic `name` field.

The `pack_err!` macro provides a concise way to define error types that implement `Groupped` and are automatically registered for inclusion in the program enum. The `name` field is automatically set to the snake_case version of the struct name at compile time.

Two forms are supported:

```rust
// Simple form — generates a struct with only `name: String` and a `Default` impl:
pack_err!(ErrorNotFound);

// Typed form — generates a struct with `name: String` + `info: Type` and a `new(info)` constructor:
pack_err!(ErrorNotDir = PathBuf);
```

For `pack_err!(ErrorNotFound)`, the generated code is:

```rust
#[derive(::mingling::Groupped)]
pub struct ErrorNotFound {
    name: String,
}

impl Default for ErrorNotFound {
    fn default() -> Self {
        Self {
            name: "error_not_found".into(),
        }
    }
}
```

For `pack_err!(ErrorNotDir = PathBuf)`:

```rust
#[derive(::mingling::Groupped)]
pub struct ErrorNotDir {
    name: String,
    info: PathBuf,
}

impl ErrorNotDir {
    pub fn new(info: PathBuf) -> Self {
        Self {
            name: "error_not_dir".into(),
            info,
        }
    }
}
```

This macro is only available with the `extra_macros` feature.

9. **[`mingling`]** Added `Groupped` trait to the `mingling::prelude` module, so it can now be imported via `use mingling::prelude::*` without needing to separately import the trait from the `mingling` crate root.

10. **[`macros:group`]** Added the `group!` macro for registering outside-types from external crates as group members without modifying their definitions. This macro generates a `Groupped` implementation and registers the type's simple name as an enum variant.

Uses the type's last path segment as the enum variant name:

```rust,ignore
group!(std::io::Error); // registers as `Error` variant
```

An aliased syntax is also supported for descriptive variant naming:

```rust,ignore
// registers as `IoError` variant, creates `pub type IoError = std::io::Error;`
group!(IoError = std::io::Error);
```

This macro is only available with the `extra_macros` feature.

11. **[`macros`]** `#[help]` and `#[completion]` now support resource injection parameters, consistent with `#[chain]` and `#[renderer]`. Specific changes:

- `#[help]`: Removed the restriction of "must have exactly one parameter". The first parameter still serves as the entry type, while subsequent parameters are treated as resource injections. The internal implementation was changed from a nested `help_wrapper` function to an inline body (consistent with the renderer), making resource variables visible within the scope.
- `#[completion]`: Removed the restriction of "must have exactly one parameter". The first parameter `ctx: &ShellContext` is used for the `Completion::comp` trait method signature, while subsequent parameters are treated as resource injections. Within the `comp` method body, resource bindings are injected via `::mingling::this::<P>().res_or_default::<T>()` and `modify_res`.

```rust
#[help]
fn help_my_entry(prev: EntryMyEntry, res: &ResA) {
    r_println!("res: {:?}", *res);
}

#[completion(EntryMyEntry)]
fn comp_my_entry(ctx: &ShellContext, res: &mut ResA) -> Suggest {
    // res is injected from the program's global resources
    suggest! {}
}
```

For mutable resources (`&mut T`), both macros use `Program::modify_res` (with constraint `Return: Default`) instead of `#[chain]`'s dedicated `__modify_res_and_return_route` (with constraint `Return: Into<ChainProcess>`), because the return types of help/completion are `()` and `Suggest` respectively.

12. **[`macros`]** Added async mutable resource injection support for `#[chain]`. Previously, async chain functions could only use `&T` (immutable) resource injection; `&mut T` was rejected with a compile-time error. Now, async chain functions support `&mut T` resource injection by using an extract‑store pattern: each mutable resource is cloned out of the global store before the body executes (via `__extract_res_mut`), bound as `&mut` within a scoped block, and written back after the block completes (via `__store_res`). This avoids holding a mutable borrow across `.await` points while still providing a natural `&mut T` syntax.

```rust
use mingling::macros::{chain, pack, gen_program};

pack!(MyOutput = String);

#[chain]
async fn greet(prev: HelloEntry, ec: &mut ResExitCode) -> Next {
    let name = prev.first().cloned().unwrap_or_else(|| "World".to_string());
    ec.exit_code = 42;
    some_async_fn(&name).await;
    MyOutput::new(name)
}
```

13. **[`pathf`]** Added the `mingling_pathf` sub-crate and the `pathf` feature for build-time type path resolution.

The `pathf` (pathfinder) system enables automatic resolution of type module paths at build time. It scans source files, identifies Mingling macro invocations (`pack!`, `#[chain]`, `#[renderer]`, `#[help]`, `#[completion]`, `dispatcher!`, `#[dispatcher_clap]`, `group!`, `#[derive(Groupped)]`, etc.), infers their module paths from the file system layout, and generates a mapping file consumed by `gen_program!()` at compile time.

**Feature activation**: Enable the `pathf` feature on the `mingling` crate:

```toml
mingling = { version = "0.2", features = ["pathf"] }
```

**Usage** — Add a `build.rs` to your project:

```rust
// build.rs
fn main() {
    mingling::pathf::analyze_and_build_type_mapping().unwrap();
}
```

The pathfinder system consists of:

- **`mingling_pathf` sub-crate** — A standalone crate for build-time source analysis:
    - `module_pathf::analyze()` — Scans the crate's source tree and infers module paths from the directory structure
    - `pattern_analyzer::init()` — Creates a `PatternAnalyzer` registered with all supported Mingling patterns
    - `analyze_and_build_type_mapping()` / `analyze_and_build_type_mapping_for()` — Convenience functions for build scripts
    - **Pattern matchers** — Individual pattern implementations for each Mingling macro:
        - `PackPattern` — Matches `pack!`, `pack_err!`, `pack_structural!`, `pack_err_structural!` invocations
        - `GroupPattern` — Matches `group!` and `group_structural!` invocations
        - `GrouppedDerivePattern` — Matches `#[derive(Groupped)]` and `#[derive(GrouppedSerialize)]`
        - `ChainPattern` — Matches `#[chain]` functions, extracts `__internal_chain_*` names
        - `RendererPattern` — Matches `#[renderer]` functions, extracts `__internal_renderer_*` names
        - `HelpPattern` — Matches `#[help]` functions, extracts `__internal_help_*` names
        - `CompletionPattern` — Matches `#[completion(T)]` functions, extracts `__internal_completion_*` names
        - `DispatcherPattern` — Matches `dispatcher!` invocations, extracts entry type names (supports both explicit and implicit forms)
        - `DispatcherClapPattern` — Matches `#[dispatcher_clap]` structs, extracts struct names
    - `type_mapping_builder` — Assembles the mapping from all analyzed files and writes `MAPPING` and `type_using.rs` output files

- **Integration with `gen_program!()`** — When the `pathf` feature is enabled, `gen_program!()` includes the generated `type_using.rs` file via `include!()`, making all type paths available in scope for the generated dispatch code.

- **Public re-exports** — The `mingling` crate re-exports `mingling_pathf` types under `mingling::pathf::*` and error types under `mingling::error::*` (behind the `pathf` feature gate).

#### **BREAKING CHANGES** (API CHANGES):

---

**IMPORTANT**: **Breaking: Remove All Explicit Program Name Modes**

**All macros no longer accept a custom program path.** The program name is now always `crate::ThisProgram`, determined by `gen_program!()`.

The following explicit syntaxes are **removed**:

| Macro                 | Removed syntax                                                |
| --------------------- | ------------------------------------------------------------- |
| `pack!`               | `pack!(MyProgram, Type = Inner)` → only `pack!(Type = Inner)` |
| `group!`              | `group!(MyProgram, Type)` → only `group!(Type)`               |
| `#[derive(Groupped)]` | `#[group(MyProgram)]` attribute                               |
| `#[chain]`            | `#[chain(MyProgram)]` argument                                |
| `#[renderer]`         | `#[renderer(MyProgram)]` argument                             |
| `dispatcher!`         | `dispatcher!(MyProgram, "cmd", CMD => Entry)`                 |
| `#[dispatcher_clap]`  | `#[dispatcher_clap(MyProgram, "cmd", Disp)]`                  |
| `#[program_setup]`    | `#[program_setup(MyProgram)]` argument                        |
| `gen_program!`        | `gen_program!(MyProgram)` → only `gen_program!()`             |

> **Tradeoff Rationale** — Removing explicit program names is a sacrifice of flexibility in exchange for reduced development and maintenance complexity. In practice, no use case has emerged that genuinely requires a custom program name — all real-world programs can be expressed with the default `ThisProgram`. Keeping the parameter in every macro would add ongoing documentation, testing, and cognitive overhead that is not justified by current needs.

---

1. **[`core`]** **[`structural_renderer`]** Renamed the `general_renderer` feature to `structural_renderer`. All associated types, structs, and APIs have been renamed accordingly:

    - Feature flag: `general_renderer` → `structural_renderer`
    - Setup struct: `GeneralRendererSetup` → `StructuralRendererSetup`
    - Simple setup struct: `GeneralRendererSimpleSetup` → `StructuralRendererSimpleSetup`
    - Renderer type: `GeneralRenderer` → `StructuralRenderer`
    - Setting enum: `GeneralRendererSetting` → `StructuralRendererSetting`
    - Error type: `GeneralRendererSerializeError` → `StructuralRendererSerializeError`
    - Field name: `program.general_renderer_name` → `program.structural_renderer_name`
    - Trait method: `ProgramCollect::general_render()` → `ProgramCollect::structural_render()`
    - Internal module: `mingling_core::renderer::general` → `mingling::renderer::structural`
    - Internal static: `GENERAL_RENDERERS` → `STRUCTURAL_RENDERERS`
    - Feature gate attributes: `#[cfg(feature = "general_renderer")]` → `#[cfg(feature = "structural_renderer")]`
    - Sub-features: `general_renderer_empty` → `structural_renderer_empty`, `general_renderer_full` → `structural_renderer_full`
    - Runtime feature constant: `MINGLING_GENERAL_RENDERER` → `MINGLING_STRUCTURAL_RENDERER` (and similarly for `_EMPTY` and `_FULL`)
    - Derive macro feature gate: `#[cfg(feature = "general_renderer")]` on `#[derive(StructuralData)]` and `#[derive(GrouppedSerialize)]` → `#[cfg(feature = "structural_renderer")]`
    - Example project: `example-general-renderer` renamed to `example-structural-renderer`
    - Test crate: `test-general-renderer` renamed to `test-structural-renderer`

2. **[`core`]** Changed the signature of `ProgramSetup::setup` from `fn setup(&mut self, program: &mut Program<C>) -> S` to `fn setup(self, program: &mut Program<C>)`, consuming `self` instead of taking a mutable reference. Correspondingly, `Program::with_setup` now accepts `S` by value (`&mut self, setup: S`) instead of by mutable reference (`&mut self, setup: &mut S`).

3. **[`core`]** Consolidated resource naming for `ExitCode` and `REPL`:
    - Renamed `ExitCode` to `ResExitCode` and moved `ResREPL` from the `mingling` root to `mingling::res::ResREPL` (the `mingling::ResREPL` re-export is removed).
    - This aligns with the naming convention where resources are prefixed with `Res`.
    - The corresponding setup `ExitCodeSetup` and resource injection remain unchanged.

```rust
// Before
use mingling::{res::ExitCode, REPL};

// After
use mingling::{res::ResExitCode, res::ResREPL};
```

4. **[`core`]** **[`macros`]** Migrated `to_chain()` and `to_render()` methods from being generated individually per type by `#[derive(Groupped)]` and `pack!` macros, to being provided as default trait methods on the `Groupped` trait itself.

    Previously, each packed or derived type had its own inherent `to_chain()` and `to_render()` methods generated by the macros. Now, these methods are defined on the `Groupped<Group>` trait with default implementations, making them available to all types that implement the trait without redundant code generation.

    ```rust
    // Before (generated per type by macros):
    impl MyType {
        pub fn to_chain(self) -> ChainProcess<Group> {
            AnyOutput::new(self).route_chain()
        }
        pub fn to_render(self) -> ChainProcess<Group> {
            AnyOutput::new(self).route_renderer()
        }
    }

    // After (provided by Groupped trait default methods):
    // just ensure Groupped is implemented — to_chain() and to_render()
    // are automatically available
    ```

    Removed the per-type inherent method generation from both `groupped.rs` and `pack.rs` in `mingling_macros`.

5. **[`macros`]** Changed the `route!()` macro's error branch from `return e` to `return ::mingling::Groupped::to_chain(e)`, so that the error type no longer needs to be pre-converted to `ChainProcess` via `.to_chain()` or `.to_render()`. The macro now accepts any type implementing `Groupped` in the error position and automatically converts it.

```rust
// Before
let value = route!(prev.pick_or_route((), Error::default().to_chain()).unpack());

// After
let value = route!(prev.pick_or_route((), Error::default()).unpack());
```

6. **[`core`]** **[`hook`]** Refactored the hook system to use structured info types and return `ProgramControls<C>` instead of raw values.

    The hook system has been redesigned for better type safety, extensibility, and control flow management:

    - **All hook callbacks now receive structured info types** (e.g., `&HookPreDispatchInfo`, `&HookPostChainInfo<C>`) instead of raw tuples or bare values. Each hook event has a dedicated info struct with named fields, making hook signatures self-documenting and easier to evolve.

    - Hook signatures changed from `fn(...)` to `Box<dyn Fn(&InfoType) -> R>`, with `R: Into<ProgramControls<C>>`. Closures that return `()`are automatically converted to`ProgramControls::Empty`via the`From<()>` impl.

        ```rust
        // Before
        .on_begin(|| println!("Program started"))
        .on_pre_dispatch(|args| println!("Dispatching: {args:?}"))
        .on_finish(|| 0)  // returns i32 as exit code

        // After
        .on_begin::<_, ()>(|_| println!("Program started"))
        .on_pre_dispatch(|info| println!("Dispatching: {}", info.arguments.join(" ")))
        .on_finish(|_| ProgramControlUnit::OverrideExitCode(0))
        ```

    - **Added `ProgramControls<C>` and `ProgramControlUnit<C>`** — a new control flow system that replaces the previous approach where only the `finish` hook could return a value (exit code). Now any hook can issue control instructions:
        - `ProgramControlUnit::OverrideExitCode(i32)` — override the program's exit code
        - `ProgramControlUnit::RouteToChain(AnyOutput<C>)` — route to another chain processor
        - `ProgramControlUnit::RouteToRender(AnyOutput<C>)` — route directly to the renderer
        - `ProgramControlUnit::RouteToHelp(AnyOutput<C>)` — route to help display

    - **Added `handle_program_control` function** in `exec.rs` that processes `ProgramControls` returned by hooks, updating the current execution state (exit code, current `AnyOutput`) or triggering early returns (e.g., routing to render/help).

    - **`ExitCodeSetup` updated** — its `on_finish` hook now returns `ProgramControlUnit::OverrideExitCode(this.exit_code)` instead of just `this.exit_code`.

    - **`HookPostReadlineInfo` now wraps `line: &mut String`** — the `repl_post_readline` hook receives a structured info object instead of a raw `&mut String`.

    - **`HookOnReceiveResultInfo` now wraps `result: &RenderResult`** — the `repl_on_receive_result` hook receives the result through an info struct with a `.result` field instead of directly.

    - **`hook` module made public** — moved from `#[doc(hidden)]` to a documented public module (`pub mod hook`), along with all associated info types and control unit types.

    - **Added `dispatch_args_trie` default method** on `ProgramCollect` (behind `#[cfg(not(feature = "dispatch_tree"))]`) that calls `unreachable!()` by default, avoiding `#[cfg]` gymnastics in `exec.rs`.

    - **Examples and internal callers updated** throughout the codebase to use the new hook API patterns.

7. **[`core`]** **[`structural_renderer`]** Added the `pack_err_structural!`, `pack_structural!`, and `group_structural!` macros for creating types that support structured output (JSON/YAML/TOML/RON). These are like `pack_err!`, `pack!`, and `group!` respectively, but also mark the type with the `StructuralData` trait, enabling the `StructuralRenderer` to serialize them.

8. **[`core`]** **[`structural_renderer`]** Added the `StructuralData` derive macro and sealed trait, decoupling structured output from `Groupped`. Previously, under the `structural_renderer` feature, all `pack!` and `pack_err!` types automatically derived `Serialize`. Now, structured output is an opt-in property controlled by `StructuralData`:

- `pack!` / `pack_err!` / `group!` no longer derive `Serialize` even when `structural_renderer` is enabled.
- To enable structured output, use `pack_structural!` / `pack_err_structural!` / `group_structural!` or the `#[derive(StructuralData)]` marker.
- The `Groupped` trait no longer requires `Serialize` bounds, and `AnyOutput::new` no longer requires `Serialize`.
- `StructuralRenderer::render` now accepts `T: StructuralData + Send` instead of `T: Serialize + Send`, and the individual format methods (`render_to_json`, etc.) are now private.

9. **[`core`]** **[`structural_renderer`]** Added `mingling::__private::StructuralDataSealed` and `mingling::__private::StructuralData` (re-exported from `mingling_core::renderer::structural::structural_data`) to support the sealed trait pattern. The `StructuralData` trait is only implementable via the derive macro or the `_structural` macro variants.

10. **[`macros`]** Changed `ResultEmpty` from a tuple struct (wrapping `()`) to a fieldless unit struct. `ResultEmpty::new(())` is now simply `ResultEmpty`. This simplifies construction and reduces generated code.

- `pack!(ResultEmpty = ())` → `#[derive(...)] pub struct ResultEmpty;`
- `crate::ResultEmpty::new(())` → `crate::ResultEmpty`
- Updated all references in `#[chain]` code generation, `empty_result!()` macro, and `program_fallback_gen`/`program_final_gen` accordingly.

When the `structural_renderer` feature is enabled, `ResultEmpty` also derives `Serialize` and `StructuralData` for consistency with structured output support.

---

### Release 0.1.9 (2026-05-29)

#### Fixes:

1. **[`macros:dispatcher_clap`]** Fixed the issue where clap error messages (`DisplayHelp` and parse errors from `try_parse_from`) could not output ANSI
    - For error paths, use `e.render().ansi()` instead of `e.to_string()` to prevent ANSI codes from being stripped by `strip_str` in `StyledStr::Display`
    - For help info paths, use with `BasicProgramSetup`, output ANSI-colored help content through the mingling framework's `render_help` flow

#### Optimizations:

1. **[`macros`]** Removed dependency `once_cell`, replaced with `std::sync::OnceLock`

#### Features:

1. **[`macros`]** Added the `empty_result!()` macro for early return from a chain function. This macro is a shorthand for constructing an `EmptyResult` and converting it into a `ChainProcess`, signaling to the pipeline that there is no meaningful output to continue processing.

```rust
use mingling::macros::empty_result;

#[chain]
fn maybe_skip(prev: SomeEntry) -> Next {
    if should_skip() {
        return empty_result!();
    }
    // ... continue processing
    NextEntry::new(result)
}
```

Expands to: `crate::EmptyResult::new(()).to_chain()`

2. **[`picker`]** Added support for `PathBuf` and `Vec<PathBuf>`, and added `PathChecker` for filtering and validating file paths

```rust
#[chain]
fn handle_path_pick(prev: PathPick) {
    let path = prev
        // Extract the list of PathBufs
        .pick::<Vec<PathBuf>>(())
        // Filter, keep only existing files
        .after(|p| p.passed(&PathCheckRule::new().must_file().must_exist()))
        .unpack();
}
```

3. **[`macros`]** Extended the `#[renderer]` attribute to support custom return types. Previously, `#[renderer]` functions could only return `()`, and the generated helper function always returned `RenderResult`. Now:
    - **`fn foo(x: T)` / `fn foo(x: T) -> ()`** → The generated helper function returns `()`. If the internal `RenderResult` (`dummy_r`) is non-empty, it is automatically printed to stdout.
    - **`fn foo(x: T) -> U`** → The generated helper function returns `U`. The internal `RenderResult` is converted via `dummy_r.into()`, and no automatic printing occurs.

4. **[`macros`]** Resource injection is now shared between `#[chain]` and `#[renderer]`.  
   Extracted the common resource injection infrastructure (`ResourceInjection`, `extract_args_info`, `generate_immut_resource_bindings`, `wrap_body_with_mut_resources`) from `chain.rs` into a new `res_injection.rs` module. Both `#[chain]` and `#[renderer]` now reuse the same logic.

The `#[renderer]` attribute now supports resource injection parameters (just like `#[chain]`):

```rust
#[renderer]
fn render_greeting(prev: Greeting, res: &MyRes) {
    println!("{}{}", res.prefix, *prev);
}
```

5. **[`picker`]** Implement `Pickable` for `Option<T>`

Added `impl<T: Pickable<Output = T> + Default> Pickable for Option<T>`, allowing optional values to be directly parsed via `Pickable` without manually handling the `Option` wrapping logic.

6. **[`macros`]** Added `entry!` macro

The `entry!` macro provides a convenient way to construct packed entry wrapper types (created via `dispatcher!`) with test data. Two syntax forms are available:

```rust
// With explicit type — expands to MyEntry::new(vec!["a".to_string(), ...])
entry!(MyEntry, ["a", "b", "c"])

// Without type — uses bracket syntax, expands to vec!["a".to_string(), ...].into()
entry!["a", "b", "c"]
```

7. **[`macros`]** Added `dispatcher!` macro with implicit entry/dispatcher name derivation

```rust
// implicit
dispatcher!("remote.add" /*, CMDRemoteAdd => EntryRemoteAdd */);

// explicit
dispatcher!("remote.remove", CMDRemoteRemove => EntryRemoteRemove);
```

8. **[`macros`]** The `pack!` macro now supports adding doc comments and attributes (e.g., `#[doc(hidden)]`) to the inner structs:

```rust
pack!{
    /// Your comment
    StateGreet = String
}

pack! {
    #[doc(hidden)]
    ThisProgram, InternalErrorNoName = ()
}
```

9. **[`macros`]** The `dispatcher!` macro now supports adding doc comments and attributes (e.g., `#[doc(hidden)]`) to both the dispatcher and entry structs, similar to the `pack!` macro:

```rust
// Implicit
dispatcher! {
    /// Comment for dispatcher
    "todo.add", CMDTodoAdd =>
    /// Comment for entry
    EntryTodoAdd
}

// Explicit
dispatcher! {
    /// Comment for dispatcher
    ThisProgram, "todo.add", CMDTodoAdd =>
    /// Comment for entry
    EntryTodoAdd
}
```

10. **[`mingling`]** Added the `DirectoryEnvironmentSetup<C>` setup struct, which registers four common directory-based resources (`ResCurrentDir`, `ResCurrentExe`, `ResHomeDir`, `ResTempDir`) in a single call. These resources provide convenient access to the current working directory, the executable's path, the user's home directory, and the system temporary directory, respectively.

```rust
use mingling::setups::DirectoryEnvironmentSetup;

program.with_setup(DirectoryEnvironmentSetup::<ThisProgram>::default());
```

11. **[`mingling`]** Added four new resource types for directory environments:

- `ResCurrentDir` — Wraps `std::env::current_dir()` as a global resource. Provides `new() -> Result`, `Default` (panics on failure), and conversions from/to `PathBuf`, `&Path`, and `&PathBuf`.
- `ResCurrentExe` — Wraps `std::env::current_exe()` as a global resource. Provides `new() -> Result`, `Default` (panics on failure), and conversions from/to `PathBuf`, `&Path`, and `&PathBuf`.
- `ResHomeDir` — Wraps the user's home directory (`$HOME` on Unix, `%USERPROFILE%` on Windows) as a global resource. Provides `new() -> Result`, `Default` (panics on failure), and conversions from/to `PathBuf`, `&Path`, and `&PathBuf`.
- `ResTempDir` — Wraps `std::env::temp_dir()` as a global resource. Provides `new()` (infallible), `Default`, and conversions from/to `PathBuf`, `&Path`, and `&PathBuf`.

All four types implement `Deref<Target = PathBuf>`, `DerefMut`, `AsRef<Path>`, `Clone`, `Debug`, and `PartialEq`.

#### **BREAKING CHANGES** (API CHANGES):

1. **[`core`]** Panic Unwind will not be supported when the `async` feature is enabled
2. **[`core`]** `modify_res` signature changed: now returns `Return` instead of `()`
3. **[`core`]** Renamed internal method `__modify_res_and_return_any` to `__modify_res_and_return_route`
4. **[`macros`]** Renamed the macro-internal function parameter `r` (used with the `r_` prefix) to `__renderer_inner_result` to reduce context pollution

```rust
// Before
#[renderer]
fn render(prev: Previous) { // Implicitly introduces `r`
    r_println!("{}", *prev); // Modifies `r`
}

// After
#[renderer]
fn render(prev: Previous) { // Implicitly introduces `__renderer_inner_result`
    r_println!("{}", *prev); // Modifies `__renderer_inner_result`
}
```

5. **[`macros`]** Moved the `entry!`, `route!`, `#[program_setup]` macros into the `extra_macros` feature

6. **[`macros`]** The `crate::Next` generated by `gen_program!()` now requires explicit import into the project

```rust
use crate::Next;

#[chain]
fn handle_cmd(args: EntryCmd) -> Next {
    //                           ^^^^\_ requires explicit import
    // ...
}

gen_program!();
```

7. **[`macros:comp`]** Renamed `CompletionDispatcher` to `CMDCompletion`
8. **[`macros:comp`]** Marked `CompletionContext` and `CompletionSuggest` as `#[doc(hidden)]`
9. **[`macros`]** Renamed `DispatcherNotFound` to `ErrorDispatcherNotFound`
10. **[`macros`]** Renamed `RendererNotFound` to `ErrorRendererNotFound`
11. **[`macros`]** Renamed `EmptyResult` to `ResultEmpty`

---

### Release 0.1.8 (2026-05-18)

#### Fixes:

None

#### Optimizations:

1. **[`core`]** The core library no longer depends on `thiserror`

2. **[`mingling`]** Split the monolithic `general_renderer` feature into separate format-specific features:
    - `general_renderer` now only includes core serialization support without any specific format
    - `general_renderer_full` bundles all available serialization formats
    - Individual format features: `json_serde_fmt`, `yaml_serde_fmt`, `toml_serde_fmt`, `ron_serde_fmt`
    - A meta feature `all_serde_fmt` enables all format features at once

#### Features:

1. **[`macros`]** The `gen_program!()` macro now generates `pub fn this() -> &'static Program<#name>` for the generated program type, providing convenient static accessors.
2. **[`macros`]** The `#[chain]` macro now supports resource injection parameters (2nd to Nth). When you write:

```rust
#[chain]
fn process(prev: HelloEntry, age: &Age, name: &Name) -> Next {
    // age and name are automatically injected from global resources
}
```

Will expand:

```rust
fn proc(prev: HelloEntry) -> ChainProcess<ThisProgram> {
    let age: &Age = ::mingling::this::<ThisProgram>()
        .res_or_default::<Age>()
        .as_ref();
    let name: &Name = ::mingling::this::<ThisProgram>()
        .res_or_default::<Name>()
        .as_ref();
    // original function body inlined here
}
```

3. **[`macros`]** The `#[chain]` macro now supports mutable resource injection via the `&mut` syntax. When you write:

```rust
#[chain]
pub fn handle_some_entry(_prev: SomeEntry, exit: &mut ExitCode) -> Next {
    exit.exit_code = 2;
    Empty::default()
}
```

Will expand:

```rust
fn proc(_prev: OkEntry) -> ChainProcess<ThisProgram> {
    ::mingling::this::<ThisProgram>()
        .__modify_res_and_return_any(|exit: &mut ExitCode| {
            exit.exit_code = 2;
            Empty::default()
        })
}
```

This allows directly mutating global resources within chain functions without manually calling `modify_res`. Multiple `&mut` parameters are supported with proper nesting.

4. **[`mingling`]** Added the `dispatch_tree` feature. When enabled, it will automatically generate a prefix tree, improving the command lookup efficiency from O(n) to O(len)

5. **[`mingling`]** Added `mingling::feature` module for runtime feature detection. You can now check which features are enabled at compile time:

```rust
// Example: Check if a feature is enabled
if mingling::feature::MINGLING_ASYNC {
    // async feature is enabled
}
```

6. **[`core`]** Added `with_hook` functions to embed callback events into the program lifecycle
7. **[`core`]** Added `user_context.run_hook` configuration item to control whether the program runs hooks
8. **[`core`]** Added `exec_and_exit`, which will return an `i32` exit code after the program ends
9. **[`core`]** Added `ExitCodeSetup`, you can control the program's exit code by modifying the `mingling::res::ExitCode` resource

```rust
#[chain]
fn your_chain(_prev: Prev) -> Next {
    // Use `modify_res` to modify the value of `ExitCode`
    this::<ThisProgram>().modify_res(|r: &mut ExitCode| r.exit_code = 1);

    // Or use:
    mingling::res::update_exit_code::<ThisProgram>(1);
    // ...
}
```

10. **[`core`]** `RenderResult` now carries new data `exit_code`

11. **[`core`]** Added `modify` function to `ResourceMarker` for modifying a program's global resources

```rust
// Example
ExitCode::modify::<ThisProgram>(|code| {
    code.exit_code = 1;
});

// Equivalent to:
this::<ThisProgram>().modify_res::<ExitCode>(|code| {
    code.exit_code = 1;
});
```

12. **[`core`]** Added panic catch for program execution.
13. **[`core`]** Added the `stdout_setting.silence_panic` option, which is disabled by default. When enabled, `Program`'s `panic!` will not output to the console

14. **[`macros`]** `#[chain]` now supports a no-return-value mode, which will automatically return `crate::EmptyResult::new(()).to_chain()`

```rust
#[chain]
fn my_chain(prev: Prev) {
    // Do something
}

// Equivalent to
#[chain]
fn my_chain(prev: Prev) -> Next {
    // Do something
    crate::EmptyResult::new(()).to_chain()
}
```

#### **BREAKING CHANGES**:

1. **[`core`]** The signature of `exec` has been changed to `exec(self) -> i32` (previously was `exec(self)`)

2. **[`macros`]** All proc macros that accept a program/group name parameter (e.g. `pack!`, `dispatcher!`, `#[chain]`, `#[program_setup]`, `#[dispatcher_clap]`, `#[derive(Groupped)]`) now parse the name as a `syn::Path` instead of a bare `Ident`. This means:
    - You can now use paths like `crate::MyProgram` or `my_crate::MyProgram` in addition to plain `MyProgram`.
    - The default program name `ThisProgram` is no longer re-exported or required as an import — generated code references `crate::ThisProgram` directly.
    - If you previously imported `ThisProgram` from `crate` only for macro use, that import is no longer needed and can be removed.

```rust
use crate::ThisProgram; // Can be removed if not used directly
```

3. **[`core`]** **[`macros`]** Replace `NextProcess` placeholder with `Next`

```rust
// Before
#[chain]
fn your_chain(_prev: Prev) -> NextProcess {
    // ...
}

// After
#[chain]
fn your_chain(_prev: Prev) -> Next {
    // ...
}

```

---

### Release 0.1.7 (2026-05-04)

#### Fixes:

1. Fixed a build failure on **Windows** caused by `mingling_core/src/program.rs`
2. **[`picker`]** Fixed an issue where the `Pickable` trait for `Yes` and `True` types could not correctly parse explicit boolean `--value true`

#### Optimizations:

1. **[`macros`]** Optimized the memory usage of the `gen_program!()` macro: the internal generated enum now uses the smallest possible integer representation (`u8`, `u16`, `u32`, or `u128`) based on the number of packed types, instead of always using `u32`.

#### Features:

1. **[`mingling`]** Added the scaffolding tool `mling`, which can quickly deploy and test your command-line programs
2. **[`macros`]** Completed the `clap` feature: **Mingling** now supports parsing input using `clap::Parser`

```rust
#[derive(Groupped, clap::Parser)]
#[dispatcher_clap("your_cmd", YourClapCommand)]
// #[dispatcher_clap("...", ..., error = YourCommandParseError)] // Dispatch when parse failed
// #[dispatcher_clap("...", ..., help = true)] // Enable clap help
struct YourCommandEntry {
    #[arg(long, short)]
    str_param: String,

    #[arg(long, short)]
    path_param: PathBuf,
}
```

3. **[`clap`]** Added the `stdout_setting.clap_help_print_behaviour` configuration item to `Program`, used to control the behavior of Clap Help
4. **[`core`]** Added function `new_with_args` to `Program`
5. **[`core`]** Added function `dispatch_args_dynamic` to `Program`
6. **[`core`]** Impl `std::io::Write` trait for `RenderResult`
7. **[`core`]** Added Help system, which allows binding an event for `--help` to an `Entry` via the `help!` macro

```rust
#[help]
fn your_command_help(_prev: YourEntry) {
    r_println!("Your help docs");
}
```

8. **[`core`]** Added the function `build_comp_script_to` to the `mingling::build` module: supports outputting completion scripts precisely to a specified directory
9. **[`macros`]** Added the `route!` macro, which allows quick error routing within the `chain!` function. Usage is as follows:

```rust
// Before
#[chain]
fn parse(prev: PickEntry) -> mingling::ChainProcess<ThisProgram> {
    let picker = Picker::new(prev.inner);
    let pick_result = picker
        .pick_or_route((), NoNameProvided::default().to_render())
        .unpack();

    match pick_result {
        Ok(name) => {
            // use name here
        }
        Err(e) => {
            // handle error route here
            e
        }
    }
}

// After
#[chain]
fn parse(prev: PickEntry) -> mingling::ChainProcess<ThisProgram> {
    let picker = Picker::new(prev.inner);
    let name: String = route! {
        picker
            .pick_or_route((), NoNameProvided::default().to_render())
            .unpack()
    };

    // use name here
}
```

10. Added a resource system to `Program` for managing global resources [Details](docs/res/changlog_examples/feat_program_res.rs)

```rust
// Define global resource
#[derive(Debug, Default, Clone)]
struct Global {
    name: String,
    age: i32,
}

// Add global resource
program.with_resource(Global::default());

// Read the global resource
let global = this::<ThisProgram>().res_or_default::<Global>();

// Modify the global resource
this::<ThisProgram>().modify_res(|r: &mut Global| {
    r.name = name;
    r.age = age
});
```

11. **[`picker`]** For any type that can `Into<Vec<String>>`, `.pick()`, `.pick_or()`, and `.pick_or_route()` functions are now supported

```rust
// Before
let name: String = Picker::new(prev.inner).pick("--name").unpack();

// Now
let name: String = prev.pick("--name").unpack();
```

#### **BREAKING CHANGES**:

1. **[`macros`]** Removed macro `dispatcher_render!` from `mingling_macros`
2. **[`core`]** The `<..., Group>` in `Program<Collect, Group>` no longer requires `std::fmt::Display`
3. **[`core`]** Changed `Program<Collect, Group>` to `Program<Collect>` (merged the Group and Collect types)
4. **[`picker`]** When performing `unpack` or `unpack_directly` on the result of the first `pick` of `Picker`, it no longer returns a tuple

```rust
// Before
#[chain]
fn parse_sth(prev: SomeEntry) -> NextProcess {
    let str: String = Picker::<()>::new(prev.inner)
        .pick_or((), "None")
        .unpack_directly().0;
    let parsed = Something::new(ok);
    parsed
}

// Now
#[chain]
fn parse_sth(prev: SomeEntry) -> NextProcess {
    let str: String = Picker::<()>::new(prev.inner)
        .pick_or((), "None")
        .unpack_directly(); // Directly return the type instead of a tuple
    let parsed = Something::new(ok);
    parsed
}
```

5. **[`core`]** Removed `mingling::marker::NextProcess` and moved its creation process to `gen_program!()`

```rust
use mingling::marker::NextProcess; // Remove this

// NextProcess generated here
gen_program!();
```

6. **[`picker`]** Simplified `Picker` logic:
    - `Picker` no longer requires the generic parameter `<G>` by default; it only needs it when using `pick_or_route` or `after_or_route`

    - Additionally, if no `or_route` operations are used, the `unpack_directly` function is no longer available; `unpack` will directly extract the inner value

```rust
// Before
let (name, age) = Picker::<()>::new(prev.inner) // had to specify an arbitrary type even for routers Picker without routes
    .pick::<String>(())
    .pick::<i32>(())
    .unpack_directly(); // had to use `unpack_directly` to get the inner value

// After
let (name, age) = Picker::new(prev.inner) // no longer need to specify an unused route type
    .pick::<String>(())
    .pick::<i32>(())
    .unpack(); // no longer need to use `unpack_directly`

// But ...
let (name, age) = Picker::new(prev.inner)
    .pick::<String>(())
    .pick_or_route::<i32>((), NoNumberProvided::default().to_render()) // if a route type is specified
    .unpack(); // will return Result<Value, Route>
```

7. **[`macros`]** The enum generated by `gen_program!()` no longer has a default variant (`__FallBack`), and the `#[default]` attribute has been removed accordingly.

8. **[`macros`]** Removed `#[derive(Debug)]` from generated pack types to remove unnecessary trait bounds.

9. **[`macros`]** **[`core`]** **[`mingling`]** Removed the `full` feature from all crates.

---

### Yanked 0.1.6 (2026-04-24)

> [!CAUTION]
>
> This version cannot be built correctly on **Windows**, please do not use this version.

> [!warning]
>
> To align with the `mingling` version, `mingling_core` and `mingling_macros` will skip version `0.1.5` and be released directly as `0.1.6`.

---

### Release 0.1.6 (2026-04-20)

`Mingling` 0.1.6 primarily focuses on optimizing the writing experience and code completion.

#### Fixes:

1. **[`core`]** Fixed an issue where the `Powershell` completion script could not be used.

#### Features:

1. **[`core`]** Added support for completion descriptions in `Powershell`.
2. **[`core`]** Added more context-based completion functions, such as `filling_argument` and `typing_argument`. For details, see [Docs.rs](https://docs.rs/mingling/0.1.6/mingling/)

#### **BREAKING CHANGES**:

1. **[`macros`]** The `chain!` macro no longer requires explicit type conversion when routing a type to `Chain`.

```rust
// Before
#[chain]
fn proc(_prev: SomeType) -> NextProcess {
    let result = SomeResult::new(());
    result.to_chain()
}

// Now
#[chain]
fn proc(_prev: SomeType) -> NextProcess {
    let result = SomeResult::new(());
    result // No need for `to_chain()`
}
```

2. **[`macros`]** Moved type registration from the `chain!` and `renderer!` macros forward to the `pack!` and `derive Groupped` macros

3. **[`core`]** **[`macros`]** Added an `async` feature, which is disabled by default. `Mingling` no longer forces a dependency on an Async Runtime.

4. **[`picker`]** Changed the signature of `pick_or` from `(..., or: TNext)` to `(..., or: impl Into<TNext>)`

---

> [!NOTE]
>
> Versions 0.1.0 through 0.1.5 were released before this CHANGELOG file existed (which was introduced in 0.1.6). The entries above have been retroactively reconstructed from git history and may not be fully comprehensive.

---

### Release 0.1.5 (2026-04-12)

#### Fixes:

None

#### Features:

1. **[`completion`]** Added the completion system, including `ShellContext`, shell suggestion generation, and completion script build support (`build_comp_script_to`)
2. **[`completion`]** Added `YesOrNo` and `TrueOrFalse` pickable boolean types for completion
3. **[`core`]** Implemented `mingling::this` function for accessing the current program instance
4. **[`workspace`]** Added workspace configuration and example projects
5. **[`docs`]** Added architecture diagram, project branding, and README structure improvements

#### BREAKING CHANGES:

1. **[`macros`]** Renamed `DefaultProgram` to `ThisProgram` and removed `ThisProgram` marker type

---

### Release 0.1.4 (2026-04-06)

#### Fixes:

None

#### Features:

1. **[`picker`]** Added vector pickers for collecting multiple values
2. **[`picker`]** Added error routing to `Picker` with generic route type
3. **[`picker`]** Added `after` method for post-processing picked values
4. **[`macros`]** Added `Groupped` derive macro for automatic trait implementation
5. **[`macros`]** Added `general_renderer` support with serialization formats (behind feature flag)
6. **[`macros`]** Simplified attribute parsing in macros

#### BREAKING CHANGES:

None

---

### Release 0.1.3 (2026-04-01)

#### Fixes:

1. **[`core`]** Added early exit for renderer not found in execution loop
2. **[`core`]** Added default error handling methods to `ProgramCollect` trait

#### Features:

1. **[`core`]** Replaced typeid-based dispatch with enum-based dispatch for better performance
2. **[`macros`]** Renamed `chain_struct` macro to `pack`
3. **[`docs`]** Added documentation for `mingling_core` and public items in parser modules

#### BREAKING CHANGES:

1. **[`macros`]** The `chain_struct!` macro has been renamed to `pack!`

---

### Release 0.1.2 (2026-03-31)

#### Fixes:

None

#### Features:

1. **[`parser`]** Added argument parser module with `Picker` API
2. **[`parser`]** Added `Argument` type to picker builtins and exposed `Picker` publicly
3. **[`core`]** Added `From<()>` implementation for `Flag`

#### BREAKING CHANGES:

None

---

### Release 0.1.1 (2026-03-29)

#### Fixes:

None

#### Features:

1. **[`core`]** Replaced `ChainProcess` type alias with an enum for better type safety
2. **[`core`]** Added `general_renderer` and `full` features
3. **[`core`]** Removed `ProgramEnd` and `NoChainFound` hint markers
4. **[`mingling`]** Created the `mingling` umbrella crate with core re-exports and documentation

#### BREAKING CHANGES:

1. **[`core`]** `ChainProcess` changed from a type alias to an enum; conversion code may need updating

---

### Release 0.1.0 (2026-03-29)

Initial release of the Mingling framework.

#### Features:

1. **[`core`]** Basic chain processing pipeline with `#[chain]` and `#[renderer]` macros
2. **[`macros`]** `program!` for program generation, `chain_struct!` for wrapper types, `dispatcher!` for command routing
3. **[`core`]** `Program` struct with dispatcher registration and execution
4. **[`core`]** `RenderResult` for terminal output buffering
5. **[`docs`]** README and license files

---
