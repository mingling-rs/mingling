//! Proc-macro engine of the Mingling CLI framework.
//!
//! This crate is the **macro layer** of Mingling. Each `#[attribute]` or `!`-callable
//! macro collects metadata into **compile-time global registries** (`OnceLock<Mutex<BTreeSet>>`).
//! At the end, `gen_program!` reads all registries and generates the final program struct
//! with all dispatchers, chains, renderers, and completions wired together.
//!
//! # How Macros Work Together
//!
//! The Mingling macro pipeline has three phases:
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────────────┐
//! │  Phase 1: Declaration                                            │
//! │                                                                  │
//! │  dispatcher!  pack!     node!       #[derive(Grouped)]           │
//! │  │            │         │           │                            │
//! │  V            V         V           V                            │
//! │  Declares     Wraps a   Builds      Makes a type                 │
//! │  a command    type in   a command   recognizable                 │
//! │  entry        a new     path Node   by the                       │
//! │               type                  framework                    │
//! ├──────────────────────────────────────────────────────────────────┤
//! │  Phase 2: Registration (at compile time, in statics)             │
//! │                                                                  │
//! │  #[chain]       #[renderer]     #[help]     #[completion]        │
//! │  │              │               │           │                    │
//! │  V              V               V           V                    │
//! │  Registers      Registers       Registers   Registers            │
//! │  type → chain   type → renderer type → help completion logic     │
//! ├──────────────────────────────────────────────────────────────────┤
//! │  Phase 3: Code Generation                                        │
//! │                                                                  │
//! │  gen_program!()                                                  │
//! │  │                                                               │
//! │  V                                                               │
//! │  Reads all registries → generates ThisProgram with:              │
//! │    • ProgramCollect impl (dispatch/render/chain dispatch tree)   │
//! │    • Fallback types (ErrorDispatcherNotFound, etc.)              │
//! │    • Completion logic (if `comp` feature enabled)                │
//! └──────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Macro Categories
//!
//! ## Phase 1: Command & Type Declaration
//!
//! | Macro | What it does |
//! |-------|-------------|
//! | `dispatcher!` | Declares a command entry point and its argument type |
//! | `dispatcher_clap!` | Like `dispatcher!` but powered by `clap::Parser` |
//! | `node!` | Builds a [`Node`](https://docs.rs/mingling/latest/mingling/struct.Node.html) from a dot-separated path string |
//! | `pack!` | Creates a newtype wrapper around an inner type for use in Chain/Renderer |
//! | `pack_structural!` | Like `pack!` but also derives `StructuralData` for structured output |
//! | `pack_err!` | Creates an error struct with automatic `name` field |
//! | `pack_err_structural!` | Like `pack_err!` but also derives `StructuralData` for structured output |
//! | `entry!` | Creates a packed entry from string literals |
//! | [`#[derive(Grouped)]`](derive@Grouped) | Makes a type recognizable by the framework's type registry |
//! | `#[derive(StructuralData)]` | Marks a type as eligible for structured output (JSON/YAML/etc.) |
//! | [`#[derive(EnumTag)]`](derive@EnumTag) | Adds enum variant metadata (name, description) |
//!
//! ## Phase 2: Processing & Rendering Registration
//!
//! | Macro | What it does |
//! |-------|-------------|
//! | [`#[chain]`](attr.chain.html) | Transforms a function into a chain processing step |
//! | [`#[renderer]`](attr.renderer.html) | Transforms a function into a renderer for a type |
//! | [`#[help]`](attr.help.html) | Defines help output for a command entry type |
//! | `route!` | Routes execution depending on a condition |
//! | `empty_result!` | Returns an empty result for early termination |
//! | [`#[completion]`](attr.completion.html) | Registers a shell completion handler |
//!
//! ## Phase 3: Program Generation
//!
//! | Macro | What it does |
//! |-------|-------------|
//! | `gen_program!` | **Final step**: reads all registries and generates the full program |
//! | `suggest!` | Generates suggestion logic for a dispatcher |
//! | `suggest_enum!` | Generates suggestion logic for an enum dispatcher |
//!
//! ## Internal (used by the macros above)
//!
//! | Macro | What it does |
//! |-------|-------------|
//! | `register_type!` | Registers a type in the packed-type registry |
//! | `register_chain!` | Registers a chain mapping in the chain registry |
//! | `register_renderer!` | Registers a renderer mapping in the renderer registry |
//! | `register_dispatcher!` | Registers a dispatcher for the `dispatch_tree` feature |
//! | `register_help!` | Registers a help request handler |
//! | `program_fallback_gen!` | Generates fallback error types |
//! | `program_final_gen!` | Generates the `ProgramCollect` impl and `ThisProgram` struct |
//! | `program_comp_gen!` | Generates completion logic |
//! | [`#[program_setup]`](attr.program_setup.html) | Declares a custom program setup step |
//!
//! # Feature Gates
//!
//! Some macros are only available when certain Cargo features are enabled:
//!
//! | Feature | Macros enabled |
//! |---------|---------------|
//! | `clap` | `dispatcher_clap!` |
//! | `comp` | [`#[completion]`](attr.completion.html), `suggest!`, `suggest_enum!` |
//! | `extra_macros` | `entry!`, `empty_result!`, `route!`, [`#[program_setup]`](attr.program_setup.html), `group!` |
//! | `dispatch_tree` | `register_dispatcher!` (enables trie-based command dispatch) |
//! | `structural_renderer` | `#[derive(StructuralData)]`, `pack_structural!`, `pack_err_structural!`, `group_structural!` |
//! | `structural_renderer` + `extra_macros` | `group_structural!`, `pack_err_structural!` |
//! | `async` | Enables async `#[chain]` functions |
//! | `repl` | Enables REPL execution loop |
//!
//! # The Compile-Time Registry System
//!
//! Macros in this crate do **not** generate all code immediately. Instead, they
//! store entries into `OnceLock<Mutex<BTreeSet<String>>>` statics. These string
//! entries contain the **token-stream representation** of match arms, type mappings,
//! and struct definitions.
//!
//! When `gen_program!` is called, it reads all registries, concatenates their
//! entries, and emits the complete program:
//!
//! ```rust,ignore
//! // Example of what gen_program! generates (simplified):
//! impl ProgramCollect for ThisProgram {
//!     fn build_dispatcher_not_found(args: Vec<String>) -> AnyOutput {
//!         AnyOutput::new(ErrorDispatcherNotFound::new(args))
//!     }
//!     fn has_chain(any: &AnyOutput) -> bool {
//!         match any.member_id() {
//!             MyType => true,  // ← collected from #[chain] macros
//!             _ => false,
//!         }
//!     }
//!     fn has_renderer(any: &AnyOutput) -> bool {
//!         match any.member_id() {
//!             MyType => true,  // ← collected from #[renderer] macros
//!             // When `structural_renderer` is enabled, ALL registered types
//!             // return true — non-structural types fall through to render
//!             // a `ResultEmpty` value (via structural_render fallback).
//!             _ => false,
//!         }
//!     }
//! }
//! ```

use proc_macro::TokenStream;
use quote::quote;
use std::collections::BTreeSet;
use std::sync::Mutex;
use std::sync::OnceLock;
use syn::parse_macro_input;

mod chain;
#[cfg(feature = "comp")]
mod completion;
#[cfg(feature = "dispatch_tree")]
mod dispatch_tree_gen;
mod dispatcher;
#[cfg(feature = "clap")]
mod dispatcher_clap;
#[cfg(feature = "extra_macros")]
mod entry;
mod enum_tag;
#[cfg(feature = "extra_macros")]
mod group_impl;
mod grouped;
mod help;
mod node;
mod pack;
#[cfg(feature = "extra_macros")]
mod pack_err;
#[cfg(feature = "extra_macros")]
mod program_setup;
mod renderer;
mod res_injection;
#[cfg(feature = "structural_renderer")]
mod structural_data;
#[cfg(feature = "comp")]
mod suggest;

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
/// `pack_structural!` / `group_structural!`.
#[cfg(feature = "structural_renderer")]
pub(crate) static STRUCTURED_TYPES: Registry = OnceLock::new();

#[cfg(feature = "comp")]
pub(crate) static COMPLETIONS: Registry = OnceLock::new();

#[cfg(feature = "dispatch_tree")]
pub(crate) static COMPILE_TIME_DISPATCHERS: Registry = OnceLock::new();

pub(crate) static PACKED_TYPES: Registry = OnceLock::new();
pub(crate) static CHAINS: Registry = OnceLock::new();
pub(crate) static RENDERERS: Registry = OnceLock::new();
pub(crate) static CHAINS_EXIST: Registry = OnceLock::new();
pub(crate) static RENDERERS_EXIST: Registry = OnceLock::new();
pub(crate) static HELP_REQUESTS: Registry = OnceLock::new();

/// Checks if a variant name already exists in a registered set.
/// Returns a `compile_error` token stream if a duplicate is found.
pub(crate) fn check_duplicate_variant(
    set: &std::collections::BTreeSet<String>,
    entry_str: &str,
    variant_name: &str,
    kind: &str,
    error_span: proc_macro2::Span,
) -> Result<(), proc_macro2::TokenStream> {
    for existing in set.iter() {
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
/// Handles both "StructName => Variant," and "Self::Variant => ..." formats.
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

/// Registers an outside-type as a member of a program group without modifying its definition.
///
/// This macro allows you to use outside-types from external crates (like `std::io::Error`)
/// within the Mingling framework by generating a `Grouped` implementation and registering
/// the type's simple name as an enum variant.
///
/// # Syntax
///
/// ```rust,ignore
/// group!(std::io::Error);
/// group!(ParseIntError);
/// ```
///
/// The type is registered under the default program (`crate::ThisProgram`).
///
/// # How it works
///
/// The macro generates a module containing:
/// - A `use` import for the program path and the outside-type
/// - An `impl Grouped<Program>` for the outside-type
/// - A `register_type!` call with the type's simple name
///
/// The type's simple name (e.g. `Error`) is used as the enum variant in the generated
/// program enum, just like `#[derive(Grouped)]` or `pack!`.
///
/// # Example
///
/// ```rust,ignore
/// use mingling::macros::group;
///
/// // Register std::io::Error as a group member
/// group!(std::io::Error);
/// ```
///
/// After expansion, the type can be used in chains and renderers like any
/// `#[derive(Grouped)]` type.
///
/// This macro is only available with the `extra_macros` feature.
#[cfg(feature = "extra_macros")]
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
/// Requires the `structural_renderer` and `extra_macros` features.
#[cfg(all(feature = "structural_renderer", feature = "extra_macros"))]
#[proc_macro]
pub fn group_structural(input: TokenStream) -> TokenStream {
    structural_data::group_structural(input)
}

/// Creates a `Node` from a dot-separated path string.
///
/// Each segment is converted to kebab-case (unless it starts with `_`).
/// Segments are joined via `.join()` calls, building a path hierarchy for
/// command matching.
///
/// # Syntax
///
/// ```rust,ignore
/// node!("subcommand")
/// node!("sub.subsub")
/// node!("")           // empty → Node::default()
/// ```
///
/// # Example
///
/// ```rust,ignore
/// use mingling::macros::node;
///
/// // Creates a single-level node for "hello"
/// let n = node!("hello");
///
/// // Creates a two-level node for "remote control"
/// let n = node!("remote.control");
/// ```
///
/// # Internals
///
/// The generated code is equivalent to:
/// ```rust,ignore
/// Node::default().join("hello")
/// Node::default().join("remote").join("control")
/// ```
///
/// This macro is typically used internally by `dispatcher!` and should rarely
/// need to be called directly.
#[proc_macro]
pub fn node(input: TokenStream) -> TokenStream {
    node::node(input)
}

/// Creates a type-safe wrapper struct around an inner type, with automatic
/// trait implementations for use in the Mingling chain/render pipeline.
///
/// The generated struct implements: `From`/`Into`, `AsRef`/`AsMut`, `Deref`/`DerefMut`,
/// `Default` (conditional on inner type), and conversion into `AnyOutput` /
/// `ChainProcess` for routing.
///
/// # Syntax
///
/// ```rust,ignore
/// // Default program name (uses `ThisProgram`):
/// pack!(TypeName = InnerType);
///
/// // Explicit program name:
/// pack!(MyProgram, TypeName = InnerType);
/// ```
///
/// # Example
///
/// ```rust,ignore
/// use mingling::macros::pack;
///
/// // Creates `Hello` wrapping `String`, registered under `ThisProgram`:
/// pack!(Hello = String);
///
/// // Creates `Greeting` wrapping `String`, registered under `MyApp`:
/// pack!(MyApp, Greeting = String);
/// ```
///
/// After expansion, `Hello` has:
/// - `Hello::new(String)` — constructor
/// - `Hello::to_chain()` — routes to the next chain processor
/// - `Hello::to_render()` — routes to a renderer
/// - `From<String> for Hello`, `From<Hello> for String`
/// - `Deref<Target = String>`, `DerefMut`
/// - `AsRef<String>`, `AsMut<String>`
/// - `Default` if `String: Default`
/// - `Into<AnyOutput<ThisProgram>>`, `Into<ChainProcess<ThisProgram>>`
/// - Implements `Grouped<ThisProgram>` with `member_id()` returning the enum variant
///
/// The struct is also registered via `register_type!` so that `gen_program!`
/// can include it in the program enum.
///
/// When the `structural_renderer` feature is enabled, the struct also gets
/// `#[derive(serde::Serialize)]`.
#[proc_macro]
pub fn pack(input: TokenStream) -> TokenStream {
    pack::pack(input)
}

/// Like `pack!` but also marks the type as supporting structured output
/// (JSON / YAML / TOML / RON) via `StructuralData`.
///
/// # Syntax
///
/// ```rust,ignore
/// pack_structural!(Info = (String, i32));
/// ```
///
/// This is equivalent to:
/// ```rust,ignore
/// pack!(Info = (String, i32));
/// impl ::mingling::StructuralData for Info {}
/// ```
///
/// Requires the `structural_renderer` feature.
#[cfg(feature = "structural_renderer")]
#[proc_macro]
pub fn pack_structural(input: TokenStream) -> TokenStream {
    structural_data::pack_structural(input)
}

/// Creates an error struct with a `name: String` field and optional `info: Type` field.
///
/// This macro provides a concise way to define error types that implement `Grouped`
/// and are registered for inclusion in the program enum.
///
/// The `name` field is automatically set to the snake_case version of the struct name
/// at compile time.
///
/// # Syntax
///
/// Two forms are supported:
///
/// ```rust,ignore
/// // Simple form — generates a struct with only `name: String` and a `Default` impl:
/// pack_err!(ErrorNotFound);
///
/// // Typed form — generates a struct with `name: String` + `info: Type` and a `new(info)` constructor:
/// pack_err!(ErrorNotDir = PathBuf);
/// ```
///
/// # Generated code
///
/// For `pack_err!(ErrorNotFound)`:
///
/// ```rust,ignore
/// #[derive(::mingling::Grouped)]
/// pub struct ErrorNotFound {
///     name: String,
/// }
///
/// impl Default for ErrorNotFound {
///     fn default() -> Self {
///         Self {
///             name: "error_not_found".into(),
///         }
///     }
/// }
/// ```
///
/// For `pack_err!(ErrorNotDir = PathBuf)`:
///
/// ```rust,ignore
/// #[derive(::mingling::Grouped)]
/// pub struct ErrorNotDir {
///     name: String,
///     info: PathBuf,
/// }
///
/// impl ErrorNotDir {
///     pub fn new(info: PathBuf) -> Self {
///         Self {
///             name: "error_not_dir".into(),
///             info,
///         }
///     }
/// }
/// ```
///
/// When the `structural_renderer` feature is enabled, the struct also gets
/// `#[derive(serde::Serialize)]`.
///
/// This macro is only available with the `extra_macros` feature.
#[cfg(feature = "extra_macros")]
#[proc_macro]
pub fn pack_err(input: TokenStream) -> TokenStream {
    pack_err::pack_err(input)
}

/// Like `pack_err!` but also marks the type for structured output
/// (JSON / YAML / TOML / RON) via `StructuralData`.
///
/// # Syntax
///
/// ```rust,ignore
/// pack_err_structural!(ErrorNotFound);
/// pack_err_structural!(ErrorNotDir = PathBuf);
/// ```
///
/// Requires the `structural_renderer` and `extra_macros` features.
#[cfg(all(feature = "structural_renderer", feature = "extra_macros"))]
#[proc_macro]
pub fn pack_err_structural(input: TokenStream) -> TokenStream {
    pack_err::pack_err_structural(input)
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
#[cfg(feature = "extra_macros")]
#[proc_macro]
pub fn route(input: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(input as syn::Expr);
    let expanded = quote! {
        match #expr {
            Ok(r) => r,
            Err(e) => return ::mingling::Routable::to_chain(e),
        }
    };
    TokenStream::from(expanded)
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
#[cfg(feature = "extra_macros")]
#[proc_macro]
pub fn empty_result(_input: TokenStream) -> TokenStream {
    let expanded = quote! {
        <crate::ResultEmpty as ::mingling::Grouped::<crate::ThisProgram>>::to_chain(crate::ResultEmpty)
    };
    TokenStream::from(expanded)
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
/// ## Abbreviated syntax (requires `extra_macros` feature)
///
/// When the `extra_macros` feature is enabled, the `CommandStruct => EntryStruct`
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
/// // Abbreviated form (extra_macros required):
/// // dispatcher!("remote.add");  // → CMDRemoteAdd, EntryRemoteAdd
/// ```
///
/// # How it works
///
/// The macro generates:
///
/// 1. **Entry struct** — A `pack!`-style wrapper around `Vec<String>` (the raw args).
///    Registered in the program enum via `register_type!`.
/// 2. **Dispatcher struct** — A zero-sized struct implementing [`Dispatcher<Program>`]:
///    - `node()` returns the [`Node`] hierarchy for the command path.
///    - `begin(args)` wraps `args` into the entry type and routes to chain.
///    - `clone_dispatcher()` returns a boxed clone.
/// 3. **Registration** — If the `dispatch_tree` feature is enabled, also calls
///    `register_dispatcher!` for compile-time trie construction.
///
/// With the `comp` feature, the entry type also implements `CompletionEntry`
/// for providing shell completion suggestions.
///
/// # See also
///
/// - `dispatcher_clap!` — For clap-powered argument parsing.
/// - `node!` — For building custom [`Node`] paths.
/// - [`#[chain]`](attr.chain.html) — For processing the dispatched entry.
///
/// [`ChainProcess`]: https://docs.rs/mingling/latest/mingling/enum.ChainProcess.html
/// [`Dispatcher<Program>`]: https://docs.rs/mingling/latest/mingling/trait.Dispatcher.html
/// [`Node`]: https://docs.rs/mingling/latest/mingling/struct.Node.html
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
/// use mingling::macros::{chain, pack, gen_program};
///
/// pack!(MyOutput = String);
///
/// #[chain]
/// fn greet(prev: HelloEntry) -> Next {
///     let name = prev.first().cloned().unwrap_or_else(|| "World".to_string());
///     MyOutput::new(name)
/// }
/// ```
///
/// # Sync Example with Resource Injection
///
/// ```rust,ignore
/// use mingling::macros::{chain, pack, gen_program};
///
/// #[derive(Default, Clone)]
/// struct UserName(String);
///
/// pack!(Greeting = String);
/// pack!(DisplayCount = ());
///
/// #[chain]
/// fn greet(prev: HelloEntry, user_name: &UserName, count: &mut u64) -> Next {
///     *count += 1;
///     Greeting::new(format!("Hello, {}!", user_name.0))
/// }
/// ```
///
/// # Async Example (with `async` feature)
///
/// ```rust,ignore
/// use mingling::macros::{chain, pack, gen_program};
///
/// pack!(MyOutput = String);
///
/// #[chain]
/// async fn greet(prev: HelloEntry) -> Next {
///     let name = prev.first().cloned().unwrap_or_else(|| "World".to_string());
///     some_async_fn(&name).await;
///     MyOutput::new(name)
/// }
/// ```
///
/// # Async Example with Immutable Resource Injection
///
/// ```rust,ignore
/// use mingling::macros::{chain, pack, gen_program};
///
/// pack!(MyOutput = String);
///
/// #[chain]
/// async fn greet(prev: HelloEntry, prefix: &Prefix) -> Next {
///     let name = prev.first().cloned().unwrap_or_else(|| "World".to_string());
///     some_async_fn(&name).await;
///     MyOutput::new(format!("{}{}", prefix.0, name))
/// }
/// ```
///
/// # Async Example with Mutable Resource Injection
///
/// ```rust,ignore
/// use mingling::macros::{chain, pack, gen_program};
///
/// pack!(MyOutput = String);
///
/// #[chain]
/// async fn greet(prev: HelloEntry, ec: &mut ResExitCode) -> Next {
///     let name = prev.first().cloned().unwrap_or_else(|| "World".to_string());
///     ec.exit_code = 42;
///     some_async_fn(&name).await;
///     MyOutput::new(name)
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
/// use mingling::macros::{renderer, pack, gen_program};
/// use std::io::Write;
///
/// pack!(Greeting = String);
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
/// - `ErrorDispatcherNotFound` — triggered when no matching dispatcher is found
///
/// ```rust,ignore
/// #[renderer]
/// fn fallback_dispatcher_not_found(prev: ErrorDispatcherNotFound) -> RenderResult {
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
    renderer::renderer_attr(attr, item)
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
#[cfg(feature = "extra_macros")]
#[proc_macro_attribute]
pub fn program_setup(attr: TokenStream, item: TokenStream) -> TokenStream {
    program_setup::setup_attr(attr, item)
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
/// via `pack!` or `dispatcher!`) with test data, typically used in unit tests
/// or quick prototypes.
///
/// # Syntax
///
/// Two forms:
///
/// ```rust,ignore
/// // Named form — wraps into a specific pack type:
/// entry!(MyEntry, ["a", "b", "c"])
/// // Expands to: MyEntry::new(vec!["a".to_string(), "b".to_string(), "c".to_string()])
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
/// // Named form (with a specific pack type):
/// let args = entry!(MyEntry, ["--name", "Alice", "--count", "5"]);
///
/// // Bracket form (type inference):
/// let args: Vec<String> = entry!["hello", "world"];
/// ```
///
/// # See also
///
/// - `pack!` — For creating the wrapper types used with `entry!`.
/// - `dispatcher!` — Which implicitly creates entry types via `pack!`.
#[cfg(feature = "extra_macros")]
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
    help::register_help(input)
}

/// Registers a dispatcher at compile time for the `dispatch_tree` feature.
///
/// This macro is called internally by `dispatcher!` when the `dispatch_tree`
/// feature is enabled. Each call stores the node name into the global
/// `COMPILE_TIME_DISPATCHERS` registry and generates a static variable for the
/// dispatcher instance. This data is later consumed by `gen_program!` to
/// generate a character-level **Trie** for efficient command dispatch.
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
/// - `dispatch_tree_gen` module — The trie generation logic.
#[proc_macro]
pub fn register_dispatcher(input: TokenStream) -> TokenStream {
    dispatcher::register_dispatcher(input)
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
/// use mingling::macros::{help, pack, gen_program};
/// use mingling::{prelude::*, setup::BasicProgramSetup, RenderResult};
/// use std::io::Write;
///
/// pack!(MyEntry = Vec<String>);
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
pub fn help(_attr: TokenStream, item: TokenStream) -> TokenStream {
    help::help_attr(item)
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
/// This is equivalent to using `pack!` but works with custom structs that
/// have named fields. For simple wrappers, prefer `pack!`.
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
    structural_data::derive_structural_data(input)
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
/// 3. **`program_fallback_gen!(...)`** — Generates `ErrorRendererNotFound` and `ErrorDispatcherNotFound` types.
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
pub fn gen_program(_input: TokenStream) -> TokenStream {
    #[cfg(feature = "comp")]
    let comp_gen = quote! {
        ::mingling::macros::program_comp_gen!();
    };

    #[cfg(not(feature = "comp"))]
    let comp_gen = quote! {};

    TokenStream::from(quote! {
        /// Alias for the current program type `crate::ThisProgram`
        pub type Next = ::mingling::ChainProcess<crate::ThisProgram>;

        impl ::mingling::Routable<crate::ThisProgram> for ::mingling::ChainProcess<crate::ThisProgram>
        {
            fn to_chain(self) -> ::mingling::ChainProcess<crate::ThisProgram> {
                match self {
                    ::mingling::ChainProcess::Ok((any, _)) => {
                        ::mingling::ChainProcess::Ok((any, mingling::NextProcess::Chain))
                    }
                    other => other,
                }
            }

            fn to_render(self) -> ::mingling::ChainProcess<crate::ThisProgram> {
                match self {
                    ::mingling::ChainProcess::Ok((any, _)) => {
                        ::mingling::ChainProcess::Ok((any, mingling::NextProcess::Renderer))
                    }
                    other => other,
                }
            }
        }

        #comp_gen
        ::mingling::macros::program_fallback_gen!();
        ::mingling::macros::program_final_gen!();
    })
}

/// Internal macro used by `gen_program!` to generate completion infrastructure.
///
/// **This macro is only available with the `comp` feature.**
///
/// This is an internal macro and should not be called directly by user code.
/// It generates a completion dispatcher, the `CompletionContext` type, and
/// the execution/render logic for shell completion.
///
/// The generated module `__completion_gen` contains:
/// - A `__comp` dispatcher that routes completion requests
/// - A `__exec_completion` chain that processes `CompletionContext` into `CompletionSuggest`
/// - A `__render_completion` renderer that outputs completion suggestions
#[proc_macro]
#[cfg(feature = "comp")]
pub fn program_comp_gen(_input: TokenStream) -> TokenStream {
    #[cfg(feature = "async")]
    let fn_exec_comp = quote! {
        #[doc(hidden)]
        #[::mingling::macros::chain]
        pub async fn __exec_completion(prev: CompletionContext) -> Next {
            use ::mingling::Grouped;

            let read_ctx = ::mingling::ShellContext::try_from(prev.inner);
            match read_ctx {
                Ok(ctx) => {
                    let suggest = ::mingling::CompletionHelper::exec_completion::<crate::ThisProgram>(&ctx);
                    crate::CompletionSuggest::new((ctx, suggest)).to_render()
                }
                Err(_) => std::process::exit(1),
            }
        }
    };

    #[cfg(not(feature = "async"))]
    let fn_exec_comp = quote! {
        #[doc(hidden)]
        #[::mingling::macros::chain]
        pub fn __exec_completion(prev: CompletionContext) -> Next {
            use ::mingling::Grouped;

            let read_ctx = ::mingling::ShellContext::try_from(prev.inner);
            match read_ctx {
                Ok(ctx) => {
                    let suggest = ::mingling::CompletionHelper::exec_completion::<crate::ThisProgram>(&ctx);
                    crate::CompletionSuggest::new((ctx, suggest)).to_render()
                }
                Err(_) => std::process::exit(1),
            }
        }
    };

    #[cfg(feature = "dispatch_tree")]
    let internal_dispatcher_comp = quote! {
        use __internal_completion_mod::__internal_dispatcher_comp;
    };

    #[cfg(not(feature = "dispatch_tree"))]
    let internal_dispatcher_comp = quote! {};

    let comp_dispatcher = quote! {
        #[doc(hidden)]
        mod __internal_completion_mod {
            use ::mingling::Grouped;
            ::mingling::macros::dispatcher!("__comp", CMDCompletion => CompletionContext);
            ::mingling::macros::pack!(
                CompletionSuggest = (::mingling::ShellContext, ::mingling::Suggest)
            );
        }
        #internal_dispatcher_comp
        use __internal_completion_mod::CompletionContext;
        use __internal_completion_mod::CompletionSuggest;
        pub use __internal_completion_mod::CMDCompletion;

        #fn_exec_comp

        ::mingling::macros::register_type!(CompletionContext);

        #[allow(unused)]
        #[doc(hidden)]
        #[::mingling::macros::renderer]
        pub fn __render_completion(prev: CompletionSuggest) -> ::mingling::RenderResult {
            let result = ::mingling::RenderResult::default();
            let (ctx, suggest) = prev.inner;
            ::mingling::CompletionHelper::render_suggest::<crate::ThisProgram>(ctx, suggest);
            result
        }
    };

    TokenStream::from(comp_dispatcher)
}

/// Registers a type into the global packed types registry for inclusion in
/// the program enum generated by `gen_program!`.
///
/// This macro is called internally by `pack!` and `#[derive(Grouped)]`(`macro.derive_grouped.html`)
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
    let type_ident = parse_macro_input!(input as syn::Ident);
    let entry_str = type_ident.to_string();

    get_global_set(&PACKED_TYPES)
        .lock()
        .unwrap()
        .insert(entry_str);

    TokenStream::new()
}

/// Registers a chain mapping from a previous type to a chain struct.
///
/// This macro is called internally by `#[chain]`(macro.chain.html) and is
/// generally not needed in user code. It inserts entries into the global
/// `CHAINS` and `CHAINS_EXIST` registries.
///
/// # Syntax
///
/// ```rust,ignore
/// register_chain!(PreviousType, ChainStruct);
/// ```
///
/// The `PreviousType` is the input type of the chain step, and `ChainStruct`
/// is the generated struct that implements the `Chain` trait.
#[proc_macro]
pub fn register_chain(input: TokenStream) -> TokenStream {
    chain::register_chain(input)
}

/// Registers a renderer mapping from a type to a renderer struct.
///
/// This macro is called internally by `#[renderer]`(macro.renderer.html) and is
/// generally not needed in user code. It inserts entries into the global
/// `RENDERERS`, `RENDERERS_EXIST` and (with `structural_renderer` feature)
/// `STRUCTURAL_RENDERERS` registries.
///
/// # Syntax
///
/// ```rust,ignore
/// register_renderer!(PreviousType, RendererStruct);
/// ```
///
/// The `PreviousType` is the input type of the renderer, and `RendererStruct`
/// is the generated struct that implements the `Renderer` trait.
#[proc_macro]
pub fn register_renderer(input: TokenStream) -> TokenStream {
    renderer::register_renderer(input)
}

/// Internal macro used by `gen_program!` to generate fallback types.
///
/// This macro generates the fallback wrapper types that are essential
/// for error handling in the Mingling pipeline:
///
/// - **`ErrorRendererNotFound`** — Wraps a `String` (the name of the missing renderer).
///   Used when no matching renderer is found for a given output type.
/// - **`ErrorDispatcherNotFound`** — Wraps `Vec<String>` (the unrecognized command args).
///   Used when no matching dispatcher is found for user input.
/// - **`ResultEmpty`** — Wraps `()` (the unit type).
///   Used when the chain returns an empty result.
///
/// Users can (and should) write `#[renderer]` functions for these types
/// to provide meaningful error messages.
///
/// This macro is called automatically by `gen_program!` and should not
/// be called directly by user code.
///
/// # Syntax
///
/// ```rust,ignore
/// // Called internally by gen_program!:
/// program_fallback_gen!();
/// ```
///
/// # Generated code equivalent
///
/// ```rust,ignore
/// pack!(ErrorRendererNotFound = String);
/// pack!(ErrorDispatcherNotFound = Vec<String>);
/// pack!(ResultEmpty = ());
/// ```
#[proc_macro]
pub fn program_fallback_gen(_input: TokenStream) -> TokenStream {
    #[cfg(feature = "structural_renderer")]
    let pack_empty = quote! {
        #[derive(::serde::Serialize, ::mingling::StructuralData, ::mingling::Grouped, Default)]
        pub struct ResultEmpty;
    };

    #[cfg(not(feature = "structural_renderer"))]
    let pack_empty = quote! {
        #[derive(::mingling::Grouped, Default)]
        pub struct ResultEmpty;
    };

    let expanded = quote! {
        ::mingling::macros::pack!(ErrorRendererNotFound = String);
        ::mingling::macros::pack!(ErrorDispatcherNotFound = Vec<String>);
        #pack_empty
    };
    TokenStream::from(expanded)
}

/// Internal macro used by `gen_program!` to generate the final program enum
/// and its `ProgramCollect` implementation.
///
/// This is the core code generation macro that:
/// 1. Collects all registered types (from `pack!`, `#[derive(Grouped)]`, etc.) and
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
///
/// # Panics
///
// Feature detection: baked into the proc-macro binary at compile time
#[cfg(feature = "async")]
const ASYNC_ENABLED: bool = true;
#[cfg(not(feature = "async"))]
const ASYNC_ENABLED: bool = false;

/// Parses an entry of the format `StructName => EnumVariant,` into a pair of idents.
fn parse_entry_pair(entry: &proc_macro2::TokenStream) -> (proc_macro2::Ident, proc_macro2::Ident) {
    let s = entry.to_string();
    let arrow_idx = s
        .find("=>")
        .unwrap_or_else(|| panic!("Entry missing '=>': {s}"));
    let struct_str = s[..arrow_idx].trim();
    let variant_str = s[arrow_idx + 2..].trim().trim_end_matches(',').trim();
    let struct_ident = proc_macro2::Ident::new(struct_str, proc_macro2::Span::call_site());
    let variant_ident = proc_macro2::Ident::new(variant_str, proc_macro2::Span::call_site());
    (struct_ident, variant_ident)
}

/// Loads the pathf type mapping from `$OUT_DIR/{crate}/type_using.rs`.
/// Always compiled; returns empty map when pathf feature is not enabled.
fn load_pathf_map() -> std::collections::HashMap<String, String> {
    if !cfg!(feature = "pathf") {
        return std::collections::HashMap::new();
    }
    let out_dir = std::env::var("OUT_DIR").ok();
    let crate_name = std::env::var("CARGO_PKG_NAME").ok();
    match (out_dir, crate_name) {
        (Some(dir), Some(name)) => {
            let path = std::path::Path::new(&dir).join(&name).join("type_using.rs");
            match std::fs::read_to_string(&path) {
                Ok(content) => content
                    .lines()
                    .filter_map(|line| {
                        let line = line.trim();
                        if let Some(rest) = line.strip_prefix("use ") {
                            let path = rest.strip_suffix(';').unwrap_or(rest);
                            if let Some((_mod, type_name)) = path.rsplit_once("::") {
                                return Some((type_name.to_string(), path.to_string()));
                            }
                        }
                        None
                    })
                    .collect(),
                Err(_) => std::collections::HashMap::new(),
            }
        }
        _ => std::collections::HashMap::new(),
    }
}

/// Resolves a type name to its full path token stream using the pathf mapping.
pub(crate) fn resolve_type(
    name: &str,
    map: &std::collections::HashMap<String, String>,
) -> proc_macro2::TokenStream {
    if let Some(full_path) = map.get(name) {
        syn::parse_str::<proc_macro2::TokenStream>(full_path).unwrap_or_else(|_| {
            let ident = proc_macro2::Ident::new(name, proc_macro2::Span::call_site());
            quote! { #ident }
        })
    } else {
        let ident = proc_macro2::Ident::new(name, proc_macro2::Span::call_site());
        quote! { #ident }
    }
}

/// Panics if any of the global registries (`PACKED_TYPES`, `RENDERERS`, `CHAINS`, etc.)
/// are poisoned.
#[proc_macro]
#[allow(clippy::too_many_lines)]
pub fn program_final_gen(_input: TokenStream) -> TokenStream {
    let name = syn::Ident::new("ThisProgram", proc_macro2::Span::call_site());

    let packed_types = get_global_set(&PACKED_TYPES).lock().unwrap().clone();

    let renderers = get_global_set(&RENDERERS).lock().unwrap().clone();
    let chains = get_global_set(&CHAINS).lock().unwrap().clone();
    let renderer_exist = get_global_set(&RENDERERS_EXIST).lock().unwrap().clone();
    let chain_exist = get_global_set(&CHAINS_EXIST).lock().unwrap().clone();

    #[cfg(feature = "structural_renderer")]
    let structural_renderers = get_global_set(&STRUCTURAL_RENDERERS)
        .lock()
        .unwrap()
        .clone();

    #[cfg(feature = "comp")]
    let completions = get_global_set(&COMPLETIONS).lock().unwrap().clone();

    let packed_types: Vec<proc_macro2::TokenStream> = packed_types
        .iter()
        .map(|s| syn::parse_str::<proc_macro2::TokenStream>(s).unwrap())
        .collect();

    let renderer_tokens: Vec<proc_macro2::TokenStream> = renderers
        .iter()
        .map(|s| syn::parse_str::<proc_macro2::TokenStream>(s).unwrap())
        .collect();

    let chain_tokens: Vec<proc_macro2::TokenStream> = chains
        .iter()
        .map(|s| syn::parse_str::<proc_macro2::TokenStream>(s).unwrap())
        .collect();

    let renderer_exist_tokens: Vec<proc_macro2::TokenStream> = renderer_exist
        .iter()
        .map(|s| syn::parse_str::<proc_macro2::TokenStream>(s).unwrap())
        .collect();

    let chain_exist_tokens: Vec<proc_macro2::TokenStream> = chain_exist
        .iter()
        .map(|s| syn::parse_str::<proc_macro2::TokenStream>(s).unwrap())
        .collect();

    let pathf_map: std::collections::HashMap<String, String> = if cfg!(feature = "pathf") {
        load_pathf_map()
    } else {
        std::collections::HashMap::new()
    };

    let pathf_uses: Vec<proc_macro2::TokenStream> = if cfg!(feature = "pathf") {
        pathf_map
            .values()
            .map(|path| format!("use {};", path).parse().unwrap_or_default())
            .collect()
    } else {
        Vec::new()
    };

    #[cfg(feature = "structural_renderer")]
    let structural_renderer_tokens: Vec<proc_macro2::TokenStream> = structural_renderers
        .iter()
        .map(|s| syn::parse_str::<proc_macro2::TokenStream>(s).unwrap())
        .collect();

    #[cfg(feature = "structural_renderer")]
    let structural_render = quote! {
        fn structural_render(
            any: ::mingling::AnyOutput<Self::Enum>,
            setting: &::mingling::StructuralRendererSetting,
        ) -> Result<::mingling::RenderResult, ::mingling::error::StructuralRendererSerializeError> {
            #[allow(unused_imports)]
            #(#pathf_uses)*
            match any.member_id {
                #(#structural_renderer_tokens)*
                _ => {
                    // Non-structural types: render ResultEmpty (which implements
                    // StructuralData + Serialize) instead of producing nothing.
                    let mut r = ::mingling::RenderResult::default();
                    ::mingling::StructuralRenderer::render(&ResultEmpty, setting, &mut r)?;
                    Ok(r)
                }
            }
        }
    };

    #[cfg(not(feature = "structural_renderer"))]
    let structural_render = quote! {};

    #[cfg(feature = "dispatch_tree")]
    let compile_time_dispatchers: Vec<String> = get_global_set(&COMPILE_TIME_DISPATCHERS)
        .lock()
        .unwrap()
        .clone()
        .iter()
        .cloned()
        .collect();

    #[cfg(feature = "dispatch_tree")]
    let dispatch_tree_nodes = {
        let entries: Vec<(String, String, String)> = compile_time_dispatchers
            .iter()
            .filter_map(|entry| {
                let parts: Vec<&str> = entry.split(':').collect();
                if parts.len() == 3 {
                    Some((
                        parts[0].to_string(),
                        parts[1].to_string(),
                        parts[2].to_string(),
                    ))
                } else {
                    None
                }
            })
            .collect();

        let get_nodes_fn = dispatch_tree_gen::gen_get_nodes(&entries, &pathf_map);
        let dispatch_trie_fn = dispatch_tree_gen::gen_dispatch_args_trie(&entries, &pathf_map);

        quote! {
            #get_nodes_fn
            #dispatch_trie_fn
        }
    };

    #[cfg(not(feature = "dispatch_tree"))]
    let dispatch_tree_nodes = quote! {};

    #[cfg(feature = "comp")]
    let completion_tokens: Vec<proc_macro2::TokenStream> = completions
        .iter()
        .map(|s| syn::parse_str::<proc_macro2::TokenStream>(s).unwrap())
        .collect();

    #[cfg(feature = "comp")]
    let comp = quote! {
        fn do_comp(any: &::mingling::AnyOutput<Self::Enum>, ctx: &::mingling::ShellContext) -> ::mingling::Suggest {
            #[allow(unused_imports)]
            #(#pathf_uses)*
            match any.member_id {
                #(#completion_tokens)*
                _ => ::mingling::Suggest::FileCompletion,
            }
        }
    };

    #[cfg(not(feature = "comp"))]
    let comp = quote! {};

    // Build render function arms from stored entries
    let render_fn =
        if renderer_tokens.is_empty() {
            quote! {
                fn render(_any: ::mingling::AnyOutput<Self::Enum>) -> ::mingling::RenderResult {
                    ::mingling::RenderResult::default()
                }
            }
        } else {
            let render_arms: Vec<_> = renderer_tokens.iter().map(|entry| {
            let (struct_ident, variant_ident) = parse_entry_pair(entry);
            let downcast_ty = resolve_type(&variant_ident.to_string(), &pathf_map);
            let resolved_struct = resolve_type(&struct_ident.to_string(), &pathf_map);
            quote! {
                Self::#variant_ident => {
                    // SAFETY: The `type_id` check ensures that `any` contains a value of type `#variant_ident`,
                    // so downcasting to `#variant_ident` is safe.
                    let value = unsafe { any.downcast::<#downcast_ty>().unwrap_unchecked() };
                    <#resolved_struct as ::mingling::Renderer>::render(value)
                }
            }
        }).collect();
            quote! {
                fn render(any: ::mingling::AnyOutput<Self::Enum>) -> ::mingling::RenderResult {
                    match any.member_id {
                        #(#render_arms)*
                        _ => ::mingling::RenderResult::default(),
                    }
                }
            }
        };

    // Build do_chain function (async and sync versions)
    let chain_arms_async: Vec<_> = chain_tokens.iter().map(|entry| {
        let (struct_ident, variant_ident) = parse_entry_pair(entry);
        let downcast_ty = resolve_type(&variant_ident.to_string(), &pathf_map);
        let resolved_struct = resolve_type(&struct_ident.to_string(), &pathf_map);
        quote! {
            Self::#variant_ident => {
                // SAFETY: The `type_id` check ensures that `any` contains a value of type `#variant_ident`,
                // so downcasting to `#variant_ident` is safe.
                let value = unsafe { any.downcast::<#downcast_ty>().unwrap_unchecked() };
                let fut = async { <#resolved_struct as ::mingling::Chain<Self::Enum>>::proc(value).await };
                ::std::boxed::Box::pin(fut)
            }
        }
    }).collect();

    let chain_arms_sync: Vec<_> = chain_tokens
        .iter()
        .map(|entry| {
            let (struct_ident, variant_ident) = parse_entry_pair(entry);
            let downcast_ty = resolve_type(&variant_ident.to_string(), &pathf_map);
            let resolved_struct = resolve_type(&struct_ident.to_string(), &pathf_map);
            quote! {
                Self::#variant_ident => {
                    // SAFETY: The `type_id` check ensures that `any` contains a value of type `#variant_ident`,
                    // so downcasting to `#variant_ident` is safe.
                    let value = unsafe { any.downcast::<#downcast_ty>().unwrap_unchecked() };
                    <#resolved_struct as ::mingling::Chain<Self::Enum>>::proc(value)
                }
            }
        })
        .collect();

    let do_chain_fn = if chain_tokens.is_empty() {
        quote! {
            fn do_chain(_any: ::mingling::AnyOutput<Self::Enum>) -> ::mingling::ChainProcess<Self::Enum> {
                ::core::panic!("No chain found for type id")
            }
        }
    } else if ASYNC_ENABLED {
        quote! {
            fn do_chain(
                any: ::mingling::AnyOutput<Self::Enum>,
            ) -> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ::mingling::ChainProcess<Self::Enum>> + ::std::marker::Send>> {
                match any.member_id {
                    #(#chain_arms_async)*
                    _ => ::core::panic!("No chain found for type id: {:?}", any.type_id),
                }
            }
        }
    } else {
        quote! {
            fn do_chain(
                any: ::mingling::AnyOutput<Self::Enum>,
            ) -> ::mingling::ChainProcess<Self::Enum> {
                match any.member_id {
                    #(#chain_arms_sync)*
                    _ => ::core::panic!("No chain found for type id: {:?}", any.type_id),
                }
            }
        }
    };

    let help_tokens: Vec<proc_macro2::TokenStream> = get_global_set(&HELP_REQUESTS)
        .lock()
        .unwrap()
        .clone()
        .iter()
        .map(|s| syn::parse_str::<proc_macro2::TokenStream>(s).unwrap())
        .collect();

    let num_variants = packed_types.len();
    let repr_type = if u8::try_from(num_variants).is_ok() {
        quote! { u8 }
    } else if u16::try_from(num_variants).is_ok() {
        quote! { u16 }
    } else if u32::try_from(num_variants).is_ok() {
        quote! { u32 }
    } else {
        quote! { u128 }
    };

    let expanded = quote! {
        #[derive(Debug, PartialEq, Eq, Clone)]
        #[repr(#repr_type)]
        #[allow(nonstandard_style)]
        pub enum #name {
            #(#packed_types),*
        }

        impl ::std::fmt::Display for #name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                match self {
                    #(#name::#packed_types => write!(f, stringify!(#packed_types)),)*
                }
            }
        }

        impl ::mingling::ProgramCollect for #name {
            type Enum = #name;
            type ErrorDispatcherNotFound = ErrorDispatcherNotFound;
            type ErrorRendererNotFound = ErrorRendererNotFound;
            type ResultEmpty = ResultEmpty;
            fn build_renderer_not_found(member_id: Self::Enum) -> ::mingling::AnyOutput<Self::Enum> {
                ::mingling::AnyOutput::new(ErrorRendererNotFound::new(member_id.to_string()))
            }
            fn build_dispatcher_not_found(args: Vec<String>) -> ::mingling::AnyOutput<Self::Enum> {
                ::mingling::AnyOutput::new(ErrorDispatcherNotFound::new(args))
            }
            fn build_empty_result() -> ::mingling::AnyOutput<Self::Enum> {
                ::mingling::AnyOutput::new(ResultEmpty)
            }
            #render_fn
            #do_chain_fn
            fn render_help(any: ::mingling::AnyOutput<Self::Enum>) -> ::mingling::RenderResult {
                #[allow(unused_imports)]
                #(#pathf_uses)*
                match any.member_id {
                    #(#help_tokens)*
                    _ => ::mingling::RenderResult::default(),
                }
            }
            fn has_renderer(any: &::mingling::AnyOutput<Self::Enum>) -> bool {
                match any.member_id {
                    #(#renderer_exist_tokens)*
                    _ => false
                }
            }
            fn has_chain(any: &::mingling::AnyOutput<Self::Enum>) -> bool {
                match any.member_id {
                    #(#chain_exist_tokens)*
                    _ => false
                }
            }
            #dispatch_tree_nodes
            #structural_render
            #comp
        }

        impl #name {
            /// Creates a new `Program<#name>` instance with default configuration.
            pub fn new() -> ::mingling::Program<#name> {
                ::mingling::Program::new()
            }

            /// Returns a static reference to the global `Program<#name>` singleton.
            pub fn this() -> &'static ::mingling::Program<#name> {
                &::mingling::this::<#name>()
            }
        }
    };

    // Clear all global registries to prevent stale state in Rust Analyzer
    get_global_set(&PACKED_TYPES).lock().unwrap().clear();
    get_global_set(&CHAINS).lock().unwrap().clear();
    get_global_set(&CHAINS_EXIST).lock().unwrap().clear();
    get_global_set(&RENDERERS).lock().unwrap().clear();
    get_global_set(&RENDERERS_EXIST).lock().unwrap().clear();
    get_global_set(&HELP_REQUESTS).lock().unwrap().clear();
    #[cfg(feature = "comp")]
    get_global_set(&COMPLETIONS).lock().unwrap().clear();
    #[cfg(feature = "dispatch_tree")]
    get_global_set(&COMPILE_TIME_DISPATCHERS)
        .lock()
        .unwrap()
        .clear();
    #[cfg(feature = "structural_renderer")]
    get_global_set(&STRUCTURAL_RENDERERS)
        .lock()
        .unwrap()
        .clear();

    TokenStream::from(expanded)
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
    suggest::suggest_enum(input)
}
