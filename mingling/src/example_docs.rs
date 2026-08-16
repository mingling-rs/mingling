// Auto generated

/// Example Argument Picker
///
///  > Demonstrates how to use Mingling's `picker` feature and `Picker` to extract typed arguments from the command line.
///
///  Run:
///  ```bash
///  cargo run --manifest-path examples/example-argument-picker/Cargo.toml --quiet -- calc 1 + 1
///  cargo run --manifest-path examples/example-argument-picker/Cargo.toml --quiet -- calc 7 * 7
///  cargo run --manifest-path examples/example-argument-picker/Cargo.toml --quiet -- calc
///  cargo run --manifest-path examples/example-argument-picker/Cargo.toml --quiet -- calc 1
///  cargo run --manifest-path examples/example-argument-picker/Cargo.toml --quiet -- calc 1 +
///  cargo run --manifest-path examples/example-argument-picker/Cargo.toml --quiet -- calc 4 / 3
///  cargo run --manifest-path examples/example-argument-picker/Cargo.toml --quiet -- calc 4 / 3 --round
///  ```
///
///  Output:
///  ```plaintext
///  Result: 2
///  Result: 49
///  Error: First number (number_a) was not provided.
///  Error: Operator was not provided.
///  Error: Second number (number_b) was not provided.
///  Result: 1.3333334
///  Result: 1
///  ```
///
/// Source code (./Cargo.toml)
/// ```toml
/// [package]
/// name = "example-argument-picker"
/// version = "0.1.0"
/// edition = "2024"
///
/// [dependencies.mingling]
/// path = "../../mingling"
///
/// # Enable `picker` features
/// features = ["picker", "extras"]
///
/// [workspace]
/// ```
///
/// Source code (./src/main.rs)
/// ```ignore
/// use mingling::{
///     consts::REMAINS,
///     macros::route,
///     picker::{
///         IntoPicker, PickerArgResult, SinglePickable,
///         parselib::{ParserStyle, UNIX_STYLE},
///         value::Flag,
///     },
///     prelude::*,
/// };
///
/// // --------- IMPORTANT ---------
/// // Use picker::BasicProgramSetup instead of the original BasicProgramSetup
/// // It uses arg-picker to rewrite the logic of the original BasicProgramSetup
/// use mingling::setup::picker::BasicProgramSetup;
///
/// // --------- IMPORTANT ---------
///
/// dispatcher!("calc", EntryCalculate);
///
/// #[derive(Grouped, Default)]
/// pub struct ErrorNumberANotProvided;
///
/// #[derive(Grouped, Default)]
/// pub struct ErrorNumberBNotProvided;
///
/// #[derive(Grouped, Default)]
/// pub struct ErrorNumberOperatorNotProvided;
///
/// #[derive(Grouped, Default)]
/// pub struct ErrorDivisionByZero;
///
/// #[derive(Grouped, Wrap)]
/// pub struct StateAdd((f32, f32));
///
/// #[derive(Grouped, Wrap)]
/// pub struct StateSubtract((f32, f32));
///
/// #[derive(Grouped, Wrap)]
/// pub struct StateMultiply((f32, f32));
///
/// #[derive(Grouped, Wrap)]
/// pub struct StateDivide((f32, f32));
///
/// #[derive(Grouped, Wrap)]
/// pub struct ResultNumber(f32);
///
/// #[derive(Grouped)]
/// struct StateCalculate {
///     number_a: f32,
///     operator: Operator,
///     number_b: f32,
/// }
///
/// #[derive(Debug, PartialEq, Eq)]
/// enum Operator {
///     Plus,
///     Dash,
///     Slash,
///     Star,
/// }
///
/// // --------- IMPORTANT ---------
/// // Define SinglePickable for type Operator
/// // This allows the type to be picked as an argument
/// impl SinglePickable for Operator {
///     fn pick_single(str: Option<&str>) -> PickerArgResult<Self> {
///         let Some(str) = str else {
///             return PickerArgResult::NotFound;
///         };
///         let op = match str.chars().next() {
///             Some('+') => Operator::Plus,
///             Some('-') => Operator::Dash,
///             Some('*') => Operator::Star,
///             Some('/') => Operator::Slash,
///             _ => return PickerArgResult::NotFound,
///         };
///         PickerArgResult::Parsed(op)
///     }
/// }
/// // --------- IMPORTANT ---------
///
/// #[derive(Default, Clone)]
/// struct ResNumberDisplaySetting {
///     round: bool,
/// }
///
/// fn main() {
///     let mut program = ThisProgram::new();
///
///     // Use ParserStyle to manage the arg-picker theme
///     ParserStyle::set_global_style(&UNIX_STYLE);
///
///     // Enable picker::BasicProgramSetup
///     program.with_setup(BasicProgramSetup);
///
///     // --------- IMPORTANT ---------
///     // Pre-process global arguments before executing commands
///     let (round, args) = program
///         .take_args()
///         //     Use arg![round: Flag] to indicate the `--round` | `-R` flag
///         //     |
///         //     vvvvvvvvvvvvvvvv
///         .pick(&arg![round: Flag, 'R'])
///         //    Use REMAINS to extract remaining arguments
///         //    |
///         //    vvvvvvvv
///         .pick(&REMAINS)
///         // Since Flag and REMAINS will not fail to parse,
///         //   we can safely unwrap here
///         .unwrap();
///     program.replace_args(args.into());
///
///     program.with_resource(ResNumberDisplaySetting { round: *round });
///     // --------- IMPORTANT ---------
///
///     program.exec_and_exit();
/// }
///
/// #[chain]
/// fn handle_calc(args: EntryCalculate) -> Next {
///     // --------- IMPORTANT ---------
///     let (number_a, operator, number_b) = route!(
///         //                 Use the arg! macro to define a positional argument of type f32
///         //                 |
///         //                 vvvvvvvvvv
///         args.pick_or_route(&arg![f32], || ErrorNumberANotProvided.to_chain())
///             .pick_or_route(&arg![Operator], || {
///                 ErrorNumberOperatorNotProvided.to_chain()
///             }) //                         Returns a routable type when not found or fails to parse
///             //                            |
///             //                            vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv
///             .pick_or_route(&arg![f32], || ErrorNumberBNotProvided.to_chain())
///             // Use `to_result` to parse arguments
///             //   and convert to Result<(Tuple, ...), Route> type
///             .to_result()
///     );
///     // --------- IMPORTANT ---------
///
///     if operator == Operator::Slash && number_b == 0. {
///         return ErrorDivisionByZero.to_chain();
///     }
///
///     StateCalculate {
///         number_a,
///         operator,
///         number_b,
///     }
///     .to_chain()
/// }
///
/// #[chain]
/// fn handle_state_calculate(state: StateCalculate) -> Next {
///     match (state.operator, state.number_a, state.number_b) {
///         (Operator::Plus, a, b) => StateAdd((a, b)).to_chain(),
///         (Operator::Dash, a, b) => StateSubtract((a, b)).to_chain(),
///         (Operator::Slash, a, b) => StateDivide((a, b)).to_chain(),
///         (Operator::Star, a, b) => StateMultiply((a, b)).to_chain(),
///     }
/// }
///
/// #[chain]
/// fn handle_state_add(state_add: StateAdd) -> ResultNumber {
///     let (a, b) = state_add.0;
///     ResultNumber(a + b)
/// }
///
/// #[chain]
/// fn handle_state_subtract(state_subtract: StateSubtract) -> ResultNumber {
///     let (a, b) = state_subtract.0;
///     ResultNumber(a - b)
/// }
///
/// #[chain]
/// fn handle_state_multiply(state_multiply: StateMultiply) -> ResultNumber {
///     let (a, b) = state_multiply.0;
///     ResultNumber(a * b)
/// }
///
/// #[chain]
/// fn handle_state_divide(state_divide: StateDivide) -> ResultNumber {
///     let (a, b) = state_divide.0;
///     ResultNumber(a / b)
/// }
///
/// #[renderer]
/// fn render_result_number(result: ResultNumber, setting: &ResNumberDisplaySetting) -> String {
///     let round = setting.round;
///     let result = if round { result.round() } else { result.0 };
///     format!("Result: {}", result)
/// }
///
/// #[renderer]
/// fn render_error_division_by_zero(_: ErrorDivisionByZero) -> String {
///     "Error: Division by zero is not allowed!".to_string()
/// }
///
/// #[renderer]
/// fn render_error_number_a_not_provided(_: ErrorNumberANotProvided) -> String {
///     "Error: First number (number_a) was not provided.".to_string()
/// }
///
/// #[renderer]
/// fn render_error_number_b_not_provided(_: ErrorNumberBNotProvided) -> String {
///     "Error: Second number (number_b) was not provided.".to_string()
/// }
///
/// #[renderer]
/// fn render_error_number_operator_not_provided(_: ErrorNumberOperatorNotProvided) -> String {
///     "Error: Operator was not provided.".to_string()
/// }
///
/// gen_program!();
/// ```
pub mod example_argument_picker {}
/// Example Async Runtime Support
///
///  > This example shows how to drive an async runtime using the `async` feature
///
///  ## Note
///
///  When the `async` feature is enabled, **Mingling** provides a different framework implementation,
///  allowing you to use the `async` keyword directly within `#[chain]`.
///
///  However, you will lose some capabilities:
///
///  1. The program will not be able to use panic unwind functionality
///
///  Run:
///  ```bash
///  cargo run --manifest-path examples/example-async-support/Cargo.toml --quiet -- download README.md
///  ```
///
///  Output:
///  ```plaintext
///  Download begin
///  # (Will pause for 1 second here)
///  "README.md" downloaded.
///  ```
///
/// Source code (./Cargo.toml)
/// ```toml
/// [package]
/// name = "example-async-support"
/// version = "0.1.0"
/// edition = "2024"
///
/// [dependencies.mingling]
/// path = "../../mingling"
///
/// # Enable `picker` features
/// features = ["async", "picker"]
///
/// # Import any async runtime, e.g. Tokio
/// [dependencies.tokio]
/// version = "1.52.3"
/// features = ["macros", "rt", "rt-multi-thread", "time"]
///
/// [workspace]
/// ```
///
/// Source code (./src/main.rs)
/// ```ignore
/// use mingling::{hook::ProgramHook, prelude::*};
/// use std::io::Write;
///
/// #[tokio::main]
/// async fn main() {
///     let mut program = ThisProgram::new();
///
///     // Add a hook to display when the download begins
///     program.with_hook(ProgramHook::empty().on_begin::<_, ()>(|_| println!("Download begin")));
///
///     // --------- IMPORTANT ---------
///     // The return values of `exec_*()` related functions have been replaced with Futures
///     program.exec_and_exit().await;
///     // --------- IMPORTANT ---------
/// }
///
/// dispatcher!("download", EntryDownload);
///
/// #[derive(Grouped, Wrap)]
/// pub struct ResultDownloaded(String);
///
/// // --------- IMPORTANT ---------
/// #[chain]
/// //  vvvvv_ `async` keyword can be used directly here
/// pub async fn handle_download(args: EntryDownload) -> Next {
///     let file_name = args.pick_or_default(&arg![String]).unwrap();
///     fake_download(file_name).await.into()
/// }
///
/// /// Renders the downloaded file name.
/// #[renderer]
/// // But renderers cannot use the `async` keyword
/// pub fn render_downloaded(result: ResultDownloaded) -> RenderResult {
///     let mut render_result = RenderResult::new();
///     writeln!(render_result, "\"{}\" downloaded.", *result).ok();
///     render_result
/// }
/// // --------- IMPORTANT ---------
///
/// gen_program!();
///
/// async fn fake_download(file_name: String) -> ResultDownloaded {
///     tokio::time::sleep(std::time::Duration::from_secs(1)).await;
///     ResultDownloaded(file_name)
/// }
/// ```
pub mod example_async_support {}
/// Example The Basic Usage of Mingling
///
///  Run:
///  ```base
///  cargo run --manifest-path examples/example-basic/Cargo.toml --quiet -- greet
///  cargo run --manifest-path examples/example-basic/Cargo.toml --quiet -- greet Alice
///  ```
///
///  Output:
///  ```plaintext
///  Hello, World!
///  Hello, Alice!
///  ```
///
/// Source code (./Cargo.toml)
/// ```toml
/// [package]
/// name = "example-basic"
/// version = "0.1.0"
/// edition = "2024"
///
/// [dependencies]
/// mingling = { path = "../../mingling" }
///
/// [workspace]
/// ```
///
/// Source code (./src/main.rs)
/// ```ignore
/// // Import commonly used Mingling modules
/// use mingling::prelude::*;
/// use std::io::Write;
///
/// // Define the `greet` subcommand
/// //            _________________ subcmd name, can be nested (e.g. "remote.add" "remote.rm")
/// //           /
/// //           |        _________ entry, records raw arguments
/// //           |       /                         ^^^^^^^^^^^^^
/// //           vvvvv   vvvvvvvvvv                \_ a newtype wrapper around Vec<String>
/// dispatcher!("greet", EntryGreet);
///
/// fn main() {
///     // Create a new ThisProgram
///     let program = ThisProgram::new();
///
///     // Run the program, then exit the process
///     program.exec_and_exit();
/// }
///
/// // Quickly wrap a type into a type recognizable by the current program
/// //     ___________________  Registers this type into ThisProgram
/// //    /            _______  Adds DerefMut, Deref, Into, From wrappers
/// //    |           /
/// //    vvvvvvvvvv  vvvvv
/// #[derive(Grouped, Wrap)]
/// pub struct ResultName(String);
///
/// // Define the `handle_greet` chain for parsing input text
/// //                     ____________________ Previous type:
/// //                    /                       Mingling deduces types at runtime and routes them to this function
/// //                    |               _____ will be expanded to:
/// //                    |              /        ChainProcess<ThisProgram>
/// #[chain] //           vvvvvvvvvv     vvvv
/// fn handle_greet(args: EntryGreet) -> Next {
///     let name: ResultName = args
///         .0
///         .first()
///         .cloned()
///         .unwrap_or_else(|| "World".to_string())
///         .into();
///     name.into()
/// }
///
/// // Define renderer `render_name`, used to render `ResultName`
/// /// Renders the greeting message with the provided name.
/// #[renderer]
/// fn render_name(name: ResultName) -> RenderResult {
///     let mut render_result = RenderResult::new();
///     writeln!(render_result, "Hello, {}!", *name).ok();
///     render_result
/// }
///
/// // Note: This macro generates the program entry point.
/// // It must be placed at the end of the root module of the crate (>= mingling@0.1.8).
/// //                          ^^^^^^     ^^^^^^^^^^^
/// // For example: lib.rs, main.rs
/// gen_program!();
/// ```
pub mod example_basic {}
/// Example Clap Binding
///
///  > This example demonstrates how to bind clap_derive to Mingling
///
///  **Note**:
///  If the `error` parameter of the `dispatcher_clap!` macro is enabled, arguments will be parsed using `try_parse_from`.
///  If you need such output to support ANSI colors, enable the `color` feature of `clap`.
///
///  Run:
///  ```bash
///  cargo run --manifest-path examples/example-clap-binding/Cargo.toml --quiet -- greet
///  cargo run --manifest-path examples/example-clap-binding/Cargo.toml --quiet -- greet Alice
///  cargo run --manifest-path examples/example-clap-binding/Cargo.toml --quiet -- greet Alice -r 5
///  cargo run --manifest-path examples/example-clap-binding/Cargo.toml --quiet -- greet --help
///  cargo run --manifest-path examples/example-clap-binding/Cargo.toml --quiet -- greet --rppat
///  ```
///
///  Output:
///  ```plaintext
///  Hello, World!
///  Hello, Alice!
///  Hello, Alice, Alice, Alice, Alice, Alice!
///  Usage: example-clap-binding [OPTIONS] [NAME]
///
///  Arguments:
///    [NAME]  [default: World]
///
///  Options:
///    -r, --repeat <REPEAT>  [default: 1]
///    -h, --help             Print help
///
///  error: unexpected argument '--rppat' found
///
///    tip: a similar argument exists: '--repeat'
///
///  Usage: example-clap-binding --repeat <REPEAT> [NAME]
///
///  For more information, try '--help'.
///  ```
///
/// Source code (./Cargo.toml)
/// ```toml
/// [package]
/// name = "example-clap-binding"
/// version = "0.1.0"
/// edition = "2024"
///
/// [dependencies.mingling]
/// path = "../../mingling"
/// # Enable `clap` features
/// features = ["clap"]
///
/// # Import `clap` to your project
/// [dependencies.clap]
/// version = "4.6.1"
/// features = [
///     # Enable `derive` feature to support `clap::Parser`
///     "derive",
///     # Enable `color` feature to support ANSI colors
///     "color",
/// ]
///
/// [workspace]
/// ```
///
/// Source code (./src/main.rs)
/// ```ignore
/// use mingling::{Grouped, macros::dispatcher_clap, prelude::*, setup::BasicProgramSetup};
/// use std::io::Write;
///
/// fn main() {
///     let mut program = ThisProgram::new();
///
///     // --------- IMPORTANT ---------
///     // Introduce BasicProgramSetup to support ["--help", "-h"] options
///     program.with_setup(BasicProgramSetup);
///
///     // Set clap help output mode
///     program.stdout_setting.clap_help_print_behaviour =
///         mingling::config::ClapHelpPrintBehaviour::WriteToRenderResult;
///     //  mingling::config::ClapHelpPrintBehaviour::PrintDirectly
///     //
///     // PrintDirectly:
///     //   Let Clap print help information directly to stdout
///     //
///     // WriteToRenderResult:
///     //   Capture Clap's help information and write to RenderResult
///     // --------- IMPORTANT ---------
///
///     program.exec_and_exit();
/// }
///
/// // Implement Clap Parser, and bind to Dispatcher
/// //        _______________________________ Default trait, provides fallback on parse failure
/// //       /         ______________________ clap::Parser, parsing logic implemented by Clap
/// //       |        /              ________ Implement mingling::Grouped
/// //       |        |             /           to ensure Mingling can recognize the type
/// //       vvvvvvv  vvvvvvvvvvvv  vvvvvvv
/// #[derive(Default, clap::Parser, Grouped)]
/// #[dispatcher_clap(
///     "greet",        // Bind EntryGreet to "greet" command
///     help = true,              // Generate clap help for EntryGreet
///     error = ErrorGreetParsed, // Generate and bind error type for parse failure
/// //  ^^^^^\__ Using `error` intercepts parse failure information into the specified type,
/// //              which is then rendered by the renderer
/// )]
/// pub struct EntryGreet {
///     // Positional argument
///     #[clap(default_value = "World")]
///     name: String,
///
///     // Option argument
///     #[arg(short, long, default_value_t = 1)]
///     repeat: i32,
/// }
///
/// /// Renders the greet output with optional repetition.
/// #[renderer]
/// fn render_greet(greet: EntryGreet) -> RenderResult {
///     let name = greet.name;
///     let count = greet.repeat.max(0) as usize;
///
///     let mut render_result = RenderResult::default();
///     write!(render_result, "Hello, ").ok();
///     for i in 0..count {
///         write!(render_result, "{name}").ok();
///         if i < count - 1 {
///             write!(render_result, ", ").ok();
///         }
///     }
///     writeln!(render_result, "!").ok();
///     render_result
/// }
///
/// /// Renders the error message when greet argument parsing fails.
/// #[renderer]
/// // renderers can return a RenderResult instead of using r_println!
/// pub fn render_greet_parse_failed(err: ErrorGreetParsed) -> RenderResult {
///     let mut render_result = RenderResult::default();
///     writeln!(render_result, "{}", *err).ok();
///     render_result
/// }
///
/// gen_program!();
/// ```
pub mod example_clap_binding {}
/// Example: Combining `pathf` + `dispatch_tree`
///
///  > This example demonstrates how to use `pathf` and `dispatch_tree` together.
///  > Types are defined in a submodule (`sub`), and `gen_program!()` resolves
///  > them automatically via pathf without explicit `use` imports.
///  >
///  > **Important**: `dispatch_tree` must be enabled in BOTH `[dependencies]`
///  > AND `[build-dependencies]` so that pathf's builder can detect
///  > `__internal_dispatcher_*` types needed by the dispatch tree.
///  >
///  > Also requires `extras` for the implicit `dispatcher!("hello")` form.
///
///  Run:
///  ```bash
///  cargo run --manifest-path examples/example-combine-pathf-dispatch-tree/Cargo.toml --quiet -- hello Alice
///  ```
///
///  Output:
///  ```plaintext
///  Hello, Alice!
///  ```
///
/// Source code (./Cargo.toml)
/// ```toml
/// [package]
/// name = "example-combine-pathf-dispatch-tree"
/// version = "0.1.0"
/// edition = "2024"
///
/// [dependencies]
/// mingling = { path = "../../mingling", features = [
///     "dispatch_tree",
///     "extras",
///     "pathf",
/// ] }
///
/// [build-dependencies]
/// mingling = { path = "../../mingling", features = [
///     "builds",
///
///     # --------- IMPORTANT ---------
///     # To use pathf under dispatch_tree
///     #   **must** enable the `dispatch_tree`
///     #   feature in build dependencies
///     "dispatch_tree",
///     "pathf",
///     # --------- IMPORTANT ---------
/// ] }
///
/// [workspace]
/// ```
///
/// Source code (./src/main.rs)
/// ```ignore
/// mod sub;
///
/// use mingling::macros::gen_program;
///
/// fn main() {
///     ThisProgram::new().exec_and_exit();
/// }
///
/// gen_program!();
/// ```
pub mod example_combine_pathf_dispatch_tree {}
/// Example: Combining pathf + entry metadata
///
///  > Demonstrates combining the `pathf` feature with entry metadata. The metadata
///  > `DataType` (`Description`) and the dispatchers/entries are defined in the `sub`
///  > module. Thanks to `pathf`, `gen_program!()` resolves these types across
///  > modules automatically, so `main` stays minimal.
///
///  Run:
///  ```bash
///  cargo run --manifest-path examples/example-combine-pathf-metadata/Cargo.toml --quiet -- hello Alice
///  cargo run --manifest-path examples/example-combine-pathf-metadata/Cargo.toml --quiet -- hello
///  cargo run --manifest-path examples/example-combine-pathf-metadata/Cargo.toml --quiet -- desc
///  ```
///
///  Output:
///  ```plaintext
///  Hello, Alice!
///  Hello, World!
///  EntryHello desc = okay
///  ```
///
/// Source code (./Cargo.toml)
/// ```toml
/// [package]
/// name = "example-combine-pathf-metadata"
/// version = "0.1.0"
/// edition = "2024"
///
/// [dependencies.mingling]
/// path = "../../mingling"
/// features = [
///     # `extras` is required by the implicit `dispatcher!("hello")` form
///     "extras",
///     # `pathf` resolves types across modules at build time
///     "pathf",
/// ]
///
/// [build-dependencies.mingling]
/// path = "../../mingling"
/// features = [
///     # Enable the `build` feature for build-time support
///     "build",
///     # `pathf` must also be enabled in build-dependencies
///     "pathf",
/// ]
///
/// [workspace]
/// ```
///
/// Source code (./src/main.rs)
/// ```ignore
/// mod sub;
///
/// use mingling::prelude::*;
///
/// fn main() {
///     ThisProgram::new().exec_and_exit();
/// }
///
/// gen_program!();
/// ```
pub mod example_combine_pathf_metadata {}
/// Example Command Macro
///
///  > Introduced how to use the `#[command]` macro to generate commands with minimal boilerplate
///
///  Run:
///  ```base
///  cargo run --manifest-path examples/example-command-macro/Cargo.toml --quiet -- hello world
///  cargo run --manifest-path examples/example-command-macro/Cargo.toml --quiet -- greet-someone Alice
///  cargo run --manifest-path examples/example-command-macro/Cargo.toml --quiet -- goodbye
///  ```
///
///  Output:
///  ```plaintext
///  Hello, World
///  Hello, Alice
///  Goodbye!
///  ```
///
/// Source code (./Cargo.toml)
/// ```toml
/// [package]
/// name = "example-command-macro"
/// version = "0.1.0"
/// edition = "2024"
///
/// [dependencies.mingling]
/// path = "../../mingling"
/// features = [
///     # Use `extras` to introduce the `#[command]` macro
///     "extras",
///
///     # Use `picker` to parse arguments
///     "picker",
/// ]
///
/// [workspace]
/// ```
///
/// Source code (./src/main.rs)
/// ```ignore
/// use mingling::{macros::buffer, picker::IntoPicker, prelude::*};
///
/// fn main() {
///     ThisProgram::new().exec_and_exit();
/// }
///
/// #[derive(Grouped, Wrap)]
/// pub struct ResultGreeting(String);
///
/// #[derive(Grouped)]
/// pub struct ResultGoodbye;
///
/// // --------- IMPORTANT ---------
/// // Auto-generates dispatcher!("hello.world", EntryHelloWorld);
/// #[command]
/// fn hello_world() -> ResultGreeting {
///     ResultGreeting("World".to_string())
/// }
///
/// // Auto-generates dispatcher!("hello-world", EntryGreetSomeone);
/// #[command(node = "greet-someone")]
/// fn greet_someone(args: Vec<String>) -> ResultGreeting {
///     let name = args.pick_or(&arg![String], || "World".to_string()).unwrap();
///     ResultGreeting(name)
/// }
///
/// // Auto-generates dispatcher!("goodbye", EntryGoodBye);
/// #[command(entry = EntryGoodBye)]
/// fn goodbye() -> ResultGoodbye {
///     ResultGoodbye
/// }
/// // --------- IMPORTANT ---------
///
/// #[renderer(buffer)]
/// fn render_greeting(result: ResultGreeting) {
///     r_println!("Hello, {}", *result);
/// }
///
/// #[renderer(buffer)]
/// fn render_goodbye(_: ResultGoodbye) {
///     r_println!("Goodbye!");
/// }
///
/// gen_program!();
/// ```
pub mod example_command_macro {}
/// Example Completion
///
///  > This example demonstrates how to use **Mingling** to create fully dynamic command-line completions
///
///  ## About Completion Scripts
///
///  To make your completions work, you need to generate a completion script using Mingling's tools
///
///  1. Enable features
///     You need to enable the `build` and `comp` features for `mingling` in `[build-dependencies]`
///
///  2. Write `build.rs`
///     Write the following in `build.rs`
///
///  ```rust,ignore
///  fn main() {
///      build_scripts();
///  }
///
///  /// Generate completion scripts
///  fn build_scripts() {
///      // `env!("CARGO_PKG_NAME")` equals the crate name, which matches the binary name.
///      // If your binary name differs from the crate name, specify it explicitly.
///      mingling::build::build_comp_scripts(
///          // Your binary name:
///          env!("CARGO_PKG_NAME"),
///      )
///      .unwrap();
///  }
///  ```
///
///  3. Verify
///     Build your project with `cargo build --release`. The completion scripts will be generated in `target/release/`
///
///     Execute the script or have it be automatically sourced by your Shell
///
///  Run:
///  ```bash
///  cargo run --manifest-path examples/example-completion/Cargo.toml --quiet -- greet Alice --repeat 3
///  ```
///
///  Output:
///  ```plaintext
///  Hello, Alice, Alice, Alice!
///  ```
///
/// Source code (./Cargo.toml)
/// ```toml
/// [package]
/// name = "example-completion"
/// version = "0.1.0"
/// edition = "2024"
///
/// [dependencies.mingling]
/// path = "../../mingling"
///
/// features = [
///     # Enable `comp` features
///     "comp",
///     "picker",
/// ]
///
/// [build-dependencies.mingling]
/// path = "../../mingling"
///
/// features = [
///     # Enable `comp` features
///     "comp",
///
///     # If you want to build completion scripts,
///     # enable `build` features
///     "build",
/// ]
///
/// [workspace]
/// ```
///
/// Source code (./src/main.rs)
/// ```ignore
/// use mingling::{ShellContext, Suggest, macros::suggest, prelude::*};
/// use std::io::Write;
///
/// fn main() {
///     let program = ThisProgram::new();
///
///     // TIP: Note that the completion script reads stdout,
///     // so make sure no output is produced before the CMDCompletion is dispatched.
///     program.exec_and_exit();
/// }
///
/// // --------- IMPORTANT ---------
/// //            __________________________________________ Entry point bound to completion behavior
/// //           /                 _________________________ Shell context for obtaining user input state
/// //           |                /                 ________ Suggest, used to return completion results
/// //           vvvvvvvvvv       |                /
/// #[completion(EntryGreet)] //  vvvvvvvvvvvv     vvvvvvv
/// fn complete_greet_entry(ctx: &ShellContext) -> Suggest {
///     // When the previous word is `greet` (the current command being typed)
///     if ctx.previous_word == "greet" {
///         // Return suggestions
///         return suggest! {
///             "Bob": "Likes to pass messages",
///             "Alice": "Likes to receive messages",
///             "Hacker": "YOU",
///             "World"
///         };
///     }
///
///     // When the user is typing `--repeat`
///     if ctx.previous_word == "-r" || ctx.previous_word == "--repeat" {
///         return suggest! {}; // Don't suggest anything
///     }
///
///     // When the user is typing `-`
///     if ctx.current_word.starts_with('-') {
///         // Remove arguments that have already been typed by the user
///         let typed: Vec<&str> = ctx.all_words.iter().map(String::as_str).collect();
///         let mut set = suggest! {
///             "-r": "Number of repetitions",
///             "--repeat": "Number of repetitions",
///         };
///         if let Suggest::Suggest(items) = &mut set {
///             items.retain(|item| !typed.contains(&item.suggest().as_str()));
///         }
///         return set;
///     }
///
///     // Otherwise, suggest nothing
///     suggest!()
///     // // You can also enable file completions using the following code,
///     // // which will invoke the Shell's default behavior
///     // Suggest::file_comp()
/// }
/// // --------- IMPORTANT ---------
///
/// dispatcher!("greet", EntryGreet);
/// #[derive(Grouped, Wrap)]
/// pub struct ResultName((u8, String));
///
/// #[chain]
/// fn handle_greet(args: EntryGreet) -> Next {
///     let result: ResultName = args
///         .pick_or(&arg![repeat: u8, 'r'], || 1)
///         .pick_or(&arg![String], || "World".to_string())
///         .unwrap()
///         .into();
///     result.into()
/// }
///
/// /// Renders the greeting with the result name and repeat count.
/// #[renderer]
/// fn render_name(result: ResultName) -> RenderResult {
///     let (repeat, name) = result.0;
///     let mut render_result = RenderResult::new();
///     let mut parts = Vec::with_capacity(repeat as usize);
///     for _ in 0..repeat {
///         parts.push(name.clone());
///     }
///     writeln!(render_result, "Hello, {}!", parts.join(", ")).ok();
///     render_result
/// }
///
/// gen_program!();
/// ```
pub mod example_completion {}
/// Example Dispatch Tree
///
///  > This example will introduce how to use `dispatch_tree`
///  > to optimize your command line lookup efficiency
///
///  When the number of commands in your project increases, you can enable
///  `dispatch_tree` to switch command matching from a linear scan to a
///  character-level trie.
///
///  Run:
///  ```bash
///  cargo run --manifest-path examples/example-dispatch-tree/Cargo.toml --quiet -- cmd5
///  ```
///
///  Output:
///  ```plaintext
///  It's works!
///  ```
///
/// Source code (./Cargo.toml)
/// ```toml
/// [package]
/// name = "example-dispatch-tree"
/// version = "0.1.0"
/// edition = "2024"
///
/// [dependencies.mingling]
/// path = "../../mingling"
///
/// features = [
///     "dispatch_tree",
/// ]
///
/// [workspace]
/// ```
///
/// Source code (./src/main.rs)
/// ```ignore
/// use mingling::prelude::*;
/// use std::io::Write;
///
/// // --------- IMPORTANT ---------
/// // You have a large number of subcommands
/// dispatcher!("cmd1",         Entry1);
/// dispatcher!("cmd2.sub1",   Entry2Sub1);
/// dispatcher!("cmd2.sub2",   Entry2Sub2);
/// dispatcher!("cmd3.sub1.leaf1", Entry3Sub1Leaf1);
/// dispatcher!("cmd3.sub1.leaf2", Entry3Sub1Leaf2);
/// dispatcher!("cmd3.sub2",   Entry3Sub2);
/// dispatcher!("cmd4.sub1.subsub1.deep", Entry4Deep);
/// dispatcher!("cmd4.sub1.subsub2",      Entry4SubSub2);
/// dispatcher!("cmd5",        Entry5);
/// dispatcher!("cmd5.extra",  Entry5Extra);
/// dispatcher!("nested.a.b.c", EntryA);
/// dispatcher!("nested.a.b.d", EntryB);
/// dispatcher!("nested.a.e",   EntryC);
/// dispatcher!("nested.f",     EntryD);
/// // --------- IMPORTANT ---------
///
/// fn main() {
///     let program = ThisProgram::new();
///     program.exec_and_exit();
/// }
///
/// /// Renders the confirmation message for the `cmd5` command.
/// #[renderer]
/// fn render_cmd5(_: Entry5) -> RenderResult {
///     let mut render_result = RenderResult::new();
///     writeln!(render_result, "It's works!").ok();
///     render_result
/// }
///
/// gen_program!();
/// ```
pub mod example_dispatch_tree {}
/// Example Enum Tag
///
///  > This example demonstrates how to use the `EnumTag` derive macro to tag enum variants with metadata,
///  > which can be used for autocompletion and parsing
///
///  Run:
///  ```bash
///  cargo run --manifest-path examples/example-enum-tag/Cargo.toml --quiet -- lang-select OCaml
///  cargo run --manifest-path examples/example-enum-tag/Cargo.toml --quiet -- lang-select
///  ```
///
///  Output:
///  ```plaintext
///  Selected: OCaml (A representative functional programming language with strong type inference)
///  Selected: Rust (A systems programming language focused on performance, safety, and concurrency)
///  ```
///
/// Source code (./Cargo.toml)
/// ```toml
/// [package]
/// name = "example-enum-tag"
/// version = "0.1.0"
/// edition = "2024"
///
/// [dependencies.mingling]
/// path = "../../mingling"
///
/// features = [
///     "comp",
///     "picker"
/// ]
///
/// [workspace]
/// ```
///
/// Source code (./src/main.rs)
/// ```ignore
/// use mingling::{
///     EnumTag, Grouped, ShellContext, Suggest,
///     macros::suggest_enum,
///     picker::{PickerArgResult, SinglePickable},
///     prelude::*,
/// };
/// use std::io::Write;
///
/// // Define the enum and derive the EnumTag trait
/// //                        ________ adds metadata to the enum, enabling it to:
/// //                       /         1. Be used by the `suggest_enum!(Enum)` macro under the `comp` feature for autocompletion
/// //                       vvvvvvv   2. Implement the `PickableEnum` trait
/// #[derive(Debug, Default, EnumTag, Grouped)]
/// pub enum ProgrammingLanguages {
///     #[enum_desc("An efficient and flexible compiled language widely used for system programming")]
///     C,
///
///     #[enum_rename("C++")]
///     #[enum_desc("A high-performance language extending C with object-oriented features")]
///     CPlusPlus,
///
///     #[enum_rename("C#")]
///     #[enum_desc("Microsoft's object-oriented programming language running on the .NET platform")]
///     Csharp,
///
///     #[enum_desc(
///         "A cross-platform object-oriented language widely used for enterprise application development"
///     )]
///     Java,
///
///     #[enum_desc(
///         "A dynamic scripting language for web development, supporting prototype chain inheritance"
///     )]
///     JavaScript,
///
///     #[enum_desc("A modern statically typed language running on the JVM, concise and safe")]
///     Kotlin,
///
///     #[enum_desc("A representative functional programming language with strong type inference")]
///     OCaml,
///
///     #[enum_desc("A general-purpose programming language with clean syntax, known for readability")]
///     Python,
///
///     #[enum_desc("An object-oriented scripting language, famous for its concise and elegant syntax")]
///     Ruby,
///
///     #[default]
///     #[enum_desc("A systems programming language focused on performance, safety, and concurrency")]
///     Rust,
/// }
///
/// // --------- IMPORTANT ---------
/// // NOTE: Due to the migration from the legacy `parser` to `picker`, the `EnumTag` -> `Picker` path
/// // is not yet complete, so a manual implementation is used for now.
/// // Once that path is complete, `#[derive(EnumTag)]` can automatically implement `SinglePickable`,
/// // replacing this manual implementation.
/// impl SinglePickable for ProgrammingLanguages {
///     fn pick_single(str: Option<&str>) -> PickerArgResult<Self> {
///         let Some(str) = str else {
///             return PickerArgResult::NotFound;
///         };
///         let lang = match str.to_lowercase().as_str() {
///             "c" => Self::C,
///             "c++" | "cpp" => Self::CPlusPlus,
///             "c#" | "csharp" => Self::Csharp,
///             "java" => Self::Java,
///             "javascript" | "js" => Self::JavaScript,
///             "kotlin" => Self::Kotlin,
///             "ocaml" => Self::OCaml,
///             "python" => Self::Python,
///             "ruby" => Self::Ruby,
///             "rust" => Self::Rust,
///             _ => return PickerArgResult::NotFound,
///         };
///         PickerArgResult::Parsed(lang)
///     }
/// }
/// // --------- IMPORTANT ---------
///
/// dispatcher!("lang-select", EntryLanguageSelection);
///
/// #[chain]
/// fn handle_language_selection(args: EntryLanguageSelection) -> Next {
///     // You can use Picker to directly parse ProgrammingLanguages
///     let lang: ProgrammingLanguages = args.pick_or_default(&arg![ProgrammingLanguages]).unwrap();
///     lang.into()
/// }
///
/// /// Renders the selected programming language with its name and description.
/// #[renderer]
/// pub fn render_programming_language(lang: ProgrammingLanguages) -> RenderResult {
///     let mut render_result = RenderResult::new();
///     let (name, desc) = lang.enum_info();
///     writeln!(render_result, "Selected: {} ({})", name, desc).ok();
///     render_result
/// }
///
/// #[completion(EntryLanguageSelection)]
/// fn complete_language_selection(_: &ShellContext) -> Suggest {
///     // Use `suggest_enum!` directly to generate enum suggestions
///     suggest_enum!(ProgrammingLanguages)
/// }
///
/// gen_program!();
///
/// fn main() {
///     ThisProgram::new().exec_and_exit();
/// }
/// ```
pub mod example_enum_tag {}
/// Example Error Handling
///
///  > This example demonstrates how to handle errors in Mingling, including custom error types and error rendering.
///
///  Run:
///  ```bash
///  cargo run --manifest-path examples/example-error-handling/Cargo.toml --quiet -- hallo
///  cargo run --manifest-path examples/example-error-handling/Cargo.toml --quiet -- hello
///  cargo run --manifest-path examples/example-error-handling/Cargo.toml --quiet -- hello Alice
///  cargo run --manifest-path examples/example-error-handling/Cargo.toml --quiet -- hello MyBestFriendAlice
///  cargo run --manifest-path examples/example-error-handling/Cargo.toml --quiet -- hello Peter
///  ```
///
///  Output:
///  ```plaintext
///  Command not found: "hallo"
///  No name provided
///  Name not available
///  Name too long: 17 > 10
///  Hello, Peter
///  ```
///
/// Source code (./Cargo.toml)
/// ```toml
/// [package]
/// name = "example-error-handling"
/// version = "0.1.0"
/// edition = "2024"
///
/// [dependencies]
/// mingling = { path = "../../mingling" }
///
/// [workspace]
/// ```
///
/// Source code (./src/main.rs)
/// ```ignore
/// use mingling::prelude::*;
/// use std::io::Write;
///
/// // In Mingling, instead of using ? to propagate errors upward,
/// // errors are treated as branches that continue execution.
///
/// dispatcher!("hello", EntryHello);
///
/// // Define error types
/// #[derive(Grouped)]
/// pub struct ErrorNoNameProvided;
///
/// #[derive(Grouped, Wrap)]
/// pub struct ErrorNameTooLong(u16);
///
/// #[derive(Grouped)]
/// pub struct ErrorNameNotAvailable;
///
/// // Define success type
/// #[derive(Grouped, Wrap)]
/// pub struct ResultName(String);
///
/// // Pre-registered names
/// static VEC_REGISTERED_NAMES: &[&str] = &["Alice", "Bob", "Charlie", "David", "Eve"];
///
/// #[chain]
/// fn handle_hello(args: EntryHello) -> Next {
///     let Some(name) = args.0.first().cloned() else {
///         // If no name is provided, pass ErrorNoNameProvided
///         return ErrorNoNameProvided.to_render();
///     };
///
///     if name.len() > 10 {
///         // If the name is too long, pass ErrorNameTooLong
///         return ErrorNameTooLong(name.len() as u16).to_render();
///     }
///
///     if VEC_REGISTERED_NAMES.contains(&name.as_str()) {
///         // If the name already exists, pass ErrorNameNotAvailable
///         return ErrorNameNotAvailable.to_render();
///     }
///
///     // If the name is valid, pass ResultName
///     ResultName(name).to_render()
/// }
///
/// /// Renders a successful greeting with the given name.
/// #[renderer]
/// fn render_result_name(name: ResultName) -> RenderResult {
///     let mut render_result = RenderResult::new();
///     writeln!(render_result, "Hello, {}", *name).ok();
///     render_result
/// }
///
/// /// Renders the error when no name is provided.
/// #[renderer]
/// fn render_error_no_name_provided(_: ErrorNoNameProvided) -> RenderResult {
///     let mut render_result = RenderResult::new();
///     writeln!(render_result, "No name provided").ok();
///     render_result
/// }
///
/// /// Renders the error when the name is already taken.
/// #[renderer]
/// fn render_error_name_not_available(_: ErrorNameNotAvailable) -> RenderResult {
///     let mut render_result = RenderResult::new();
///     writeln!(render_result, "Name not available").ok();
///     render_result
/// }
///
/// /// Renders the error when the name exceeds the maximum length.
/// #[renderer]
/// fn render_error_name_too_long(len: ErrorNameTooLong) -> RenderResult {
///     let mut render_result = RenderResult::new();
///     writeln!(render_result, "Name too long: {} > 10", *len).ok();
///     render_result
/// }
///
/// /// Renders the error when the dispatcher (subcommand) is not found.
/// #[renderer]
/// fn render_entry_fallback(err: EntryFallback) -> RenderResult {
///     let mut render_result = RenderResult::new();
///     writeln!(render_result, "Command not found: \"{}\"", err.0.join(" ")).ok();
///     render_result
/// }
///
/// gen_program!();
///
/// fn main() {
///     ThisProgram::new().exec_and_exit();
/// }
/// ```
pub mod example_error_handling {}
/// Example Error Handling
///
///  > This example demonstrates how to handle errors in Mingling, including custom error types and error rendering.
///
///  Run:
///  ```bash
///  cargo run --manifest-path examples/example-exitcode/Cargo.toml --quiet -- hello Alice
///  cargo run --manifest-path examples/example-exitcode/Cargo.toml --quiet -- hello
///  ```
///
///  Output:
///  ```plaintext
///  Hello, Alice
///  No name provided (with exit code 1)
///  ```
///
/// Source code (./Cargo.toml)
/// ```toml
/// [package]
/// name = "example-exitcode"
/// version = "0.1.0"
/// edition = "2024"
///
/// [dependencies]
/// mingling = { path = "../../mingling" }
///
/// [workspace]
/// ```
///
/// Source code (./src/main.rs)
/// ```ignore
/// use mingling::{
///     macros::help,
///     prelude::*,
///     res::ResExitCode,
///     setup::{BasicProgramSetup, ExitCodeSetup},
/// };
/// use std::io::Write;
///
/// fn main() {
///     let mut program = ThisProgram::new();
///     program.with_setup(BasicProgramSetup);
///
///     // --------- IMPORTANT ---------
///     // Register `ExitCodeSetup` for the program to enable exit codes
///     program.with_setup(ExitCodeSetup::default());
///     // --------- IMPORTANT ---------
///
///     program.exec_and_exit();
/// }
///
/// dispatcher!("hello", EntryHello);
///
/// #[derive(Grouped)]
/// pub struct ErrorNoNameProvided;
///
/// #[derive(Grouped, Wrap)]
/// pub struct ResultName(String);
///
/// #[chain]
/// fn handle_hello(args: EntryHello) -> Next {
///     let Some(name) = args.0.first().cloned() else {
///         // If no name is provided, pass ErrorNoNameProvided
///         return ErrorNoNameProvided.to_render();
///     };
///
///     // If the name is valid, pass ResultName
///     ResultName(name).to_render()
/// }
///
/// /// Renders a successful greeting with the given name.
/// #[renderer]
/// fn render_result_name(name: ResultName) -> RenderResult {
///     let mut result = RenderResult::new();
///     writeln!(result, "Hello, {}", *name).ok();
///     result
/// }
///
/// #[help]
/// fn help_hello(_p: EntryHello, ec: &mut ResExitCode) -> RenderResult {
///     let mut result = RenderResult::new();
///     writeln!(result, "Usage: hello <NAME>").ok();
///     ec.exit_code = 2;
///     result
/// }
///
/// // Define renderer, render error message                      _______________ Inject exit code resource
/// //                                                           /
/// /// Renders the error when no name is provided               |
/// #[renderer] //                                               vvvvvvvvvvvvvvvv
/// fn render_error_no_name_provided(_: ErrorNoNameProvided, ec: &mut ResExitCode) -> RenderResult {
///     ec.exit_code = 1;
///
///     let mut result = RenderResult::new();
///
///     // Prompt when no name is provided
///     writeln!(result, "No name provided (with exit code 1)").ok();
///     result
/// }
///
/// gen_program!();
/// ```
pub mod example_exitcode {}
/// Example Help
///
///  > This example demonstrates how to use the `#[help]` macro to generate help information,
///  > enabling `--help` to work
///
///  Run
///  ```bash
///  cargo run --manifest-path examples/example-help/Cargo.toml --quiet -- greet --help
///  ```
///
///  Output:
///  ```plain
///  Usage: greet <NAME>
///  ```
///
/// Source code (./Cargo.toml)
/// ```toml
/// [package]
/// name = "example-help"
/// version = "0.1.0"
/// edition = "2024"
///
/// [dependencies]
/// mingling = { path = "../../mingling" }
///
/// [workspace]
/// ```
///
/// Source code (./src/main.rs)
/// ```ignore
/// use mingling::{macros::help, prelude::*, setup::BasicProgramSetup};
/// use std::io::Write;
///
/// dispatcher!("greet", EntryGreet);
///
/// // Define help        _________ When `program.user_context.help` is `true`
/// //                   /            the command will not enter `#[chain]` / `#[renderer]`
/// #[help] //           vvvvvvvvvv   but instead enter this `#[help]` function
/// fn help_greet(_prev: EntryGreet) -> RenderResult {
///     let mut render_result = RenderResult::new();
///     writeln!(render_result, "Usage: greet <NAME>").ok();
///     render_result
/// }
///
/// fn main() {
///     let mut program = ThisProgram::new();
///
///     // --------- IMPORTANT ---------
///     // Add `BasicProgramSetup` to the program
///     // to enable `--help`, `--quiet`, and other built-in features
///     program.with_setup(BasicProgramSetup);
///     // --------- IMPORTANT ---------
///
///     program.exec_and_exit();
/// }
///
/// gen_program!();
/// ```
pub mod example_help {}
/// Example Hook
///
///  > This example demonstrates how to use Mingling's hook system to obtain debugging information during program execution
///
///  Run:
///  ```base
///  cargo run --manifest-path examples/example-hook/Cargo.toml --quiet -- greet Alice
///  ```
///
///  Output:
///  ```plaintext
///  [DEBUG] Program is begin
///  [DEBUG] Pre dispatch: ["greet", "Alice"]
///  [DEBUG] Post dispatch: EntryGreet
///  [DEBUG] Pre chain: EntryGreet
///  [DEBUG] Post chain: ResultName
///  [DEBUG] Pre render: ResultName
///  [DEBUG] Post render
///  [DEBUG] Program end
///  Hello, Alice!
///  ```
///
/// Source code (./Cargo.toml)
/// ```toml
/// [package]
/// name = "example-hook"
/// version = "0.1.0"
/// edition = "2024"
///
/// [dependencies]
/// mingling = { path = "../../mingling" }
///
/// [workspace]
/// ```
///
/// Source code (./src/main.rs)
/// ```ignore
/// use mingling::{
///     hook::{ProgramControlUnit, ProgramHook},
///     prelude::*,
/// };
/// use std::io::Write;
///
/// dispatcher!("greet", EntryGreet);
///
/// fn main() {
///     let mut program = ThisProgram::new();
///
///     // --------- IMPORTANT ---------
///     program.with_hook(
///         ProgramHook::<ThisProgram>::empty()
///             .on_begin::<_, ()>(|_| println!("[DEBUG] Program is begin"))
///             .on_pre_dispatch(|info| println!("[DEBUG] Pre dispatch: {}", info.arguments.join(" ")))
///             .on_post_dispatch(|info| println!("[DEBUG] Post dispatch: {}", info.entry))
///             .on_pre_chain(|info| {
///                 println!("[DEBUG] Pre chain: {}", info.input);
///             })
///             .on_post_chain(|info| println!("[DEBUG] Post chain: {}", info.output.member_id()))
///             .on_finish(|_| {
///                 println!("[DEBUG] Loop end");
///                 ProgramControlUnit::OverrideExitCode(0) // Override exit code
///             })
///             .on_pre_render(|info| println!("[DEBUG] Pre render: {}", info.input))
///             .on_post_render(|_| println!("[DEBUG] Post render")),
///     );
///     // --------- IMPORTANT ---------
///
///     program.exec_and_exit();
/// }
///
/// #[derive(Grouped, Wrap)]
/// pub struct ResultName(String);
///
/// #[chain]
/// fn handle_greet(args: EntryGreet) -> Next {
///     let name: ResultName = args
///         .0
///         .first()
///         .cloned()
///         .unwrap_or_else(|| "World".to_string())
///         .into();
///     name.into()
/// }
///
/// /// Renders the greeting message with the provided name.
/// /// Renders the greeting message with the provided name.
/// #[renderer]
/// fn render_name(name: ResultName) -> RenderResult {
///     let mut render_result = RenderResult::new();
///     writeln!(render_result, "Hello, {}!", *name).ok();
///     render_result
/// }
///
/// gen_program!();
/// ```
pub mod example_hook {}
/// Example Implicit Dispatcher
///
///  > This example demonstrates how to use the implicit `dispatcher!` definition syntax enabled by `extras`
///
/// Source code (./Cargo.toml)
/// ```toml
/// [package]
/// name = "example-implicit-dispatcher"
/// version = "0.1.0"
/// edition = "2024"
///
/// [dependencies.mingling]
/// path = "../../mingling"
/// features = ["extras"]
///
/// [workspace]
/// ```
///
/// Source code (./src/main.rs)
/// ```ignore
/// use mingling::prelude::*;
///
/// // When using implicit syntax, the entry name will be automatically derived
/// // from the command name (the dispatcher struct is generated internally)
/// dispatcher!("remote.add" /* => EntryRemoteAdd */);
/// dispatcher!("remote.remove", EntryRemoteRemove);
///
/// fn main() {
///     ThisProgram::new().exec_and_exit();
/// }
///
/// gen_program!();
/// ```
pub mod example_implicit_dispatcher {}
/// Example Lazy Resources
///
///  > This example demonstrates how to use `LazyRes` for lazy resource initialization.
///
///  Run:
///  ```bash
///  cargo run --manifest-path examples/example-lazy-resources/Cargo.toml --quiet none
///
///  cargo run --manifest-path examples/example-lazy-resources/Cargo.toml --quiet show
///  ```
///
///  Output:
///  ```plaintext
///  None
///
///  Initialized
///  foo: bar
///  rust: lang
///  baz: qux
///  hello: world
///  key: value
///  ```
///
/// Source code (./Cargo.toml)
/// ```toml
/// [package]
/// name = "example-lazy-resources"
/// version = "0.1.0"
/// edition = "2024"
///
/// [dependencies.mingling]
/// path = "../../mingling"
/// features = []
///
/// [workspace]
/// ```
///
/// Source code (./src/main.rs)
/// ```ignore
/// use std::collections::BTreeMap;
/// use std::io::Write;
///
/// use mingling::{LazyInit, LazyRes, prelude::*};
///
/// type Key = String;
/// type Value = String;
///
/// // Define a resource that requires time-consuming initialization
/// #[derive(Default, Clone)]
/// pub struct ResLargeData {
///     pub data: BTreeMap<Key, Value>,
/// }
///
/// fn init_res_large_data() -> ResLargeData {
///     // Perform time-consuming initialization here
///     let mut data = BTreeMap::new();
///     data.insert("foo".to_string(), "bar".to_string());
///     data.insert("baz".to_string(), "qux".to_string());
///     data.insert("hello".to_string(), "world".to_string());
///     data.insert("rust".to_string(), "lang".to_string());
///     data.insert("key".to_string(), "value".to_string());
///
///     // Print to indicate initialization is complete
///     println!("Initialized");
///     ResLargeData { data }
/// }
///
/// dispatcher!("show", EntryShow);
/// dispatcher!("none", EntryNone);
///
/// #[derive(Grouped, Wrap)]
/// pub struct ResultShow(BTreeMap<Key, Value>);
///
/// fn main() {
///     let mut program = ThisProgram::new();
///
///     // --------- IMPORTANT ---------
///     //                     _ Use lazy_init to create LazyRes<ResLargeData>
///     //                    /
///     //                    vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv
///     program.with_resource(ResLargeData::lazy_init(init_res_large_data));
///     // --------- IMPORTANT ---------
///
///     program.exec_and_exit();
/// }
///
/// // Inject LazyRes instead of a normal resource
/// //                                           __________________________ Must use &mut because `get_ref` and `get_mut`
/// //                                          /                             both require mutable borrow
/// //                                          |     _____________________ Use LazyRes<ResLargeData>
/// //                                          |    /                        instead of ResLargeData
/// #[renderer] //                              vvvv vvvvvvvvvvvvvvvvvvvvv
/// fn render_entry_show(_args: EntryShow, res: &mut LazyRes<ResLargeData>) -> RenderResult {
///     let mut render_result = RenderResult::new();
///
///     //             _______ Initialization happens here
///     //            /
///     //            vvvvvvv
///     let res = res.get_ref();
///     for (key, value) in &res.data {
///         writeln!(render_result, "{}: {}", key, value).ok();
///     }
///     render_result
/// }
///
/// // When not using LazyRes<ResLargeData>, it will not be initialized
/// #[renderer]
/// fn render_entry_none(_args: EntryNone) -> RenderResult {
///     let mut render_result = RenderResult::new();
///     writeln!(render_result, "None").ok();
///     render_result
/// }
///
/// gen_program!();
/// ```
pub mod example_lazy_resources {}
/// Example: Entry Metadata (no `pathf`)
///
///  > Demonstrates attaching arbitrary, compile-time-typed metadata (`Description`)
///  > to an entry via `#[metadata(Entry)]`, and retrieving it at runtime through
///  > `ProgramCollect::get_metadata`. The `desc` and `nodoc` subcommands dispatch
///  > through the normal chain/render pipeline — exactly like `example-basic`.
///
///  Run:
///  ```bash
///  cargo run --manifest-path examples/example-metadata/Cargo.toml --quiet -- greet Alice
///  cargo run --manifest-path examples/example-metadata/Cargo.toml --quiet -- greet
///  cargo run --manifest-path examples/example-metadata/Cargo.toml --quiet -- desc
///  cargo run --manifest-path examples/example-metadata/Cargo.toml --quiet -- nodoc
///  ```
///
///  Output:
///  ```plaintext
///  Hello, Alice!
///  Hello, World!
///  EntryGreet desc = ok
///  EntryDescription has no description
///  ```
///
/// Source code (./Cargo.toml)
/// ```toml
/// [package]
/// name = "example-metadata"
/// version = "0.1.0"
/// edition = "2024"
///
/// [dependencies]
/// mingling = { path = "../../mingling" }
///
/// [workspace]
/// ```
///
/// Source code (./src/main.rs)
/// ```ignore
/// use mingling::{macros::metadata, prelude::*};
/// use std::io::Write;
///
/// // Define the `greet` subcommand
/// dispatcher!("greet", EntryGreet);
///
/// // Define the `desc` subcommand, which queries metadata bound to EntryGreet
/// dispatcher!("desc", EntryDescription);
///
/// // Define the `nodoc` subcommand, which queries metadata for an entry that has none
/// dispatcher!("nodoc", EntryNoDescription);
///
/// fn main() {
///     ThisProgram::new().exec_and_exit();
/// }
///
/// /// The metadata type attached to an entry.
/// #[derive(Debug, PartialEq, Eq)]
/// pub struct Description {
///     pub desc: String,
/// }
///
/// // --------- IMPORTANT ---------
/// /// Attach a `Description` to `EntryGreet`.
/// ///
/// /// - `BindType` = `EntryGreet` (the enum variant / entry type)
/// /// - `DataType` = `Description` (the function's return type)
/// #[metadata(EntryGreet)]
/// pub fn greet_desc() -> Description {
///     Description {
///         desc: "ok".to_string(),
///     }
/// }
/// // --------- IMPORTANT ---------
///
/// #[derive(Grouped, Wrap)]
/// pub struct ResultName(String);
///
/// #[derive(Grouped, Wrap)]
/// pub struct DescResult(String);
///
/// /// Chain for `greet` — reads the name and produces a `ResultName`.
/// #[chain]
/// fn handle_greet(args: EntryGreet) -> Next {
///     let name: ResultName = args
///         .0
///         .first()
///         .cloned()
///         .unwrap_or_else(|| "World".to_string())
///         .into();
///     name.into()
/// }
///
/// /// Chain for `desc` — looks up the metadata bound to `EntryGreet`.
/// #[chain]
/// fn handle_desc(_args: EntryDescription) -> Next {
///     use mingling::ProgramCollect;
///     // --------- IMPORTANT ---------
///     let msg = match ThisProgram::get_metadata::<Description>(ThisProgram::EntryGreet) {
///         Some(d) => format!("EntryGreet desc = {}", d.desc),
///         None => "EntryGreet has no description".to_string(),
///     };
///     // --------- IMPORTANT ---------
///     DescResult(msg).to_render()
/// }
///
/// /// Chain for `nodoc` — asks for metadata on an entry that has none.
/// #[chain]
/// fn handle_nodoc(_args: EntryNoDescription) -> Next {
///     use mingling::ProgramCollect;
///     // --------- IMPORTANT ---------
///     let msg = match ThisProgram::get_metadata::<Description>(ThisProgram::EntryDescription) {
///         Some(d) => format!("EntryDescription desc = {}", d.desc),
///         None => "EntryDescription has no description".to_string(),
///     };
///     // --------- IMPORTANT ---------
///     DescResult(msg).to_render()
/// }
///
/// /// Renders the greeting message with the provided name.
/// #[renderer]
/// fn render_name(name: ResultName) -> RenderResult {
///     let mut render_result = RenderResult::new();
///     writeln!(render_result, "Hello, {}!", *name).ok();
///     render_result
/// }
///
/// /// Renders the metadata query result.
/// #[renderer]
/// fn render_desc(msg: DescResult) -> RenderResult {
///     let mut render_result = RenderResult::new();
///     writeln!(render_result, "{}", *msg).ok();
///     render_result
/// }
///
/// gen_program!();
/// ```
pub mod example_metadata {}
/// Example: Using the `group!()` Macro to Register Outside Types
///
///  This example demonstrates how to use the `group!()` macro to make outside
///  types (from `std` or other crates) recognizable by the Mingling framework,
///  without modifying the original type definition.
///
///  Run:
///  ```bash
///  cargo run --manifest-path examples/example-outside-type/Cargo.toml --quiet -- parse 42
///  cargo run --manifest-path examples/example-outside-type/Cargo.toml --quiet -- parse hello
///  cargo run --manifest-path examples/example-outside-type/Cargo.toml --quiet -- error
///  ```
///
///  Output:
///  ```plaintext
///  Parsed number: 42
///  Parse error: invalid digit found in string
///  IO_ERROR: Error
///  ```
///
/// Source code (./Cargo.toml)
/// ```toml
/// [package]
/// name = "example-outside-type"
/// version = "0.1.0"
/// edition = "2024"
///
/// [dependencies.mingling]
/// path = "../../mingling"
/// features = [
///     "extras",
/// ]
///
/// [workspace]
/// ```
///
/// Source code (./src/main.rs)
/// ```ignore
/// use mingling::{macros::group, prelude::*};
/// use std::io::Write;
/// use std::{io::ErrorKind::Other, num::ParseIntError};
///
/// dispatcher!("parse");
/// dispatcher!("error");
///
/// #[chain]
/// fn handle_entry_error(_args: EntryError) -> Next {
///     std::io::Error::new(Other, "Error").to_render()
/// }
///
/// // --------- IMPORTANT ---------
/// // You can directly use the `group!` macro to define outside types as types
/// // recognizable by Mingling
/// //      _____________ from std::num::ParseIntError
/// //     /
/// //     vvvvvvvvvvvvv
/// group!(ParseIntError);
/// group!(ErrorIo = std::io::Error);
/// //     ^^^^^^^^^^^^^^^^^^^^^^^^
/// //     \_____________ For types whose names may cause ambiguity,
/// //                      you can use this syntax to create an alias simultaneously
/// // --------- IMPORTANT ---------
///
/// #[derive(Grouped, Wrap)]
/// pub struct ParsedNumber(i32);
///
/// /// Parse the first argument as an `i32`
/// ///
/// /// On success, routes to `render_number`.
/// /// On failure, routes to `render_parse_error` via the registered outside type.
/// #[chain]
/// fn parse_number(args: EntryParse) -> Next {
///     let input = args.0.first().cloned().unwrap_or_default();
///     match input.parse::<i32>() {
///         Ok(num) => ParsedNumber(num).to_chain(),
///         Err(e) => e.to_chain(),
///     }
/// }
///
/// /// Renderer for successful parse — displays the parsed integer.
/// //                     _____________ Using std::num::ParseIntError as a chain input
/// //                    /
/// #[renderer] //        vvvvvvvvvvvv
/// fn render_number(num: ParsedNumber) -> RenderResult {
///     let mut render_result = RenderResult::new();
///     write!(render_result, "Parsed number: {}", *num).ok();
///     render_result
/// }
///
/// /// Renderer for parse errors — using the outside `ParseIntError` type.
/// ///
/// /// The `ParseIntError` type is registered via `group!` above, so it implements
/// /// `Grouped<ThisProgram>` and can be used directly in a `#[renderer]` function.
/// #[renderer]
/// fn render_parse_error(err: ParseIntError) -> RenderResult {
///     let mut render_result = RenderResult::new();
///     write!(render_result, "Parse error: {}", err).ok();
///     render_result
/// }
///
/// /// Renderer for IO errors — using `std::io::Error` registered as `ErrorIo`.
/// //                       ________ Must use alias `ErrorIo` here, not bare `std::io::Error`
/// //                      /
/// #[renderer] //          vvvvvvv
/// fn render_error_io(err: ErrorIo) -> RenderResult {
///     let mut render_result = RenderResult::new();
///     write!(render_result, "IO_ERROR: {}", err).ok();
///     render_result
/// }
///
/// fn main() {
///     ThisProgram::new().exec_and_exit();
/// }
///
/// gen_program!();
/// ```
pub mod example_outside_type {}
/// Example Panic Unwind
///
///  > This example introduces how to catch Panic in the Mingling program loop
///
///  Run:
///  ```bash
///  cargo run --manifest-path examples/example-panic-unwind/Cargo.toml --quiet -- panic
///  cargo run --manifest-path examples/example-panic-unwind/Cargo.toml --quiet -- panic OhMyGod
///  ```
///
///  Output:
///  ```plaintext
///  Program not panic
///  Program panic: OhMyGod
///  OhMyGod
///  ```
///
/// Source code (./Cargo.toml)
/// ```toml
/// [package]
/// name = "example-panic-unwind"
/// version = "0.1.0"
/// edition = "2024"
///
/// [dependencies.mingling]
/// path = "../../mingling"
/// features = ["picker"]
///
/// # Enable panic unwinding in release builds
/// [profile.release]
/// panic = "unwind"
///
/// # Enable panic unwinding in dev builds
/// [profile.dev]
/// panic = "unwind"
///
/// [workspace]
/// ```
///
/// Source code (./src/main.rs)
/// ```ignore
/// use mingling::config::PanicSilence;
/// use mingling::{hook::ProgramHook, prelude::*};
/// use std::io::Write;
///
/// dispatcher!("panic", EntryPanic);
///
/// #[derive(Grouped)]
/// pub struct NotPanic;
///
/// fn main() {
///     let mut program = ThisProgram::new();
///
///     // --------- IMPORTANT ---------
///     // Enable silence_panic to suppress automatic Panic output
///     program.stdout_setting.silence_panic = PanicSilence::Silence;
///
///     // Define a hook to output &ProgramPanic when a Panic occurs
///     program.with_hook(
///         ProgramHook::empty()
///             .on_exec_panic::<_, ()>(|info| println!("Program panic: {}", info.panic)),
///     );
///     // --------- IMPORTANT ---------
///
///     let _ = program.exec();
/// }
///
/// #[chain]
/// fn handle_panic(prev: EntryPanic) -> Next {
///     let panic_info = prev.pick_or_default(&arg![Option<String>]).unwrap();
///     match panic_info {
///         Some(s) => {
///             // Panic happens here, will be caught
///             panic!("{}", s)
///         }
///         None => NotPanic.into(),
///     }
/// }
///
/// /// Renders the message when no panic occurs.
/// #[renderer]
/// pub fn render(_: NotPanic) -> RenderResult {
///     let mut render_result = RenderResult::new();
///     writeln!(render_result, "Program not panic").ok();
///     render_result
/// }
///
/// gen_program!();
/// ```
pub mod example_panic_unwind {}
/// Example: Module Pathfinder (pathf)
///
///  > This example demonstrates how to use the `pathf` feature to define types
///  > in submodules without needing explicit `use` in the main module.
///  > All type paths are resolved automatically at build time.
///
///  Run:
///  ```bash
///  cargo run --manifest-path examples/example-pathfinder/Cargo.toml --quiet -- greet
///  cargo run --manifest-path examples/example-pathfinder/Cargo.toml --quiet -- greet Alice
///  ```
///
///  Output:
///  ```plaintext
///  Hello, World!
///  Hello, Alice!
///  ```
///
/// Source code (./Cargo.toml)
/// ```toml
/// [package]
/// name = "example-pathfinder"
/// version = "0.1.0"
/// edition = "2024"
///
/// [dependencies.mingling]
/// path = "../../mingling"
///
/// features = [
///     # Enable `pathf` features
///     "pathf",
/// ]
///
/// [build-dependencies.mingling]
/// path = "../../mingling"
///
/// features = [
///     # Enable `pathf` features
///     "pathf",
///
///     # Enable the `build` feature for build-time support
///     "build",
/// ]
///
/// [workspace]
/// ```
///
/// Source code (./src/main.rs)
/// ```ignore
/// mod sub;
///
/// use mingling::macros::gen_program;
///
/// fn main() {
///     ThisProgram::new().exec_and_exit();
/// }
///
/// gen_program!();
/// ```
pub mod example_pathfinder {}
/// Example REPL Basic
///
///  > This example demonstrates how to develop a REPL program using the `repl` feature
///
///  Run:
///  ```bash
///  cargo run --manifest-path examples/example-repl-basic/Cargo.toml --quiet
///  ```
///
/// Source code (./Cargo.toml)
/// ```toml
/// [package]
/// name = "example-repl-basic"
/// version = "0.1.0"
/// edition = "2024"
///
/// [dependencies.mingling]
/// path = "../../mingling"
/// features = ["repl", "picker", "extras"]
///
/// [dependencies]
/// just_fmt = "0.1.2"
///
/// [workspace]
/// ```
///
/// Source code (./src/main.rs)
/// ```ignore
/// use mingling::{
///     hook::ProgramHook,
///     prelude::*,
///     res::ResREPL,
///     setup::{BasicREPLOutputSetup, BasicREPLPromptSetup, BasicREPLReadlineSetup},
///     this,
/// };
/// use std::io::Write;
/// use std::{env::current_dir, path::PathBuf};
///
/// // Resource to store the current directory
/// #[derive(Clone)]
/// struct ResCurrentDir {
///     dir: PathBuf,
/// }
///
/// impl Default for ResCurrentDir {
///     fn default() -> Self {
///         Self {
///             dir: current_dir().unwrap(),
///         }
///     }
/// }
///
/// fn main() {
///     let mut program = ThisProgram::new();
///
///     // Resource
///     program.with_resource(ResCurrentDir::default());
///
///     // Setups
///     // Enable basic std::io::stdin().read_line(&mut input)
///     program.with_setup(BasicREPLReadlineSetup);
///
///     // Enable basic output, using println! after Renderer finishes drawing
///     program.with_setup(BasicREPLOutputSetup);
///
///     // Enable basic Prompt display, with custom display logic
///     program.with_setup(BasicREPLPromptSetup::func(|| {
///         // Get the ResCurrentDir resource from the program
///         let res = this::<ThisProgram>().res::<ResCurrentDir>().unwrap();
///         let dir_str: String = res.dir.to_string_lossy().into();
///         let prompt = format!(
///             "{}> ",
///             dir_str
///                 .replace(&['/', '\\'][..], ">")
///                 .trim_start_matches('>')
///                 .trim_end_matches('>')
///         );
///         prompt
///     }));
///
///     // Add hooks to handle REPL-related events
///     program.with_hook(ProgramHook::empty().on_repl_begin(|_| {
///         // Print welcome message
///         println!("Welcome!");
///     }));
///
///     // Start the REPL loop
///     program.exec_repl();
/// }
///
/// // Create error route
/// #[derive(Grouped, Wrap)]
/// pub struct ErrorDirectoryNotExist(PathBuf);
///
/// // Create commands: cd ls exit
/// dispatcher!("cd", EntryCd);
/// dispatcher!("ls", EntryLs);
/// dispatcher!("exit", EntryExit);
/// dispatcher!("clear", EntryClear);
///
/// // Define data needed for the cd command's execution phase
/// #[derive(Grouped, Wrap)]
/// pub struct StateChangeDirectory(String);
///
/// // Define data needed for the ls command's rendering phase
/// #[derive(Grouped, Wrap)]
/// pub struct ResultList(Vec<String>);
///
/// // Parse cd command arguments
/// #[chain]
/// fn parse_cd_args(prev: EntryCd) -> Next {
///     let join = prev.pick_or_default(&arg![String]).unwrap();
///     StateChangeDirectory(join).into()
/// }
///
/// // Execute directory change
/// #[chain]
/// fn handle_cd(prev: StateChangeDirectory, current_dir: &mut ResCurrentDir) -> Next {
///     use just_fmt::fmt_path::fmt_path;
///
///     let join = prev.0;
///     let new_dir = fmt_path(current_dir.dir.join(join)).unwrap_or_default();
///
///     // If the path is not found, route to error handling
///     if !new_dir.exists() {
///         return ErrorDirectoryNotExist(new_dir).to_render();
///     }
///
///     current_dir.dir = new_dir;
///     empty_result!()
/// }
///
/// // Get directory contents via the CurrentDir resource
/// #[chain]
/// fn handle_ls(_prev: EntryLs, current_dir: &ResCurrentDir) -> Next {
///     let dir = &current_dir.dir;
///     let entries: Vec<String> = std::fs::read_dir(dir)
///         .into_iter()
///         .flat_map(|rd| rd.filter_map(std::result::Result::ok))
///         .map(|e| {
///             let name = e.file_name().to_string_lossy().to_string();
///             if e.file_type().map(|t| t.is_dir()).unwrap_or(false) {
///                 format!("{name}/")
///             } else {
///                 name
///             }
///         })
///         .collect();
///
///     // Render ResultList
///     ResultList(entries).to_render()
/// }
///
/// /// Render ResultList data
/// #[renderer]
/// fn render_list(list: ResultList) -> RenderResult {
///     let mut render_result = RenderResult::new();
///     for item in list.0 {
///         writeln!(render_result, "{}", item).ok();
///     }
///     render_result
/// }
///
/// // Handle exit command event
/// #[chain]
/// fn handle_exit(
///     _prev: EntryExit,
///     repl: &mut ResREPL, // Import REPL resource, registered in `exec_repl`, usable directly
/// ) {
///     // Set the REPL exit flag; REPL will exit after this loop iteration
///     repl.exit = true;
/// }
///
/// /// Handle clear command event
/// #[chain]
/// fn handle_clear(_prev: EntryClear) {
///     // Clear the terminal screen
///     print!("\x1B[2J\x1B[1;1H");
/// }
///
/// /// Handle path not found event
/// #[renderer]
/// fn render_error_directory_not_exist(err: ErrorDirectoryNotExist) -> RenderResult {
///     let mut render_result = RenderResult::new();
///     writeln!(render_result, "Directory not found: {}", err.0.display()).ok();
///     render_result
/// }
///
/// /// Handle dispatcher not found event
/// /// Renders the error when a command is not found.
/// #[renderer]
/// fn dispatcher_not_found(prev: EntryFallback) -> RenderResult {
///     let mut render_result = RenderResult::new();
///     writeln!(render_result, "Command not found: \"{}\"", prev.join(", ")).ok();
///     render_result
/// }
///
/// gen_program!();
/// ```
pub mod example_repl_basic {}
/// Example Resource Injection
///
///  > This example demonstrates how to read and write the program's global state using Mingling's resource system
///
///  Run:
///  ```bash
///  cargo run --manifest-path examples/example-resources/Cargo.toml --quiet current
///  cargo run --manifest-path examples/example-resources/Cargo.toml --quiet modify-current src
///  ```
///
///  Output:
///  ```plaintext
///  Current directory: /home/alice/mingling
///  Current directory: /home/alice/mingling/src
///  ```
///
/// Source code (./Cargo.toml)
/// ```toml
/// [package]
/// name = "example-resources"
/// version = "0.1.0"
/// edition = "2024"
///
/// [dependencies.mingling]
/// path = "../../mingling"
/// features = ["picker"]
///
/// [workspace]
/// ```
///
/// Source code (./src/main.rs)
/// ```ignore
/// use mingling::prelude::*;
/// use std::io::Write;
/// use std::path::PathBuf;
///
/// // Create resource
/// //        ______________ Resource needs to
/// //       /        /        implement the following two traits
/// //       vvvvvvv  vvvvv
/// #[derive(Default, Clone)]
/// struct ResCurrentDir {
///     current_dir: PathBuf,
/// }
///
/// fn main() {
///     let mut program = ThisProgram::new();
///
///     // --------- IMPORTANT ---------
///     // Use `with_resource` to inject a singleton into the program
///     program.with_resource(ResCurrentDir {
///         current_dir: std::env::current_dir().unwrap(),
///     });
///     // --------- IMPORTANT ---------
///
///     program.exec_and_exit();
/// }
///
/// dispatcher!("current", EntryCurrent);
/// dispatcher!("modify-current", EntryModifyCurrent);
///
/// // Define chain for modifying current directory                  _________________ Injected muttable resource
/// //                                                              /
/// #[chain] //                                                     vvvvvvvvvvvvvvvvvv
/// fn render_modify_current(args: EntryModifyCurrent, current_dir: &mut ResCurrentDir) -> Next {
///     current_dir.current_dir = current_dir
///         .current_dir
///         .join(args.pick_or_default(&arg![String]).unwrap());
///     EntryCurrent::default().into()
/// }
///
/// // Define renderer for output current path       _____________ Injected resource
/// //                                              /
/// /// Renders the current directory path.         |
/// #[renderer] //                                  vvvvvvvvvvvvvv
/// fn render_current(_: EntryCurrent, current_dir: &ResCurrentDir) -> RenderResult {
///     let mut render_result = RenderResult::new();
///     write!(
///         render_result,
///         "Current directory: {}",
///         current_dir.current_dir.display()
///     )
///     .ok();
///     render_result
/// }
///
/// gen_program!();
/// ```
pub mod example_resources {}
/// Example Setup
///
///  > This example demonstrates how to build a custom Setup that encapsulates a
///  > group of related resources and registers them with `with_resource`.
///
/// Source code (./Cargo.toml)
/// ```toml
/// [package]
/// name = "example-setup"
/// version = "0.1.0"
/// edition = "2024"
///
/// [dependencies]
/// mingling = { path = "../../mingling", features = ["extras"] }
///
/// [workspace]
/// ```
///
/// Source code (./src/main.rs)
/// ```ignore
/// use mingling::{Program, macros::program_setup, prelude::*};
/// use std::io::Write;
///
/// // A group of related resources — here, the demo app's identity.
/// // Resource types are plain structs: any `Default + Clone + Send + Sync` type
/// // can be used as a resource, and it is identified by its type.
/// #[derive(Default, Clone)]
/// struct ResAppName {
///     name: String,
/// }
///
/// #[derive(Default, Clone)]
/// struct ResAppVersion {
///     version: String,
/// }
///
/// #[derive(Default, Clone)]
/// struct ResGreetingPrefix {
///     prefix: String,
/// }
///
/// fn main() {
///     let mut program = ThisProgram::new();
///
///     // --------- IMPORTANT ---------
///     // Introduce `CustomSetup` generated by `custom_setup`
///     program.with_setup(CustomSetup);
///     // --------- IMPORTANT ---------
///
///     program.exec_and_exit();
/// }
///
/// // --------- IMPORTANT ---------
/// // Define `CustomSetup` (inferred from `custom_setup`)
/// // Package part of the program construction logic into this type for modular
/// // management — e.g. register a group of related resources here.
/// #[program_setup]
/// fn custom_setup(program: &mut Program<ThisProgram>) {
///     program.with_resource(ResAppName {
///         name: "mingling".to_string(),
///     });
///     program.with_resource(ResAppVersion {
///         version: "0.5.0".to_string(),
///     });
///     program.with_resource(ResGreetingPrefix {
///         prefix: "Hello".to_string(),
///     });
/// }
/// // --------- IMPORTANT ---------
///
/// dispatcher!("greet", EntryGreet);
///
/// #[derive(Grouped, Wrap)]
/// pub struct ResultGreeting(String);
///
/// /// Chain: reads the `ResAppName` and `ResAppVersion` resources.
/// #[chain]
/// fn handle_greet(args: EntryGreet, app: &ResAppName, version: &ResAppVersion) -> Next {
///     let who = args
///         .0
///         .first()
///         .cloned()
///         .unwrap_or_else(|| "World".to_string());
///     let greeting: ResultGreeting = format!("{} from {} v{}", who, app.name, version.version).into();
///     greeting.into()
/// }
///
/// /// Renderer: injects the `ResGreetingPrefix` resource to decorate the output.
/// #[renderer]
/// fn render_greet(greeting: ResultGreeting, prefix: &ResGreetingPrefix) -> RenderResult {
///     let mut render_result = RenderResult::new();
///     writeln!(render_result, "{}, {}!", prefix.prefix, *greeting).ok();
///     render_result
/// }
///
/// gen_program!();
/// ```
pub mod example_setup {}
/// Example structural renderer
///
///  > This example demonstrates how to use the `structural_renderer` feature to render data into structures such as json / yaml
///
///  Run
///  ```bash
///  cargo run --manifest-path examples/example-structural-renderer/Cargo.toml --quiet -- render Bob 22
///  cargo run --manifest-path examples/example-structural-renderer/Cargo.toml --quiet -- render Bob 22 --json
///  cargo run --manifest-path examples/example-structural-renderer/Cargo.toml --quiet -- render Bob 22 --yaml
///  ```
///
///  Output:
///  ```plain
///  Bob is 22 years old
///  {"member_name":"Bob","member_age":22}
///  member_name: Bob
///  member_age: 22
///  ```
///
/// Source code (./Cargo.toml)
/// ```toml
/// [package]
/// name = "example-structural-renderer"
/// version = "0.1.0"
/// edition = "2024"
///
/// [dependencies]
/// serde = { version = "1.0.228", features = ["derive"] }
///
/// [dependencies.mingling]
/// path = "../../mingling"
/// features = [
///     "structural_renderer",
///     "yaml_serde_fmt",
///     "picker",
/// ]
///
/// [workspace]
/// ```
///
/// Source code (./src/main.rs)
/// ```ignore
/// use mingling::setup::picker::StructuralRendererSetup;
/// use mingling::{Grouped, StructuralData, prelude::*};
/// use serde::Serialize;
/// use std::io::Write;
///
/// dispatcher!("render", EntryRender);
///
/// fn main() {
///     let mut program = ThisProgram::new();
///     // Add `StructuralRendererSetup` to receive user input `--json` `--yaml` parameters
///     program.with_setup(StructuralRendererSetup);
///     let _ = program.exec();
/// }
///
/// // --------- IMPORTANT ---------
/// // For beautiful output structure, do not wrap the types that need to be output
/// // in a newtype; instead, use a named struct.
/// // Instead, manually implement
/// //        ____________________________________ Mark as structured data so it can be rendered
/// //       /                ____________________ Implement serde::Serialize
/// //       |               /           _________ Implement mingling::Grouped
/// //       |               |          /            to ensure Mingling can recognize the type
/// //       vvvvvvvvvvvv    vvvvvvvvv  vvvvvvv
/// #[derive(StructuralData, Serialize, Grouped)]
/// struct Info {
///     #[serde(rename = "member_name")]
///     name: String,
///     #[serde(rename = "member_age")]
///     age: i32,
/// }
/// // This will output: {"member_name":"name","member_age":32} structure
///
/// // If wrapping with a tuple newtype (e.g. `#[derive(Grouped, Wrap)] pub struct Info((String, i32));`)
/// // Output: {"inner":["name", 32]}
///
/// // --------- IMPORTANT ---------
///
/// #[chain]
/// fn parse_render(prev: EntryRender) -> Next {
///     let (name, age) = prev
///         .pick_or_default(&arg![String])
///         .pick_or_default(&arg![i32])
///         .unwrap();
///     Info { name, age }.to_render()
/// }
///
/// /// Implement default renderer for when structural_renderer is not specified
/// #[renderer]
/// fn render_info(prev: Info) -> RenderResult {
///     let mut render_result = RenderResult::new();
///     writeln!(render_result, "{} is {} years old", prev.name, prev.age).ok();
///     render_result
/// }
///
/// gen_program!();
/// ```
pub mod example_structural_renderer {}
/// Example Unit Test
///
///  > This example shows how to write unit tests for Chain and Renderer in Mingling
///
///  ```bash
///  cargo test --manifest-path examples/example-unit-test/Cargo.toml
///  ```
///
/// Source code (./Cargo.toml)
/// ```toml
/// [package]
/// name = "example-unit-test"
/// version = "0.1.0"
/// edition = "2024"
///
/// [dependencies]
/// mingling = { path = "../../mingling", features = ["extras"] }
///
/// [workspace]
/// ```
///
/// Source code (./src/main.rs)
/// ```ignore
/// use mingling::prelude::*;
/// use std::io::Write;
///
/// #[cfg(test)]
/// mod tests {
///     use super::*;
///     use mingling::macros::entry;
///     use mingling::{assert_member_id, assert_render_result, unpack_chain_process};
///
///     // --------- IMPORTANT ---------
///     #[test]
///     fn test_handle_hello() {
///         let hello_without_args = handle_hello(entry!()).into();
///         assert_render_result!(hello_without_args);
///         assert_member_id!(hello_without_args, ThisProgram::ErrorNoNameProvided);
///
///         let hello_with_registered_name = handle_hello(entry!("Alice")).into();
///         assert_render_result!(hello_with_registered_name);
///         assert_member_id!(
///             hello_with_registered_name,
///             ThisProgram::ErrorNameNotAvailable
///         );
///
///         let hello_with_long_name = handle_hello(entry!("It's a VeryLongName")).into();
///         assert_render_result!(hello_with_long_name);
///         assert_member_id!(hello_with_long_name, ThisProgram::ErrorNameTooLong);
///
///         let hello_with_valid_name = handle_hello(entry!("Peter")).into();
///         assert_render_result!(hello_with_valid_name);
///         let result_name = unpack_chain_process!(hello_with_valid_name, ResultName);
///         assert_eq!(result_name.0, "Peter");
///     }
///
///     #[test]
///     fn test_render_result_name() {
///         let r = render_result_name(ResultName("Peter".into()));
///         assert_eq!(r.to_string().as_str(), "Hello, Peter!")
///     }
///
///     #[test]
///     fn test_render_error_no_name_provided() {
///         let r = render_error_no_name_provided(ErrorNoNameProvided);
///         assert_eq!(r.to_string().as_str(), "No name provided")
///     }
///
///     #[test]
///     fn test_render_error_name_not_available() {
///         let r = render_error_name_not_available(ErrorNameNotAvailable);
///         assert_eq!(r.to_string().as_str(), "Name not available")
///     }
///
///     #[test]
///     fn test_render_error_name_too_long() {
///         let r = render_error_name_too_long(ErrorNameTooLong(17));
///         assert_eq!(r.to_string().as_str(), "Name too long: 17 > 10")
///     }
///     // --------- IMPORTANT ---------
/// }
///
/// dispatcher!("hello", EntryHello);
///
/// #[derive(Grouped)]
/// pub struct ErrorNoNameProvided;
///
/// #[derive(Grouped, Wrap)]
/// pub struct ErrorNameTooLong(u16);
///
/// #[derive(Grouped)]
/// pub struct ErrorNameNotAvailable;
///
/// #[derive(Grouped, Wrap)]
/// pub struct ResultName(String);
///
/// static VEC_REGISTERED_NAMES: &[&str] = &["Alice", "Bob", "Charlie", "David", "Eve"];
///
/// #[chain]
/// fn handle_hello(args: EntryHello) -> Next {
///     let Some(name) = args.0.first().cloned() else {
///         return ErrorNoNameProvided.to_render();
///     };
///
///     if name.len() > 10 {
///         return ErrorNameTooLong(name.len() as u16).to_render();
///     }
///
///     if VEC_REGISTERED_NAMES.contains(&name.as_str()) {
///         return ErrorNameNotAvailable.to_render();
///     }
///
///     ResultName(name).to_render()
/// }
///
/// /// Renders a successful greeting with the given name.
/// #[renderer]
/// fn render_result_name(name: ResultName) -> RenderResult {
///     let mut render_result = RenderResult::new();
///     writeln!(render_result, "Hello, {}!", *name).ok();
///     render_result
/// }
///
/// /// Renders the error when no name is provided.
/// #[renderer]
/// fn render_error_no_name_provided(_: ErrorNoNameProvided) -> RenderResult {
///     let mut render_result = RenderResult::new();
///     writeln!(render_result, "No name provided").ok();
///     render_result
/// }
///
/// /// Renders the error when the name is already taken.
/// #[renderer]
/// fn render_error_name_not_available(_: ErrorNameNotAvailable) -> RenderResult {
///     let mut render_result = RenderResult::new();
///     writeln!(render_result, "Name not available").ok();
///     render_result
/// }
///
/// /// Renders the error when the name exceeds the maximum length.
/// #[renderer]
/// fn render_error_name_too_long(len: ErrorNameTooLong) -> RenderResult {
///     let mut render_result = RenderResult::new();
///     writeln!(render_result, "Name too long: {} > 10", *len).ok();
///     render_result
/// }
///
/// /// Renders the error when the dispatcher (subcommand) is not found.
/// #[renderer]
/// fn render_entry_fallback(err: EntryFallback) -> RenderResult {
///     let mut render_result = RenderResult::new();
///     writeln!(render_result, "Command not found: \"{}\"", err.0.join(" ")).ok();
///     render_result
/// }
///
/// gen_program!();
///
/// fn main() {
///     ThisProgram::new().exec_and_exit();
/// }
/// ```
pub mod example_unit_test {}
