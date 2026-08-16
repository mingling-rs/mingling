//! Proc-macro engine of the Mingling CLI framework.
//!
//! This crate is the **macro layer** of Mingling. Each `#[attribute]` or `!`-callable
//! macro collects metadata into **compile-time global registries** (`OnceLock<Mutex<BTreeSet>>`).
//! At the end, `gen_program!` reads all registries and generates the final program struct
//! with all dispatchers, chains, renderers, and completions wired together.

#![deny(missing_docs)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]
#![allow(clippy::redundant_pub_crate)]

use proc_macro::TokenStream;
use std::collections::BTreeSet;
use std::sync::Mutex;
use std::sync::OnceLock;

mod attr;
mod derive;
mod func;
mod systems;

mod extensions;
mod utils;

// Bring all sub-modules into scope at the old paths so that existing
// references (e.g. `chain::chain_attr`, `renderer::renderer_attr`)
// continue to work without any `use`-path changes.
#[cfg(feature = "comp")]
use attr::completion;
#[cfg(feature = "clap")]
use attr::dispatcher_clap;
#[cfg(feature = "extras")]
use attr::program_setup;
use attr::{chain, help, metadata, renderer};
use derive::{enum_tag, grouped, wrap};
use func::dispatcher;
#[cfg(feature = "extras")]
use func::entry;
#[cfg(feature = "extras")]
pub(crate) use func::group as group_impl;
#[cfg(feature = "comp")]
use func::suggest;
use systems::res_injection;
pub(crate) fn default_program_path() -> proc_macro2::TokenStream {
    quote::quote! { crate::ThisProgram }
}

// Helper to get or init a OnceLock<Mutex<BTreeSet<String>>>
pub(crate) fn get_global_set(lock: &OnceLock<Mutex<BTreeSet<String>>>) -> &Mutex<BTreeSet<String>> {
    lock.get_or_init(|| Mutex::new(BTreeSet::new()))
}

pub(crate) type Registry = OnceLock<Mutex<BTreeSet<String>>>;

// Global variables
#[cfg(feature = "structural_renderer")]
pub(crate) static STRUCTURAL_RENDERERS: Registry = OnceLock::new();

/// Types explicitly marked with `#[derive(StructuralData)]` or created via
/// `group_structural!`.
#[cfg(feature = "structural_renderer")]
pub(crate) static STRUCTURED_TYPES: Registry = OnceLock::new();

#[cfg(feature = "comp")]
pub(crate) static COMPLETIONS: Registry = OnceLock::new();

pub(crate) static COMPILE_TIME_DISPATCHERS: Registry = OnceLock::new();

pub(crate) static PACKED_TYPES: Registry = OnceLock::new();
pub(crate) static CHAINS: Registry = OnceLock::new();
pub(crate) static RENDERERS: Registry = OnceLock::new();
pub(crate) static CHAINS_EXIST: Registry = OnceLock::new();
pub(crate) static RENDERERS_EXIST: Registry = OnceLock::new();
pub(crate) static HELP_REQUESTS: Registry = OnceLock::new();
pub(crate) static METADATA: Registry = OnceLock::new();

/// Checks if a variant name already exists in a registered set.
/// Returns a `compile_error` token stream if a duplicate is found.
pub(crate) fn check_duplicate_variant(
    set: &std::collections::BTreeSet<String>,
    entry_str: &str,
    variant_name: &str,
    kind: &str,
    error_span: proc_macro2::Span,
) -> Result<(), proc_macro2::TokenStream> {
    for existing in set {
        if existing == entry_str {
            // Exact same entry - re-registration from RA re-analysis, skip
            continue;
        }
        if entry_has_variant(existing, variant_name) {
            return Err(syn::Error::new(
                error_span,
                format!(
                    "duplicate {kind} registration for `{variant_name}`: a {kind} with this type already exists"
                ),
            )
            .to_compile_error());
        }
    }
    Ok(())
}

/// Checks if a stored entry string contains the given variant name.
/// Handles both "[`StructName`] => Variant," and "[`Self::Variant`] => ..." formats.
fn entry_has_variant(entry: &str, variant_name: &str) -> bool {
    let variant_match = format!("=> {variant_name}");

    // "StructName => Variant," — exact match with trailing comma
    if entry.contains(&format!("{variant_match},")) {
        return true;
    }
    // "StructName => Variant " — exact match with trailing space
    if entry.contains(&format!("{variant_match} ")) {
        return true;
    }
    // "StructName => Variant" (fallback) — must NOT be followed by identifier chars
    if let Some(idx) = entry.find(&variant_match) {
        let after = idx + variant_match.len();
        if after >= entry.len()
            || !entry[after..].starts_with(|c: char| c.is_alphanumeric() || c == '_')
        {
            return true;
        }
    }
    // "Self::Variant => ..." — match-arm existence check format
    entry.contains(&format!(":: {variant_name} =>"))
}

/// Registers an outside type (from `std` or other crates) as a type recognizable
/// by the Mingling framework, without modifying the original type definition.
///
/// This macro generates a newtype wrapper around the given type that implements
/// `Grouped`, `Into<AnyOutput>`, `Into<ChainProcess>`, and the `Routable` trait,
/// making the outside type usable in `#[chain]` and `#[renderer]` functions.
///
/// # Syntax
///
/// ```rust,ignore
/// // Simple form — creates a wrapper named after the type's last segment:
/// group!(ParseIntError);
///
/// // Aliased form — creates a wrapper with a custom name:
/// group!(ErrorIo = std::io::Error);
/// ```
///
/// # Example
///
/// See the full example in the crate documentation or run:
/// ```bash
/// cargo run --example example-outside-type -- parse 42
/// cargo run --example example-outside-type -- parse hello
/// cargo run --example example-outside-type -- error
/// ```
///
/// # Requirements
///
/// - The type must be accessible at the call site (imported or fully qualified).
/// - The alias name (if provided) must not conflict with existing types.
#[cfg(feature = "extras")]
#[proc_macro]
pub fn group(input: TokenStream) -> TokenStream {
    group_impl::group_macro(input)
}

/// Like `group!` but also marks the type as supporting structured output
/// (JSON / YAML / TOML / RON) via `StructuralData`.
///
/// # Syntax
///
/// ```rust,ignore
/// group_structural!(std::io::Error);
/// group_structural!(IoError = std::io::Error);
/// ```
///
/// Requires the `structural_renderer` and `extras` features.
#[cfg(all(feature = "structural_renderer", feature = "extras"))]
#[proc_macro]
pub fn group_structural(input: TokenStream) -> TokenStream {
    func::group_structural::group_structural(input)
}

/// Early-returns the error from a `Result`, converting the `Ok` branch to the
/// next chain process value.
///
/// This macro is equivalent to:
/// ```rust,ignore
/// match expr {
///     Ok(r) => r,
///     Err(e) => return ::mingling::Routable::to_chain(e),
/// }
/// ```
///
/// It is useful inside chain functions where you have a `Result<SuccessType, ErrorType>`
/// where both types implement `Routable` and you want to propagate the error case
/// as an early return via `Routable::to_chain()`.
///
/// The key difference from a simple `?` operator is that `route!` converts **both**
/// the success and error types into the chain process — the `Ok` value is unwrapped
/// directly, while the `Err` value is converted via `Routable::to_chain()` and
/// returned early.
///
/// ## Interaction with `#[routeify]`
///
/// The [`#[routeify]`](attr.routeify.html) attribute macro automatically replaces
/// every `expr?` inside a function with `route!(expr)`. This means you can use the
/// familiar `?` syntax in chain functions instead of writing `route!(...)`
/// explicitly:
///
/// ```rust,ignore
/// use mingling::macros::chain;
///
/// #[chain(routeify)]
/// fn process(prev: SomeEntry) -> Next {
///     // `?` here expands to `route!(...)` → this macro → the match block
///     let value = some_fallible_call()?;
///     value.to_chain()
/// }
/// ```
///
/// Because `#[routeify]` maps the span of `?` to this macro, hovering over `?` in
/// a `#[routeify]` function will display this documentation — explaining what
/// the `?` actually expands to.
///
/// # Example
///
/// ```rust,ignore
/// use mingling::macros::{chain, route};
///
/// #[chain]
/// fn process(prev: SomeEntry) -> Next {
///     let value = route!(current_dir().map_err(|e| ErrorEntry::new(e.to_string_lossy().to_string())));
///     // value is the PathBuf from current_dir()
///     value.to_chain()
/// }
/// ```
#[cfg(feature = "extras")]
#[proc_macro]
pub fn route(input: TokenStream) -> TokenStream {
    func::route::route(input)
}

/// Routes errors to the rendering pipeline instead of the chain pipeline.
///
/// This macro is similar to [`route!`] but instead of routing errors through
/// `Routable::to_chain()` (which returns `ChainProcess`), it routes them
/// directly to the renderer via `crate::ThisProgram::render(AnyOutput::new(e))`
/// (which returns `RenderResult`).
///
/// This is useful in `#[renderer]` and `#[help]` functions where the return
/// type is `RenderResult` rather than `ChainProcess`.
///
/// # Syntax
///
/// ```rust,ignore
/// render_route!(expr)
/// ```
///
/// Where `expr` is an expression of type `Result<T, E>`.
///
/// # Interaction with `#[routeify]`
///
/// When `#[routeify]` is used on a `#[renderer]` or `#[help]` function (e.g.
/// `#[renderer(routeify)]` or `#[help(routeify)]`), every `expr?` is automatically
/// replaced with `render_route!(expr)` instead of `route!(expr)`.
///
/// # Example
///
/// ```rust,ignore
/// use mingling::macros::{renderer, render_route};
/// use std::io::Write;
///
/// #[renderer]
/// fn render_something(prev: SomeType) -> RenderResult {
///     let data = render_route!(fetch_data().map_err(|e| ErrorEntry::new(e.to_string())))?;
///     // ... render data
///     Ok(RenderResult::new())
/// }
/// ```
#[cfg(feature = "extras")]
#[proc_macro]
pub fn render_route(input: TokenStream) -> TokenStream {
    func::render_route::render_route(input)
}

/// Creates an empty result value wrapped in `ChainProcess` for early return
/// from a chain function.
///
/// This macro is a shorthand for constructing a [`ResultEmpty`] and converting
/// it into a [`ChainProcess`], which signals to the pipeline that there is
/// no meaningful output to continue processing and the chain should terminate.
///
/// This is useful in `#[chain]` functions where a condition determines that
/// no further processing is needed (e.g., validation failures or early exits).
///
/// # Syntax
///
/// ```rust,ignore
/// empty_result!()
/// ```
///
/// # Example
///
/// ```rust,ignore
/// use mingling::macros::{chain, empty_result};
///
/// #[chain]
/// fn maybe_skip(prev: SomeEntry) -> Next {
///     if should_skip() {
///         return empty_result!();  // Terminate chain gracefully
///     }
///     // ... continue processing
///     NextEntry::new(result).to_chain()
/// }
/// ```
///
/// # Generated code
///
/// The macro expands to:
/// ```rust,ignore
/// crate::ResultEmpty::new(()).to_chain()
/// ```
///
/// This works because [`ResultEmpty`] is automatically generated by `gen_program!`
/// and implements the necessary trait conversions into [`ChainProcess`].
///
/// # See also
///
/// - [`ResultEmpty`] — The type that represents an empty result.
/// - `route!` — For early-return from `Result` expressions.
///
/// [`ResultEmpty`]: https://docs.rs/mingling/latest/mingling/type.ResultEmpty.html
/// [`ChainProcess`]: https://docs.rs/mingling/latest/mingling/enum.ChainProcess.html
#[cfg(feature = "extras")]
#[proc_macro]
pub fn empty_result(input: TokenStream) -> TokenStream {
    func::empty_result::empty_result(input)
}

/// Creates a `Dispatcher` implementation for a subcommand.
///
/// This is the primary way to define command-line subcommands in Mingling.
/// It generates a dispatcher struct that, when matched against user input,
/// converts the arguments into a [`ChainProcess`] via the specified entry type.
///
/// The generated dispatcher implements [`Dispatcher<Program>`], which the
/// framework uses to route user input to the correct chain processor.
///
/// # Syntax
///
/// ## Full syntax (recommended)
///
/// ```rust,ignore
/// // Default program name (uses `ThisProgram`):
/// dispatcher!("command.path", CommandStruct => EntryStruct);
///
/// // Explicit program name:
/// dispatcher!(MyProgram, "command.path", CommandStruct => EntryStruct);
/// ```
///
/// ## Abbreviated syntax (requires `extras` feature)
///
/// When the `extras` feature is enabled, the `CommandStruct => EntryStruct`
/// portion can be omitted. Struct names are auto-derived from the command path
/// using `PascalCase` conversion:
///
/// ```rust,ignore
/// // "remote.add" → CMDRemoteAdd ⇒ EntryRemoteAdd
/// dispatcher!("remote.add");
/// ```
///
/// ⚠️ **Note**: The abbreviated form generates structs with internal names.
/// If you need to reference the dispatcher (e.g., `program.with_dispatcher(Dispatcher)`),
/// use the full syntax instead.
///
/// # Examples
///
/// ```rust,ignore
/// use mingling::macros::dispatcher;
///
/// // Single-level command: "hello"
/// dispatcher!("hello", HelloCommand => HelloEntry);
///
/// // Nested command: "remote.control" creates a two-level path
/// dispatcher!("remote.control", RemoteControlCommand => RemoteControlEntry);
///
/// // With explicit program:
/// dispatcher!(MyApp, "status", StatusCommand => StatusEntry);
///
/// // Abbreviated form (extras required):
/// // dispatcher!("remote.add");  // → CMDRemoteAdd, EntryRemoteAdd
/// ```
///
/// # How it works
///
/// The macro generates:
///
/// 1. **Entry struct** — A newtype wrapper around `Vec<String>` (the raw args).
///    Registered in the program enum via `register_type!`.
/// 2. **Dispatcher struct** — A hidden zero-sized struct implementing [`Dispatcher<Program>`]:
///    - `begin(args)` wraps `args` into the entry type and routes to chain.
/// 3. **Registration** — Calls `register_dispatcher!` to collect the command
///    at compile time (the `dispatch_tree` feature only selects the matching
///    strategy generated later by `gen_program!`).
///
/// With the `comp` feature, the entry type also implements `CompletionEntry`
/// for providing shell completion suggestions.
///
/// # See also
///
/// - `dispatcher_clap!` — For clap-powered argument parsing.
/// - [`#[chain]`](attr.chain.html) — For processing the dispatched entry.
///
/// [`ChainProcess`]: https://docs.rs/mingling/latest/mingling/enum.ChainProcess.html
/// [`Dispatcher<Program>`]: https://docs.rs/mingling/latest/mingling/trait.Dispatcher.html
#[proc_macro]
pub fn dispatcher(input: TokenStream) -> TokenStream {
    dispatcher::dispatcher(input)
}

/// Declares a chain processing step that transforms one type into another
/// within a Mingling pipeline.
///
/// The `#[chain]` attribute converts an ordinary function (or async function
/// with the `async` feature) into a chain step by:
/// 1. Generating a hidden struct implementing the `Chain` trait.
/// 2. Registering the chain mapping in the global chain registry.
/// 3. Keeping the original function for direct calls.
///
/// # Syntax
///
/// ```rust,ignore
/// #[chain]
/// fn my_step(prev: InputType) -> Next {
///     // transform `prev`...
///     OutputType::new(result)
/// }
/// ```
///
/// # Resource Injection
///
/// The `#[chain]` macro supports automatic injection of global resources
/// via the 2nd to Nth parameters. You can read resources immutably with
/// `&T` or mutate them with `&mut T`.
///
/// ## Immutable Resource (`&T`)
///
/// When you write `&SomeResource` as a parameter, the macro automatically
/// resolves it from the global resource store:
///
/// ```rust,ignore
/// #[chain]
/// fn process(prev: HelloEntry, age: &Age, name: &Name) -> Next {
///     // `age` and `name` are automatically injected
///     println!("Age: {}, Name: {}", age, name);
///     NextStep::default()
/// }
/// ```
///
/// This expands to:
///
/// ```rust,ignore
/// let __age_binding = ::mingling::this::<ThisProgram>().res_or_default::<Age>();
/// let age: &Age = __age_binding.as_ref();
/// let __name_binding = ::mingling::this::<ThisProgram>().res_or_default::<Name>();
/// let name: &Name = __name_binding.as_ref();
/// ```
///
/// ## Mutable Resource (`&mut T`)
///
/// When you write `&mut SomeResource` as a parameter, the macro wraps the
/// function body in nested `__modify_res_and_return_any` calls:
///
/// ```rust,ignore
/// #[chain]
/// fn process(prev: HelloEntry, count: &mut InvocationCount, name: &Name) -> Next {
///     count.0 += 1;
///     println!("Invocation #{} for {}", count.0, name);
///     NextStep::default()
/// }
/// ```
///
/// This expands to:
///
/// ```rust,ignore
/// let __name_binding = ::mingling::this::<ThisProgram>().res_or_default::<Name>();
/// let name: &Name = __name_binding.as_ref();
///
/// ::mingling::this::<ThisProgram>().__modify_res_and_return_any(|count: &mut InvocationCount| {
///     count.0 += 1;
///     println!("Invocation #{} for {}", count.0, name);
///     NextStep::default()
/// }).into()
/// ```
///
/// Multiple `&mut` parameters are supported with proper nesting.
///
/// ## Restrictions
///
/// - The first parameter (previous type) must be taken **by move**, not by reference.
/// - Resource injection parameters **must** be references (`&T` or `&mut T`),
///   owned values are not allowed.
///
/// # Sync Example
///
/// ```rust,ignore
/// use mingling::macros::{chain, gen_program};
/// use mingling::{Grouped, Wrap};
///
/// #[derive(Grouped, Wrap)]
/// pub struct MyOutput(String);
///
/// #[chain]
/// fn greet(prev: HelloEntry) -> Next {
///     let name = prev.first().cloned().unwrap_or_else(|| "World".to_string());
///     MyOutput(name)
/// }
/// ```
///
/// # Sync Example with Resource Injection
///
/// ```rust,ignore
/// use mingling::macros::{chain, gen_program};
/// use mingling::{Grouped, Wrap};
///
/// #[derive(Default, Clone)]
/// struct UserName(String);
///
/// #[derive(Grouped, Wrap)]
/// pub struct Greeting(String);
/// #[derive(Grouped, Wrap)]
/// pub struct DisplayCount(());
///
/// #[chain]
/// fn greet(prev: HelloEntry, user_name: &UserName, count: &mut u64) -> Next {
///     *count += 1;
///     Greeting(format!("Hello, {}!", user_name.0))
/// }
/// ```
///
/// # Async Example (with `async` feature)
///
/// ```rust,ignore
/// use mingling::macros::{chain, gen_program};
/// use mingling::{Grouped, Wrap};
///
/// #[derive(Grouped, Wrap)]
/// pub struct MyOutput(String);
///
/// #[chain]
/// async fn greet(prev: HelloEntry) -> Next {
///     let name = prev.first().cloned().unwrap_or_else(|| "World".to_string());
///     some_async_fn(&name).await;
///     MyOutput(name)
/// }
/// ```
///
/// # Async Example with Immutable Resource Injection
///
/// ```rust,ignore
/// use mingling::macros::{chain, gen_program};
/// use mingling::{Grouped, Wrap};
///
/// #[derive(Grouped, Wrap)]
/// pub struct MyOutput(String);
///
/// #[chain]
/// async fn greet(prev: HelloEntry, prefix: &Prefix) -> Next {
///     let name = prev.first().cloned().unwrap_or_else(|| "World".to_string());
///     some_async_fn(&name).await;
///     MyOutput(format!("{}{}", prefix.0, name))
/// }
/// ```
///
/// # Async Example with Mutable Resource Injection
///
/// ```rust,ignore
/// use mingling::macros::{chain, gen_program};
/// use mingling::{Grouped, Wrap};
///
/// #[derive(Grouped, Wrap)]
/// pub struct MyOutput(String);
///
/// #[chain]
/// async fn greet(prev: HelloEntry, ec: &mut ResExitCode) -> Next {
///     let name = prev.first().cloned().unwrap_or_else(|| "World".to_string());
///     ec.exit_code = 42;
///     some_async_fn(&name).await;
///     MyOutput(name)
/// }
/// ```
///
/// # Requirements
///
/// - The function must have at least **one** parameter (the previous type in the chain).
/// - The first parameter must be taken **by move**.
/// - The function may return `Next`, `ChainProcess<ProgramName>`, `()`, or omit the return type.
/// - The original function signature is preserved unchanged.
/// - With the `async` feature, async functions are supported; without it, async functions are rejected.
#[proc_macro_attribute]
pub fn chain(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Extension point: if attr contains extension identifiers like `routeify`,
    // re-dispatch as #[ext1] #[ext2] #[chain] fn ...
    if let Some(redispatch) = extensions::try_redispatch_simple(attr.clone(), &item, "chain") {
        return redispatch;
    }
    chain::chain_attr(attr, item)
}

/// Declares a renderer step that renders the output of a chain to the terminal.
///
/// The `#[renderer]` attribute converts a function into a renderer by:
/// 1. Generating a hidden struct implementing the `Renderer` trait.
/// 2. Registering the renderer mapping in the global renderer registry.
/// 3. Keeping the original function for direct calls.
///
/// # Syntax
///
/// ```rust,ignore
/// #[renderer]
/// fn render_my_type(prev: MyType) -> RenderResult {
///     let mut result = RenderResult::new();
///     writeln!(result, "Output: {:?}", *prev);
///     result
/// }
/// ```
///
/// # Example
///
/// ```rust,ignore
/// use mingling::macros::{renderer, gen_program};
/// use mingling::{Grouped, Wrap};
/// use std::io::Write;
///
/// #[derive(Grouped, Wrap)]
/// pub struct Greeting(String);
///
/// #[renderer]
/// fn render_greeting(prev: Greeting) -> RenderResult {
///     let mut result = RenderResult::new();
///     writeln!(result, "Hello, {}!", *prev);
///     result
/// }
/// ```
///
/// # Requirements
///
/// - The function must have exactly **one** parameter (the type to render).
/// - The function must return `()` (unit).
/// - The function **cannot** be async.
///
/// # Fallback Renderers
///
/// The macros `gen_program!` automatically generates two fallback types that
/// you can provide renderers for:
/// - `ErrorRendererNotFound` — triggered when no matching renderer is found
/// - `EntryFallback` — triggered when no matching dispatcher is found
///
/// ```rust,ignore
/// #[renderer]
/// fn fallback_dispatcher_not_found(prev: EntryFallback) -> RenderResult {
///     let mut result = RenderResult::new();
///     writeln!(result, "Unknown command: {}", prev.join(", "));
///     result
/// }
///
/// #[renderer]
/// fn fallback_renderer_not_found(prev: ErrorRendererNotFound) -> RenderResult {
///     let mut result = RenderResult::new();
///     writeln!(result, "No renderer for `{}`", *prev);
///     result
/// }
/// ```
#[proc_macro_attribute]
pub fn renderer(attr: TokenStream, item: TokenStream) -> TokenStream {
    if let Some(redispatch) = extensions::try_redispatch_simple(attr, &item, "renderer") {
        return redispatch;
    }
    renderer::renderer_attr(item)
}

/// Declares a completion suggestion provider for a command entry type.
///
/// **This macro is only available with the `comp` feature.**
///
/// The `#[completion]` attribute converts a function into a completion provider by:
/// 1. Generating a hidden struct implementing the `Completion` trait.
/// 2. Registering the completion mapping for the specified entry type.
/// 3. Keeping the original function for direct calls.
///
/// # Syntax
///
/// ```rust,ignore
/// #[completion(EntryType)]
/// fn complete_my_entry(ctx: &ShellContext) -> Suggest {
///     // Return suggestions based on current input state...
/// }
/// ```
///
/// # Example
///
/// ```rust,ignore
/// use mingling::macros::{completion, suggest, suggest_enum};
/// use mingling::{ShellContext, Suggest};
///
/// #[completion(MyEntry)]
/// fn complete_my_command(ctx: &ShellContext) -> Suggest {
///     if ctx.filling_argument_first("--name") {
///         return suggest!();
///     }
///     if ctx.filling_argument_first("--type") {
///         return suggest_enum!(MyEnum);
///     }
///     if ctx.typing_argument() {
///         return suggest! {
///             "--name": "Provide a name",
///             "--type": "Select a type"
///         }.strip_typed_argument(ctx);
///     }
///     suggest!()
/// }
/// ```
///
/// # Requirements
///
/// - The `comp` feature must be enabled.
/// - The function must have exactly one parameter of type `&ShellContext`.
/// - The function must return `Suggest`.
/// - The function cannot be async.
#[cfg(feature = "comp")]
#[proc_macro_attribute]
pub fn completion(attr: TokenStream, item: TokenStream) -> TokenStream {
    if let Some(redispatch) = extensions::try_redispatch_completion(attr.clone(), &item) {
        return redispatch;
    }
    completion::completion_attr(attr, item)
}

/// Declares a program setup function that initializes the program instance
/// before execution.
///
/// The `#[program_setup]` attribute converts a function into a setup step by:
/// 1. Generating a struct implementing the `ProgramSetup` trait.
/// 2. The setup function receives a mutable reference to `&mut Program<G>`.
///
/// # Syntax
///
/// ```rust,ignore
/// #[program_setup]
/// fn setup_my_program(program: &mut Program<ThisProgram>) {
///     program.stdout_setting.render_output = false;
/// }
/// ```
///
/// # Example
///
/// ```rust,ignore
/// use mingling::macros::program_setup;
/// use mingling::Program;
///
/// #[program_setup]
/// fn configure(program: &mut Program<ThisProgram>) {
///     program.with_setup(StructuralRendererSetup);
///     program.user_context.some_flag = true;
/// }
/// ```
///
/// # Requirements
///
/// - The function must have exactly one parameter of type `&mut Program<G>`.
/// - The function must return `()`.
/// - The function cannot be async.
#[cfg(feature = "extras")]
#[proc_macro_attribute]
pub fn program_setup(attr: TokenStream, item: TokenStream) -> TokenStream {
    program_setup::setup_attr(&attr, item)
}

/// Declares a command from a plain function.
///
/// **This macro is only available with the `extras` feature.**
///
/// The `#[command]` attribute converts a function taking `Vec<String>` into a
/// Mingling command by:
/// 1. Calling `dispatcher!("command_name")` to register the dispatcher entry.
/// 2. Generating a `#[chain]` wrapper that bridges the entry type (`Entry{Pascal}`)
///    to the original function.
/// 3. Preserving the original function unchanged (including attributes, extensions,
///    visibility, and asyncness).
///
/// # Syntax
///
/// ## Simple form (auto-derives names from function name)
///
/// ```rust,ignore
/// #[command]
/// fn greet(args: Vec<String>) -> Next {
///     // ...
/// }
/// ```
///
/// This deduces:
/// - Command path: `"greet"` (via `dot_case` of function name)
/// - Dispatcher struct: `CMDGreet`
/// - Entry struct: `EntryGreet`
/// - Dispatches via `dispatcher!("greet")`
///
/// ## Explicit attributes
///
/// ```rust,ignore
/// #[command(node = "hello.world")]
/// fn greet(args: Vec<String>) -> Next {
///     // ...
/// }
/// // → dispatcher!("hello.world", CMDGreet => EntryGreet)
/// ```
///
/// ```rust,ignore
/// #[command(name = MyDispatcher, entry = MyEntry)]
/// fn greet(args: Vec<String>) -> Next {
///     // ...
/// }
/// // → dispatcher!("greet", MyDispatcher => MyEntry)
/// ```
///
/// ## Extension attributes
///
/// Extra bare paths (e.g. `buffer`, `routeify`, `::mingling::macros::routeify`)
/// are emitted as `#[ext]` attributes **on the original function**, not on the
/// chain wrapper. The chain wrapper always uses bare `#[::mingling::macros::chain]`.
///
/// ```rust,ignore
/// #[command(buffer)]
/// fn greet(args: Vec<String>) {
///     r_println!("Hello!");
/// }
/// ```
///
/// # Resource injection
///
/// Parameters after the first are treated as resource injections and passed
/// through to the generated `#[chain]` wrapper unchanged (as reference params):
///
/// ```rust,ignore
/// #[command]
/// fn greet(args: Vec<String>, ec: &mut ResExitCode) -> Next {
///     ec.exit_code = 0;
///     // ...
/// }
/// ```
///
/// The generated chain wrapper calls the original function with `entry.into()`
/// for the first argument and passes all subsequent arguments directly.
///
/// # Requirements
///
/// - The function must have at least one parameter (the `Vec<String>` entry argument).
/// - The function must not have a `self` parameter.
/// - Visibility (`pub` etc.) and `async` are preserved on the original function.
#[cfg(feature = "extras")]
#[proc_macro_attribute]
pub fn command(attr: TokenStream, item: TokenStream) -> TokenStream {
    attr::command::command_attr(attr, item)
}

/// Declares a `Dispatcher` that uses `clap::Parser` for argument parsing.
///
/// **This macro is only available with the `clap` feature.**
///
/// The `#[dispatcher_clap]` attribute:
/// 1. Keeps the original struct definition (typically with `#[derive(clap::Parser)]`).
/// 2. Generates a dispatcher struct that parses arguments using clap and routes
///    to the chain pipeline.
/// 3. Optionally generates a `#[help]` block for displaying clap-generated help.
///
/// # Syntax
///
/// ```rust,ignore
/// // Default program (ThisProgram):
/// #[derive(clap::Parser)]
/// #[dispatcher_clap("command.name", DispatcherStruct)]
/// struct MyEntry { /* ... */ }
///
/// // With explicit error type and help:
/// #[derive(clap::Parser)]
/// #[dispatcher_clap("cmd", Disp, error = ParseError, help = true)]
/// struct CmdEntry { /* ... */ }
/// ```
///
/// # Example
///
/// ```rust,ignore
/// use clap::Parser;
/// use mingling::macros::dispatcher_clap;
///
/// #[derive(Parser)]
/// #[dispatcher_clap("greet", GreetDispatcher, error = GreetParseError, help = true)]
/// struct GreetArgs {
///     #[arg(short, long)]
///     name: String,
/// }
/// ```
///
/// # Options
///
/// - `error = ErrorType` — Specifies an error wrapper type for clap parse failures.
///   The error message is captured and routed to the renderer.
/// - `help = true` — Generates a `#[help]` block that displays clap's help output
///   when `--help` is passed.
#[cfg(feature = "clap")]
#[proc_macro_attribute]
pub fn dispatcher_clap(attr: TokenStream, item: TokenStream) -> TokenStream {
    dispatcher_clap::dispatcher_clap_attr(attr, item)
}

/// Creates a packed entry value from a list of string literals.
///
/// This is a convenience macro for constructing entry wrapper types (created
/// via `dispatcher!`) with test data, typically used in unit tests
/// or quick prototypes.
///
/// # Syntax
///
/// Two forms:
///
/// ```rust,ignore
/// // Named form — wraps into a specific entry type:
/// entry!(MyEntry, ["a", "b", "c"])
/// // Expands to: MyEntry(vec!["a".to_string(), "b".to_string(), "c".to_string()])
///
/// // Bracket form — returns Vec<String>.into() for type inference:
/// entry!["a", "b", "c"]
/// // Expands to: vec!["a".to_string(), ...].into()
/// ```
///
/// # Example
///
/// ```rust,ignore
/// use mingling::macros::entry;
///
/// // Named form (with a specific entry type):
/// let args = entry!(MyEntry, ["--name", "Alice", "--count", "5"]);
///
/// // Bracket form (type inference):
/// let args: Vec<String> = entry!["hello", "world"];
/// ```
///
/// # See also
///
/// - `dispatcher!` — Which implicitly creates entry types.
#[cfg(feature = "extras")]
#[proc_macro]
pub fn entry(input: TokenStream) -> TokenStream {
    entry::entry(input)
}

/// Registers a help request mapping between an entry type and a help struct.
///
/// This macro is used internally by the `#[help]`(macro.help.html) attribute
/// and is also available for manual registration if needed.
///
/// # Syntax
///
/// ```rust,ignore
/// register_help!(EntryType, HelpStruct);
/// ```
///
/// This adds an entry to the global `HELP_REQUESTS` registry, mapping the
/// enum variant for `EntryType` to the help rendering logic in `HelpStruct`.
#[proc_macro]
pub fn register_help(input: TokenStream) -> TokenStream {
    func::register_help::register_help(input)
}

/// Registers metadata mapping between an enum variant and a metadata type.
///
/// This macro is used internally by the `#[metadata]` attribute and is also
/// available for manual registration if needed.
///
/// # Syntax
///
/// ```rust,ignore
/// register_metadata!(EntryVariant, MetadataType);
/// ```
///
/// This adds an entry to the global `METADATA` registry, mapping the enum
/// variant for `EntryVariant` to the metadata provider trait
/// `::mingling::Metadata<MetadataType>`. The entry is consumed by
/// `gen_program!` to generate the `get_metadata` method of `ProgramCollect`.
#[proc_macro]
pub fn register_metadata(input: TokenStream) -> TokenStream {
    func::register_metadata::register_metadata_impl(input)
}

/// Registers a dispatcher at compile time.
///
/// This macro is called internally by `dispatcher!` and `dispatcher_clap!`.
/// Each call stores the node name into the global `COMPILE_TIME_DISPATCHERS`
/// registry and generates a static variable for the dispatcher instance. This
/// data is later consumed by `gen_program!` to generate command matching: a
/// character-level **trie** when the `dispatch_tree` feature is enabled, or a
/// linear longest-prefix list otherwise.
///
/// The trie dispatch works by grouping commands by their character prefix,
/// enabling O(n) lookup (where n is input length) instead of linear iteration
/// over all registered commands.
///
/// # Syntax
///
/// ```rust,ignore
/// register_dispatcher!("node.name", DispatcherType, EntryName);
/// ```
///
/// This macro should not be called directly by user code.
///
/// # See also
///
/// - `dispatcher!` — The primary way to declare dispatchers (calls this internally).
/// - `dispatch_tree_gen` / `dispatch_list_gen` modules — The matching-strategy generators.
#[proc_macro]
pub fn register_dispatcher(input: TokenStream) -> TokenStream {
    func::register_dispatcher::register_dispatcher(input)
}

/// Declares a help rendering function for an entry type.
///
/// The `#[help]` attribute converts a function into a help provider. Help
/// functions are invoked when the user passes `--help` / `-h` for a command
/// and `BasicProgramSetup` is registered on the program.
///
/// When `program.user_context.help` is `true`, the command will **skip** the
/// normal `#[chain]` and `#[renderer`] pipeline and instead route directly
/// to the registered `#[help]` function for that entry type.
///
/// The macro works by:
/// 1. Generating a hidden struct implementing the `HelpRequest` trait.
/// 2. Registering the help mapping in the global `HELP_REQUESTS` registry.
/// 3. Keeping the original function for direct calls.
///
/// Inside a `#[help]` function, you must manually create a `RenderResult`
/// and return it. Use `writeln!` on the result to
/// write help text.
///
/// # Syntax
///
/// ```rust,ignore
/// #[help]
/// fn help_my_entry(prev: MyEntry) -> RenderResult {
///     let mut result = RenderResult::new();
///     writeln!(result, "Usage: myapp myentry [options]");
///     writeln!(result, "  Does something useful.");
///     result
/// }
/// ```
///
/// # Example
///
/// ```rust,ignore
/// use mingling::macros::{help, gen_program};
/// use mingling::{prelude::*, setup::BasicProgramSetup, RenderResult};
/// use std::io::Write;
///
/// #[derive(Grouped, Wrap)]
/// pub struct MyEntry(Vec<String>);
///
/// #[help]
/// fn help_my_entry(prev: MyEntry) -> RenderResult {
///     let mut result = RenderResult::new();
///     writeln!(result, "Usage: myapp greet [name]");
///     writeln!(result, "Greets the user.");
///     result
/// }
///
/// fn main() {
///     let mut program = ThisProgram::new();
///     program.with_setup(BasicProgramSetup);  // Required for --help
///     program.with_dispatcher(CMDMyEntry);
///     program.exec_and_exit();
/// }
/// ```
///
/// # Requirements
///
/// - The function must have exactly one parameter (the entry type to provide help for).
/// - The parameter type must be a single-segment type path (e.g., `MyEntry`, not `other::MyEntry`).
/// - The function may return `RenderResult`, `()`, or any type that implements `Into<RenderResult>`.
/// - The function cannot be async.
///
/// # See also
///
/// - [`BasicProgramSetup`] — The setup that enables `--help` and `-h` flag processing.
/// - `RenderResult` — The return type for help functions.
/// - [`#[chain]`](attr.chain.html) — For processing the dispatched entry after help.
///
/// [`BasicProgramSetup`]: https://docs.rs/mingling/latest/mingling/setup/struct.BasicProgramSetup.html
#[proc_macro_attribute]
pub fn help(attr: TokenStream, item: TokenStream) -> TokenStream {
    if let Some(redispatch) = extensions::try_redispatch_simple(attr, &item, "help") {
        return redispatch;
    }
    help::help_attr(item)
}

/// Declares compile-time metadata for an entry variant.
///
/// The `#[metadata]` attribute attaches an arbitrary, compile-time-typed value
/// to an entry. The annotated function becomes the provider for the metadata:
/// its return type is the metadata type, and the attribute argument names the
/// entry enum variant the metadata belongs to.
///
/// The macro works by:
/// 1. Generating `impl ::mingling::Metadata<ReturnType> for EntryVariant` whose
///    `init_metadata()` calls the annotated function.
/// 2. Registering the entry via `register_metadata!` in the global `METADATA`
///    registry so that `gen_program!` emits the `get_metadata` method.
/// 3. Keeping the original function unchanged for direct calls.
///
/// # Syntax
///
/// ```rust,ignore
/// #[metadata(EntryGreet)]
/// fn greet_desc() -> Description {
///     Description { desc: "ok".into() }
/// }
/// ```
///
/// The metadata is later retrieved with `ProgramCollect::get_metadata`:
///
/// ```rust,ignore
/// let desc = ThisProgram::get_metadata::<Description>(ThisProgram::EntryGreet);
/// ```
///
/// # Requirements
///
/// - The attribute argument must be the enum variant to attach metadata to.
/// - The function must take no parameters and return a concrete type.
/// - The function cannot be async.
#[proc_macro_attribute]
pub fn metadata(attr: TokenStream, item: TokenStream) -> TokenStream {
    metadata::metadata_attr(attr, item)
}

/// Marker attribute for the Mingling lint system.
///
/// The content of this attribute is ignored by rustc and reserved for
/// the mlint tool to interpret. All it does is pass the item through
/// unchanged.
///
/// # Examples
///
/// ```rust,ignore
/// #[mlint(allow(MLINT_SOME_LINT))]
/// #[mlint(warn(MLINT_SOME_LINT))]
/// #[mlint(deny(MLINT_SOME_LINT))]
/// fn some_item() {}
/// ```
#[proc_macro_attribute]
pub fn mlint(attr: TokenStream, item: TokenStream) -> TokenStream {
    attr::mlint::mlint(attr, item)
}

/// Extension attribute macro that transforms `expr?` into `route!(expr)`.
///
/// Designed for use with `#[chain(routeify, ...)]` to enable concise error
/// routing in chain functions using the `?` operator syntax.
///
/// # Example
///
/// ```rust,ignore
/// #[chain(routeify)]
/// fn handle_calc(args: EntryCalculate) -> Next {
///     let a = args.pick(&arg![f32]).to_result()?;
///     let op = args.pick(&arg![Operator]).to_result()?;
///     StateCalculate { number_a: a, operator: op, ... }.to_chain()
/// }
/// ```
#[cfg(feature = "extras")]
#[proc_macro_attribute]
pub fn routeify(attr: TokenStream, item: TokenStream) -> TokenStream {
    extensions::routeify::routeify_impl(attr, item)
}

/// Extension attribute macro that transforms `expr?` into `render_route!(expr)`.
///
/// Designed for use with `#[renderer(renderify, ...)]` or `#[help(renderify, ...)]`
/// to enable concise error routing in renderer and help functions using the `?`
/// operator syntax.
///
/// Unlike `#[routeify]` which routes errors to the chain pipeline via `route!`,
/// `#[renderify]` routes errors to the rendering pipeline via `render_route!`,
/// which matches the `RenderResult` return type of renderer and help functions.
///
/// # Example
///
/// ```rust,ignore
/// #[renderer(renderify)]
/// fn render_greeting(prev: Greeting) -> RenderResult {
///     let data = load_data()?;  // expands to render_route!(load_data())
///     r_println!("{data}");
///     Ok(RenderResult::new())
/// }
/// ```
#[cfg(feature = "extras")]
#[proc_macro_attribute]
pub fn renderify(attr: TokenStream, item: TokenStream) -> TokenStream {
    extensions::renderify::renderify_impl(attr, item)
}

/// Wraps a unit-returning function to produce a `RenderResult`.
///
/// The `#[buffer]` attribute macro injects a local `__render_result_buffer`
/// variable of type `::mingling::RenderResult` and changes the function's
/// return type to `::mingling::RenderResult`. Inside the body, use the
/// `r_print!` and `r_println!` macros to write into the buffer.
///
/// # Example
///
/// ```rust,ignore
/// use mingling::macros::{buffer, r_println};
///
/// #[buffer]
/// fn render_my_type(prev: MyType) {
///     r_println!("Value: {:?}", *prev);
/// }
/// ```
///
/// This expands to:
///
/// ```rust,ignore
/// fn render_my_type(prev: MyType) -> mingling::RenderResult {
///     let mut __render_result_buffer = mingling::RenderResult::new();
///     {
///         r_println!("Value: {:?}", *prev);
///     }
///     __render_result_buffer
/// }
/// ```
///
/// # Requirements
///
/// - The function must return `()` (unit).
/// - The function cannot be async.
#[proc_macro_attribute]
pub fn buffer(attr: TokenStream, item: TokenStream) -> TokenStream {
    extensions::buffer::buffer_impl(attr, item)
}

/// Prints text to a `RenderResult` buffer, with a trailing newline.
///
/// # Implicit buffer (inside `#[buffer]` functions)
///
/// ```rust,ignore
/// use mingling::macros::{buffer, r_println};
///
/// #[buffer]
/// fn render() {
///     r_println!("Hello, {}!", name);
/// }
/// ```
///
/// # Explicit buffer
///
/// Pass a `RenderResult` variable as the first argument:
///
/// ```rust,ignore
/// use mingling::macros::r_println;
/// use mingling::RenderResult;
///
/// let mut r = RenderResult::new();
/// r_println!(r, "value: {}", 42);
/// assert_eq!(&*r, "value: 42\n");
/// ```
#[proc_macro]
pub fn r_println(input: TokenStream) -> TokenStream {
    func::r_println::r_println(input)
}

/// Prints text to a `RenderResult` buffer, without a trailing newline.
///
/// # Implicit buffer (inside `#[buffer]` functions)
///
/// ```rust,ignore
/// use mingling::macros::{buffer, r_print};
///
/// #[buffer]
/// fn render() {
///     r_print!("Hello, ");
///     r_println!("world!");
/// }
/// ```
///
/// # Explicit buffer
///
/// ```rust,ignore
/// use mingling::macros::r_print;
/// use mingling::RenderResult;
///
/// let mut r = RenderResult::new();
/// r_print!(r, "value: {}", 42);
/// assert_eq!(&*r, "value: 42");
/// ```
#[proc_macro]
pub fn r_print(input: TokenStream) -> TokenStream {
    func::r_print::r_print(input)
}

/// Prints text to a `RenderResult` buffer (standard error style), with a trailing newline.
///
/// This macro works identically to `r_println!` but conceptually targets
/// "error output" — it writes into a `RenderResult` buffer with a trailing newline.
///
/// # Implicit buffer (inside `#[buffer]` functions)
///
/// ```rust,ignore
/// use mingling::macros::{buffer, r_eprintln};
///
/// #[buffer]
/// fn render() {
///     r_eprintln!("Error: {}", err_msg);
/// }
/// ```
///
/// # Explicit buffer
///
/// Pass a `RenderResult` variable as the first argument:
///
/// ```rust,ignore
/// use mingling::macros::r_eprintln;
/// use mingling::RenderResult;
///
/// let mut r = RenderResult::new();
/// r_eprintln!(r, "error: {}", 42);
/// assert_eq!(&*r, "error: 42\n");
/// ```
#[proc_macro]
pub fn r_eprintln(input: TokenStream) -> TokenStream {
    func::r_eprintln::r_eprintln(input)
}

/// Prints text to a `RenderResult` buffer (standard error style), without a trailing newline.
///
/// This macro works identically to `r_print!` but conceptually targets
/// "error output" — it writes into a `RenderResult` buffer without a trailing newline.
///
/// # Implicit buffer (inside `#[buffer]` functions)
///
/// ```rust,ignore
/// use mingling::macros::{buffer, r_eprint};
///
/// #[buffer]
/// fn render() {
///     r_eprint!("Error: ");
///     r_eprintln!("something went wrong");
/// }
/// ```
///
/// # Explicit buffer
///
/// ```rust,ignore
/// use mingling::macros::r_eprint;
/// use mingling::RenderResult;
///
/// let mut r = RenderResult::new();
/// r_eprint!(r, "error: ");
/// r_eprintln!(r, "42");
/// assert_eq!(&*r, "error: 42\n");
/// ```
#[proc_macro]
pub fn r_eprint(input: TokenStream) -> TokenStream {
    func::r_eprint::r_eprint(input)
}

/// Appends the contents of one `RenderResult` to another.
///
/// # Implicit buffer (inside `#[buffer]` functions)
///
/// ```rust,ignore
/// use mingling::macros::{buffer, r_append};
///
/// #[buffer]
/// fn render() {
///     let other = make_other_result();
///     r_append!(other);
/// }
/// ```
///
/// # Explicit buffer
///
/// ```rust,ignore
/// use mingling::macros::r_append;
/// use mingling::RenderResult;
///
/// let mut dst = RenderResult::new();
/// let src = RenderResult::from("hello");
/// r_append!(dst, src);
/// assert!(!dst.is_empty());
/// ```
#[proc_macro]
pub fn r_append(input: TokenStream) -> TokenStream {
    func::r_append::r_append(input)
}

/// Derive macro for treating a struct as its inner (wrapped) type.
///
/// The `#[derive(Wrap)]` macro generates:
/// - `From<Inner> for Self` — construct the wrapper from the inner value
/// - `From<Self> for Inner` — unwrap back to the inner value (i.e. `Into<Inner>`)
/// - `Deref` / `DerefMut` — delegate all methods to the inner value
///
/// # Inner field selection
///
/// - Tuple struct with one field → that field is the inner type
/// - Named struct with one field → that field is the inner type
/// - Named struct with multiple fields → mark exactly one field with `#[wrap]`;
///   the remaining fields are initialized with `Default::default()` when
///   constructing via `From<Inner>`
///
/// # Example
///
/// ```rust,ignore
/// use mingling::macros::Wrap;
///
/// #[derive(Wrap)]
/// struct Name(String);
///
/// #[derive(Wrap)]
/// struct Greeting {
///     name: String,
/// }
///
/// #[derive(Wrap)]
/// struct Task {
///     #[wrap]
///     content: String,
///     done: bool,
/// }
///
/// let name = Name::from("Mingling".to_string());
/// // `Deref` forwards methods to the inner `String`
/// assert_eq!(name.len(), 8);
/// let inner: String = name.into();
/// ```
#[proc_macro_derive(Wrap, attributes(wrap))]
pub fn derive_wrap(input: TokenStream) -> TokenStream {
    wrap::derive_wrap(input)
}

/// Derive macro for automatically implementing the `Grouped` trait on a struct.
///
/// The `#[derive(Grouped)]` macro:
/// 1. Implements `Grouped<crate::ThisProgram>`.
/// 2. Registers the type via `register_type!` so it's included in the program enum.
/// 3. Generates `Into<AnyOutput<Group>>` and `Into<ChainProcess<Group>>` conversions.
/// 4. Adds `to_chain()` and `to_render()` methods to the struct.
///
/// # Syntax
///
/// ```rust,ignore
/// #[derive(Grouped)]
/// struct MyStruct {
///     // ...
/// }
/// ```
///
/// # Example
///
/// ```rust,ignore
/// use mingling::macros::Grouped;
///
/// #[derive(Grouped)]
/// struct Greeting {
///     name: String,
/// }
/// ```
///
/// This is equivalent to using `#[derive(Grouped)]` but works with custom structs that
/// have named fields.
#[proc_macro_derive(Grouped, attributes(group))]
pub fn derive_grouped(input: TokenStream) -> TokenStream {
    grouped::derive_grouped(input)
}

/// Derive macro for automatically implementing the `EnumTag` trait on an enum
/// with unit-only variants.
///
/// The `#[derive(EnumTag)]` macro generates:
/// - `enum_info(&self) -> (&'static str, &'static str)` — returns (name, description)
///   for the current variant.
/// - `build_enum(name: String) -> Option<Self>` — constructs a variant from its
///   display name (or `#[enum_rename]` value).
/// - `enums() -> &'static [(&'static str, &'static str)]` — returns all (name, description)
///   pairs.
///
/// # Attributes
///
/// - `#[enum_desc("description text")]` — Provides a description for the variant.
/// - `#[enum_rename("display name")]` — Changes the display/build name of the variant.
///
/// # Syntax
///
/// ```rust,ignore
/// #[derive(EnumTag)]
/// enum Fruit {
///     #[enum_desc("A sweet red fruit")]
///     #[enum_rename("apple")]
///     Apple,
///
///     #[enum_desc("A yellow tropical fruit")]
///     #[enum_rename("banana")]
///     Banana,
/// }
/// ```
///
/// # Requirements
///
/// - Can only be derived for **enums** (not structs or unions).
/// - All variants must be **unit variants** (no fields).
/// - Each variant is optional; variants without attributes get their Rust name as display name
///   and an empty description.
#[proc_macro_derive(EnumTag, attributes(enum_desc, enum_rename))]
pub fn derive_enum_tag(input: TokenStream) -> TokenStream {
    enum_tag::derive_enum_tag(input)
}

/// Derive macro for `StructuralData`, marking a type as eligible for structured
/// structured output (JSON / YAML / TOML / RON).
///
/// The type must also implement `serde::Serialize` — the generated
/// `impl StructuralData` will fail to compile otherwise.
///
/// # Syntax
///
/// ```rust,ignore
/// use mingling::StructuralData;
/// use serde::Serialize;
///
/// #[derive(Serialize, StructuralData)]
/// struct Info {
///     name: String,
///     age: i32,
/// }
/// ```
#[cfg(feature = "structural_renderer")]
#[proc_macro_derive(StructuralData)]
pub fn derive_structural_data(input: TokenStream) -> TokenStream {
    derive::structural_data::derive_structural_data(input)
}

/// Derive macro for implementing both `Grouped` and `serde::Serialize` on a struct.
///
/// **This macro is only available with the `structural_renderer` feature.**
///
/// This is identical to `#[derive(Grouped)]` but also adds `#[derive(serde::Serialize)]`
/// to the struct, which is required for the structural renderer to serialize output
/// to formats like JSON, YAML, TOML, or RON.
///
/// # Syntax
///
/// ```rust,ignore
/// #[derive(GroupedSerialize)]
/// struct Info {
///     name: String,
///     age: i32,
/// }
/// ```
///
/// # Example
///
/// ```rust,ignore
/// use mingling::GroupedSerialize;
/// use serde::Serialize;
///
/// #[derive(GroupedSerialize)]
/// struct Info {
///     name: String,
///     age: i32,
/// }
/// ```
#[cfg(feature = "structural_renderer")]
#[proc_macro_derive(GroupedSerialize, attributes(group))]
pub fn derive_grouped_serialize(input: TokenStream) -> TokenStream {
    grouped::derive_grouped_serialize(input)
}

/// Generates the program enum and all collected types, chains, and renderers.
///
/// This macro **must** be called at the end of your program module to collect
/// all registered types, chains, renderers, and help requests into a single
/// program enum that implements `ProgramCollect`.
///
/// # Syntax
///
/// ```rust,ignore
/// gen_program!();
/// ```
///
/// # What it generates
///
/// The macro expands to:
/// 1. **`pub type Next = ChainProcess<ProgramName>`** — A convenience type alias
///    for use in chain function return types.
/// 2. **`program_comp_gen!(...)`** (with `comp` feature) — Generates completion infrastructure.
/// 3. **`program_fallback_gen!(...)`** — Generates `ErrorRendererNotFound` and `EntryFallback` types.
/// 4. **`program_final_gen!(...)`** — Generates the program enum with:
///    - An enum with all packed types as variants
///    - `Display` implementation for the enum
///    - `ProgramCollect` implementation dispatching to all registered renderers and chains
///    - A `new()` constructor returning `Program<ProgramName>`
///
/// # Example
///
/// ```rust,ignore
/// use mingling::macros::{dispatcher, chain, renderer, gen_program, RenderResult};
/// use std::io::Write;
///
/// dispatcher!("hello", HelloCommand => HelloEntry);
///
/// #[chain]
/// fn process(prev: HelloEntry) -> Next {
///     // ...
/// }
///
/// #[renderer]
/// fn render(prev: /* ... */) -> RenderResult {
///     let mut result = RenderResult::new();
///     writeln!(result, "Done!");
///     result
/// }
///
/// // Collect everything:
/// gen_program!();
/// ```
#[proc_macro]
pub fn gen_program(input: TokenStream) -> TokenStream {
    func::gen_program::gen_program_impl(input)
}

/// Internal macro used by `gen_program!` to generate the completion infrastructure for
/// shell completion support.
///
/// **This macro is only available with the `comp` feature.**
///
/// The `program_comp_gen!` macro generates:
/// 1. A hidden `__internal_completion_mod` module containing:
///    - A `CompletionContext` packed type (wrapping `ShellContext`), dispatched via `"__comp"`.
///    - A `CompletionSuggest` packed type (wrapping `(ShellContext, Suggest)`).
///    - An internal dispatcher (`CMDCompletion`) for the `"__comp"` command path.
/// 2. An internal chain function `__exec_completion` that:
///    - Reads a `ShellContext` from the packed `CompletionContext`.
///    - Calls `CompletionHelper::exec_completion::<ThisProgram>(&ctx)` to generate suggestions.
///    - Routes the result to the completion renderer via `CompletionSuggest`.
/// 3. An internal renderer `__render_completion` that renders the suggestions via
///    `CompletionHelper::render_suggest`.
///
/// It also imports the internal dispatcher from the generated module into the
/// parent scope for compile-time collection.
///
/// This macro is called automatically by `gen_program!` and should not be called
/// directly by user code.
#[cfg(feature = "comp")]
#[proc_macro]
pub fn program_comp_gen(input: TokenStream) -> TokenStream {
    func::program_comp_gen::program_comp_gen_impl(input)
}

/// Registers a type into the global packed types registry for inclusion in
/// the program enum generated by `gen_program!`.
///
/// This macro is called internally by `#[derive(Grouped)]` (`macro.derive_grouped.html`)
/// and is generally not needed in user code. However, it can be used for manual
/// registration if you are implementing custom type registration outside of
/// the standard macros.
///
/// # Syntax
///
/// ```rust,ignore
/// register_type!(MyType);
/// ```
///
/// Each call inserts the type's name into the `PACKED_TYPES` global set, which
/// is later consumed by `program_final_gen!` to generate enum variants.
///
/// # Panics
///
/// Panics if the global `PACKED_TYPES` mutex is poisoned.
#[proc_macro]
pub fn register_type(input: TokenStream) -> TokenStream {
    func::register_type::register_type_impl(input)
}

/// Registers a chain mapping function into the global chain registry.
///
/// This macro is called internally by `#[chain]` and is generally not needed
/// in user code. Each call stores a string entry containing the source-to-target
/// type mapping, which is later consumed by `gen_program!` to generate the
/// `has_chain` and `do_chain` dispatch logic in `ProgramCollect`.
///
/// The entry string format is a match arm: the source variant maps to a call
/// that converts the value into a `ChainProcess` via the destination type.
///
/// # Panics
///
/// Panics if the global `CHAINS` mutex is poisoned.
#[proc_macro]
pub fn register_chain(input: TokenStream) -> TokenStream {
    func::register_chain::register_chain_impl(input)
}

/// Registers a renderer mapping function into the global renderer registry.
///
/// This macro is called internally by `#[renderer]` and is generally not
/// needed in user code. Each call stores a string entry containing the
/// type-to-render mapping, which is later consumed by `gen_program!` to
/// generate the `has_renderer` and `render` dispatch logic in `ProgramCollect`.
///
/// The entry string format is a match arm: the type variant maps to a call
/// of the registered renderer function that produces a `RenderResult`.
///
/// # Panics
///
/// Panics if the global `RENDERERS` mutex is poisoned.
#[proc_macro]
pub fn register_renderer(input: TokenStream) -> TokenStream {
    func::register_renderer::register_renderer_impl(input)
}

/// Internal macro used by `gen_program!` to generate the fallback types for
/// error cases when no dispatcher or renderer is found.
///
/// This macro is called automatically by `gen_program!` and should not
/// be called directly by user code.
#[proc_macro]
pub fn program_fallback_gen(input: TokenStream) -> TokenStream {
    func::program_fallback_gen::program_fallback_gen_impl(input)
}

/// Internal macro used by `gen_program!` to generate the final program enum
/// and its `ProgramCollect` implementation.
///
/// This is the core code generation macro that:
/// 1. Collects all registered types (from `#[derive(Grouped)]`, etc.) and
///    creates an enum with each type as a variant.
/// 2. Generates the `Display` implementation for the enum.
/// 3. Generates the `ProgramCollect` implementation that dispatches to all
///    registered renderers, chains, help handlers, completions, and structural renderers.
/// 4. Adds a `new()` constructor on the enum returning `Program<EnumName>`.
///
/// The generated enum's representation type (`#[repr(u8)]`, `#[repr(u16)]`, etc.)
/// is automatically chosen based on the number of variants.
///
/// This macro is called automatically by `gen_program!` and should not
/// be called directly by user code.
///
/// # Syntax
///
/// ```rust,ignore
/// program_final_gen!();
/// ```
///
/// # Generated code structure
///
/// ```rust,ignore
/// #[repr(u8)]
/// pub enum ThisProgram {
///     TypeA,
///     TypeB,
///     // ...
/// }
///
/// impl ProgramCollect for MyProgram {
///     type Enum = MyProgram;
///     type ResultEmpty = ResultEmpty;
///     fn render(any) -> RenderResult { /* dispatches to all registered renderers */ }
///     fn do_chain(any) -> ChainProcess { /* dispatches to all registered chain steps */ }
///     fn render_help(any) -> RenderResult { /* dispatches to all registered help handlers */ }
///     fn has_renderer(any) -> bool { /* checks renderer registry */ }
///     fn has_chain(any) -> bool { /* checks chain registry */ }
///     // (with comp feature) fn do_comp(...)
///     // (with structural_renderer feature) fn structural_render(...)
/// }
///
/// impl MyProgram {
///     pub fn new() -> Program<MyProgram> { Program::new() }
/// }
/// ```
#[proc_macro]
pub fn program_final_gen(input: TokenStream) -> TokenStream {
    func::program_final_gen::program_final_gen_impl(input)
}

/// Builds a `Suggest` instance with inline suggestion items.
///
/// **This macro is only available with the `comp` feature.**
///
/// The `suggest!` macro provides a concise syntax for creating shell completion
/// suggestions. Each item can be either a simple flag or a flag with a description.
///
/// # Syntax
///
/// ```rust,ignore
/// // Empty suggestions:
/// suggest!()
///
/// // Simple flags (no description):
/// suggest! { "--flag1", "--flag2" }
///
/// // Flags with descriptions:
/// suggest! {
///     "--name": "User's name",
///     "--age":  "User's age"
/// }
///
/// // Mixed:
/// suggest! {
///     "--name": "User's name",
///     "--verbose"
/// }
/// ```
///
/// # Example
///
/// ```rust,ignore
/// use mingling::macros::{completion, suggest};
/// use mingling::{ShellContext, Suggest};
///
/// #[completion(MyEntry)]
/// fn complete(ctx: &ShellContext) -> Suggest {
///     if ctx.typing_argument() {
///         return suggest! {
///             "--name": "Provide a name",
///             "--type": "Select a type"
///         }.strip_typed_argument(ctx);
///     }
///     suggest!()
/// }
/// ```
///
/// # Related
///
/// - `suggest_enum!`(`macro.suggest_enum.html`) — Build suggestions from an `EnumTag` enum.
#[cfg(feature = "comp")]
#[proc_macro]
pub fn suggest(input: TokenStream) -> TokenStream {
    suggest::suggest(input)
}

/// Builds a `Suggest` instance from an `EnumTag` enum's variants.
///
/// **This macro is only available with the `comp` feature.**
///
/// The `suggest_enum!` macro iterates over all variants of an `EnumTag`-derived
/// enum and creates suggestion items using each variant's display name
/// (from `#[enum_rename]`) and description (from `#[enum_desc]`).
///
/// # Syntax
///
/// ```rust,ignore
/// suggest_enum!(MyEnumType);
/// ```
///
/// # Example
///
/// ```rust,ignore
/// use mingling::macros::{completion, suggest_enum};
/// use mingling::{ShellContext, Suggest, EnumTag};
///
/// #[derive(EnumTag)]
/// enum Fruit {
///     #[enum_desc("A sweet red fruit")]
///     #[enum_rename("apple")]
///     Apple,
///     #[enum_desc("A yellow tropical fruit")]
///     #[enum_rename("banana")]
///     Banana,
/// }
///
/// #[completion(MyEntry)]
/// fn complete(ctx: &ShellContext) -> Suggest {
///     if ctx.filling_argument_first("--fruit") {
///         return suggest_enum!(Fruit);
///     }
///     suggest!()
/// }
/// ```
///
/// # Generated code equivalent
///
/// ```rust,ignore
/// {
///     let mut enum_suggest = Suggest::new();
///     for (name, desc) in <Fruit>::enums() {
///         if desc.is_empty() {
///             enum_suggest.insert(SuggestItem::new(name.to_string()));
///         } else {
///             enum_suggest.insert(SuggestItem::new_with_desc(name.to_string(), desc.to_string()));
///         }
///     }
///     enum_suggest
/// }
/// ```
///
/// # Related
///
/// - `suggest!`(macro.suggest.html) — Build suggestions with inline syntax.
/// - `EnumTag`(derive.EnumTag.html) — The derive macro required for the enum type.
#[cfg(feature = "comp")]
#[proc_macro]
pub fn suggest_enum(input: TokenStream) -> TokenStream {
    func::suggest_enum::suggest_enum(input)
}
