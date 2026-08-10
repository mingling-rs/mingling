// Auto generated

/// Example Argument Parse
///
///  > This example demonstrates how to use the `parser` feature to parse user input
///
///  Run:
///  ```bash
///  cargo run --manifest-path examples/example-argument-parse/Cargo.toml --quiet -- transfer README.md --size 32kib
///  cargo run --manifest-path examples/example-argument-parse/Cargo.toml --quiet -- transfer src/ --dir
///  cargo run --manifest-path examples/example-argument-parse/Cargo.toml --quiet -- strict-transfer README.md
///  cargo run --manifest-path examples/example-argument-parse/Cargo.toml --quiet -- strict-transfer --dir
///  ```
///
///  Output:
///  ```plaintext
///  file: README.md (32768)
///  dir: src/ (1048576)
///  file: README.md (1048576)
///  Error: name is not provided
///  ```
///
/// Source code (./Cargo.toml)
/// ```toml
/// [package]
/// name = "example-argument-parse"
/// version = "0.1.0"
/// edition = "2024"
///
/// [dependencies.mingling]
/// path = "../../mingling"
///
/// # Enable `parser` features
/// features = ["parser", "extras"]
///
/// [workspace]
/// ```
///
/// Source code (./src/main.rs)
/// ```ignore
/// use mingling::{macros::route, prelude::*};
/// use std::io::Write;
///
/// dispatcher!("transfer", CMDTransfer => EntryTransfer);
/// dispatcher!("strict-transfer", CMDStrictTransfer => EntryStrictTransfer);
///
/// pack!(ResultFile = (bool, usize, String)); // (IsDir, Size, Name)
///
/// #[chain]
/// fn handle_transfer_parse(args: EntryTransfer) -> Next {
///     // --------- IMPORTANT ---------
///     // First parse flag arguments (like --dir/-D), then positional arguments
///     let result: ResultFile = args
///         // Name --dir --size 20mib
///         //            ^^^^^^^^^^^^_ first
///         .pick::<bool>(["--dir", "-D"])
///         // Name --dir
///         //      ^^^^^_ second (or `-D`)
///         .pick_or::<usize>("--size", 1024 * 1024_usize)
///         // Name
///         // ^^^^_ finally, pick positional arg
///         .pick::<String>(())
///         .after(|str| str.trim().replace(' ', ""))
///         // Unpack to tuple (is_dir, size, name)
///         .unpack()
///         // Convert into ResultFile
///         .into();
///     // --------- IMPORTANT ---------
///     result.into()
/// }
///
/// pack!(ErrorNoNameProvided = ());
///
/// #[chain]
/// fn handle_strict_transfer_parse(args: EntryStrictTransfer) -> Next {
///     // --------- IMPORTANT ---------
///     // Strict parsing: error immediately if the name is not provided
///     let result: ResultFile = route! { // Use `route!` to wrap a Picker that contains `or_route`
///         args
///             .pick::<bool>(["--dir", "-D"])
///             .pick_or::<usize>("--size", 1024 * 1024_usize)
///             // Finally parse the positional argument; if not found, route to `ErrorNoNameProvided`
///             .pick_or_route::<String, _>((), ErrorNoNameProvided::default())
///             .after(|str| str.trim().replace(' ', ""))
///             .unpack()
///     }
///     // Convert into ResultFile
///     .into();
///     // --------- IMPORTANT ---------
///     result.to_chain()
/// }
///
/// /// Renders the parsed transfer result (file/dir, size, name).
/// #[renderer]
/// fn render_result_file(result: ResultFile) -> RenderResult {
///     let (is_dir, size, name) = result.into();
///     let mut result = RenderResult::new();
///     writeln!(
///         result,
///         "{}: {} ({})",
///         if is_dir { "dir" } else { "file" },
///         name,
///         size
///     )
///     .ok();
///     result
/// }
///
/// /// Renders the error when no name is provided.
/// #[renderer]
/// fn render_error_no_name_provided(_: ErrorNoNameProvided) -> RenderResult {
///     let mut result = RenderResult::new();
///     writeln!(result, "Error: name is not provided").ok();
///     result
/// }
///
/// gen_program!();
///
/// fn main() {
///     let mut program = ThisProgram::new();
///     program.with_dispatcher(CMDTransfer);
///     program.with_dispatcher(CMDStrictTransfer);
///     program.exec_and_exit();
/// }
/// ```
pub mod example_argument_parse {}
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
/// dispatcher!("calc", CMDCalculate => EntryCalculate);
///
/// pack_err!(ErrorNumberANotProvided);
/// pack_err!(ErrorNumberBNotProvided);
/// pack_err!(ErrorNumberOperatorNotProvided);
/// pack_err!(ErrorDivisionByZero);
///
/// pack!(StateAdd = (f32, f32));
/// pack!(StateSubtract = (f32, f32));
/// pack!(StateMultiply = (f32, f32));
/// pack!(StateDivide = (f32, f32));
///
/// pack!(ResultNumber = f32);
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
///     program.with_dispatcher(CMDCalculate);
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
///         args.pick_or_route(&arg![f32], || ErrorNumberANotProvided::default().to_chain())
///             .pick_or_route(&arg![Operator], || {
///                 ErrorNumberOperatorNotProvided::default().to_chain()
///             }) //                         Returns a routable type when not found or fails to parse
///             //                            |
///             //                            vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv
///             .pick_or_route(&arg![f32], || ErrorNumberBNotProvided::default().to_chain())
///             // Use `to_result` to parse arguments
///             //   and convert to Result<(Tuple, ...), Route> type
///             .to_result()
///     );
///     // --------- IMPORTANT ---------
///
///     if operator == Operator::Slash && number_b == 0. {
///         return ErrorDivisionByZero::default().to_chain();
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
///         (Operator::Plus, a, b) => StateAdd::new((a, b)).to_chain(),
///         (Operator::Dash, a, b) => StateSubtract::new((a, b)).to_chain(),
///         (Operator::Slash, a, b) => StateDivide::new((a, b)).to_chain(),
///         (Operator::Star, a, b) => StateMultiply::new((a, b)).to_chain(),
///     }
/// }
///
/// #[chain]
/// fn handle_state_add(state_add: StateAdd) -> ResultNumber {
///     let (a, b) = state_add.inner;
///     ResultNumber::new(a + b)
/// }
///
/// #[chain]
/// fn handle_state_subtract(state_subtract: StateSubtract) -> ResultNumber {
///     let (a, b) = state_subtract.inner;
///     ResultNumber::new(a - b)
/// }
///
/// #[chain]
/// fn handle_state_multiply(state_multiply: StateMultiply) -> ResultNumber {
///     let (a, b) = state_multiply.inner;
///     ResultNumber::new(a * b)
/// }
///
/// #[chain]
/// fn handle_state_divide(state_divide: StateDivide) -> ResultNumber {
///     let (a, b) = state_divide.inner;
///     ResultNumber::new(a / b)
/// }
///
/// #[renderer]
/// fn render_result_number(result: ResultNumber, setting: &ResNumberDisplaySetting) -> String {
///     let round = setting.round;
///     let result = if round { result.round() } else { result.inner };
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
/// # Enable `parser` features
/// features = ["async", "parser"]
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
///     program.with_dispatcher(CMDDownload);
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
/// dispatcher!("download", CMDDownload => EntryDownload);
///
/// pack!(ResultDownloaded = String);
///
/// // --------- IMPORTANT ---------
/// #[chain]
/// //  vvvvv_ `async` keyword can be used directly here
/// pub async fn handle_download(args: EntryDownload) -> Next {
///     let file_name = args.pick(()).unpack();
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
///     ResultDownloaded::new(file_name)
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
/// //            _____________________________ subcmd name, can be nested (e.g. "remote.add" "remote.rm")
/// //           /        _____________________ dispatcher name
/// //           |       /            _________ entry, records raw arguments
/// //           |       |           /                         ^^^^^^^^^^^^^
/// //           vvvvv   vvvvvvvv    vvvvvvvvvv                \_ equivalent to pack!(EntryGreet = Vec<String>)
/// dispatcher!("greet", CMDGreet => EntryGreet);
///
/// fn main() {
///     // Create a new ThisProgram
///     let mut program = ThisProgram::new();
///
///     // Add the CMDGreet dispatcher
///     program.with_dispatcher(CMDGreet);
///
///     // Run the program, then exit the process
///     program.exec_and_exit();
/// }
///
/// // Quickly wrap a type into a type recognizable by the current program
/// //     ____________________ Wrapped type name
/// //    /             _______ Wrapped type inner value
/// //    |            /
/// //    vvvvvvvvvv   vvvvvv
/// pack!(ResultName = String);
///
/// // Define the `handle_greet` chain for parsing input text
/// //                     ____________________ Previous type:
/// //                    /                       Mingling deduces types at runtime and routes them to this function
/// //                    |               _____ will be expanded to:
/// //                    |              /        impl Into<mingling::ChainProcess<ThisProgram>>
/// #[chain] //           vvvvvvvvvv     vvvv
/// fn handle_greet(args: EntryGreet) -> Next {
///     let name: ResultName = args
///         .inner
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
///         mingling::ClapHelpPrintBehaviour::WriteToRenderResult;
///     //  mingling::ClapHelpPrintBehaviour::PrintDirectly
///     //
///     // PrintDirectly:
///     //   Let Clap print help information directly to stdout
///     //
///     // WriteToRenderResult:
///     //   Capture Clap's help information and write to RenderResult
///     // --------- IMPORTANT ---------
///
///     program.with_dispatcher(CMDGreet);
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
///     "greet", CMDGreet,        // Bind EntryGreet to "greet" command
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
///     let mut program = ThisProgram::new();
///     program.with_dispatcher(sub::CMDHello);
///     program.with_dispatcher(sub::CMDDescription);
///     program.exec_and_exit();
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
///     let mut program = ThisProgram::new();
///
///     // Import the dispatchers generated by the `#[command]` macro
///     program.with_dispatcher(CMDHelloWorld);
///     program.with_dispatcher(CMDGreetSomeone);
///     program.with_dispatcher(CMDGoodBye);
///
///     program.exec_and_exit();
/// }
///
/// pack!(ResultGreeting = String);
/// pack!(ResultGoodbye = ());
///
/// // --------- IMPORTANT ---------
/// // Auto-generates dispatcher!("hello.world", CMDHelloWorld => EntryHelloWorld);
/// #[command]
/// fn hello_world() -> ResultGreeting {
///     ResultGreeting::new("World".to_string())
/// }
///
/// // Auto-generates dispatcher!("hello-world", CMDGreetSomeone => EntryGreetSomeone);
/// #[command(node = "greet-someone")]
/// fn greet_someone(args: Vec<String>) -> ResultGreeting {
///     let name = args.pick_or(&arg![String], || "World".to_string()).unwrap();
///     ResultGreeting::new(name)
/// }
///
/// // Auto-generates dispatcher!("goodbye", CMDGoodBye => EntryGoodBye);
/// #[command(name = CMDGoodBye, entry = EntryGoodBye)]
/// fn goodbye() -> ResultGoodbye {
///     ResultGoodbye::default()
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
///     "parser",
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
/// use mingling::{macros::suggest, prelude::*, ShellContext, Suggest};
/// use std::io::Write;
///
/// fn main() {
///     let mut program = ThisProgram::new();
///
///     program.with_dispatcher(CMDGreet);
///
///     // --------- IMPORTANT ---------
///     // The `comp` feature makes `gen_program!()` generate a CMDCompletion automatically
///     // It adds a hidden `__comp` subcommand for communication with the completion script
///     program.with_dispatcher(crate::CMDCompletion);
///     // --------- IMPORTANT ---------
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
///     if ctx.filling_argument(["-r", "--repeat"]) {
///         return suggest! {}; // Don't suggest anything
///     }
///
///     // When the user is typing `-`
///     if ctx.typing_argument() {
///         return suggest! {
///             "-r": "Number of repetitions",
///             "--repeat": "Number of repetitions",
///         }
///         // Remove arguments that have already been typed by the user
///         .strip_typed_argument(ctx);
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
/// dispatcher!("greet", CMDGreet => EntryGreet);
/// pack!(ResultName = (u8, String));
///
/// #[chain]
/// fn handle_greet(args: EntryGreet) -> Next {
///     let result: ResultName = args
///         .pick_or(["-r", "--repeat"], 1)
///         .pick_or((), "World")
///         .unpack()
///         .into();
///     result.into()
/// }
///
/// /// Renders the greeting with the result name and repeat count.
/// #[renderer]
/// fn render_name(result: ResultName) -> RenderResult {
///     let (repeat, name) = result.inner;
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
/// Example Custom Pickable
///
///  > This example demonstrates how to use the Pickable trait to add parsing for your types
///
///  Run:
///  ```bash
///  cargo run --manifest-path examples/example-custom-pickable/Cargo.toml --quiet -- connect 127.0.0.1:5012
///  cargo run --manifest-path examples/example-custom-pickable/Cargo.toml --quiet -- connect 127.0.0.1
///  ```
///
///  Output:
///  ```plaintext
///  Connected to "127.0.0.1:5012"
///  Failed to parse address
///  ```
///
/// Source code (./Cargo.toml)
/// ```toml
/// [package]
/// name = "example-custom-pickable"
/// version = "0.1.0"
/// edition = "2024"
///
/// [dependencies.mingling]
/// path = "../../mingling"
///
/// features = ["parser", "extras"]
///
/// [workspace]
/// ```
///
/// Source code (./src/main.rs)
/// ```ignore
/// use mingling::{macros::route, parser::Pickable, prelude::*, Grouped};
/// use std::io::Write;
///
/// // Define types that can be recognized by Mingling
/// //               ________________________ `Pickable` trait needs to implement Default
/// //              /                ________ The Grouped derive macro registers an ID for this type
/// //              |               /           Mingling uses this ID to identify the type
/// //              vvvvvvv         vvvvvvv
/// #[derive(Debug, Default, Clone, Grouped)]
/// pub struct Address {
///     pub ip: [u8; 4],
///     pub port: u16,
/// }
///
/// // --------- IMPORTANT ---------
/// impl Pickable for Address {
///     type Output = Address;
///     fn pick(args: &mut mingling::parser::Argument, flag: mingling::Flag) -> Option<Self::Output> {
///         // Extract the raw string from Argument using the Flag
///         let raw: String = args.pick_argument(flag)?.clone();
///
///         // Use TryFrom to parse the address
///         Address::try_from(raw).ok()
///     }
/// }
/// // --------- IMPORTANT ---------
///
/// dispatcher!("connect", CMDConnect => EntryConnect);
/// pack!(ErrorParseAddressFailed = ());
///
/// #[chain]
/// fn handle_connect(prev: EntryConnect) -> Next {
///     let connect: Address =
///         route! { prev.pick_or_route((), ErrorParseAddressFailed::default()).unpack() };
///     connect.to_chain()
/// }
///
/// /// Renders the connected address.
/// #[renderer]
/// pub fn render_address(addr: Address) -> RenderResult {
///     let mut render_result = RenderResult::new();
///     write!(render_result, "Connected to \"{}\"", addr).ok();
///     render_result
/// }
///
/// /// Renders the error message when address parsing fails.
/// #[renderer]
/// pub fn render_error_parse_address_failed(_: ErrorParseAddressFailed) -> RenderResult {
///     let mut render_result = RenderResult::new();
///     write!(render_result, "Failed to parse address").ok();
///     render_result
/// }
///
/// gen_program!();
///
/// fn main() {
///     let mut program = ThisProgram::new();
///     program.with_dispatcher(CMDConnect);
///     program.exec_and_exit();
/// }
///
/// // Address conversion
///
/// impl TryFrom<String> for Address {
///     type Error = String;
///
///     fn try_from(raw: String) -> Result<Self, Self::Error> {
///         // Expected format: "192.168.1.1:8080"
///         let parts: Vec<&str> = raw.split(':').collect();
///         if parts.len() != 2 {
///             return Err("Invalid format: expected 'IP:PORT'".to_string());
///         }
///
///         let ip_str = parts[0];
///         let port_str = parts[1];
///
///         // Parse IP address (4 octets separated by dots)
///         let ip_parts: Vec<&str> = ip_str.split('.').collect();
///         if ip_parts.len() != 4 {
///             return Err("Invalid IP address format".to_string());
///         }
///
///         let mut ip = [0u8; 4];
///         for (i, part) in ip_parts.iter().enumerate() {
///             ip[i] = part
///                 .parse::<u8>()
///                 .map_err(|_| format!("Invalid IP octet: {part}"))?;
///         }
///
///         // Parse port
///         let port = port_str
///             .parse::<u16>()
///             .map_err(|_| format!("Invalid port: {port_str}"))?;
///
///         Ok(Address { ip, port })
///     }
/// }
///
/// impl From<Address> for String {
///     fn from(addr: Address) -> String {
///         format!(
///             "{}.{}.{}.{}:{}",
///             addr.ip[0], addr.ip[1], addr.ip[2], addr.ip[3], addr.port
///         )
///     }
/// }
///
/// impl std::fmt::Display for Address {
///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
///         write!(
///             f,
///             "{}.{}.{}.{}:{}",
///             self.ip[0], self.ip[1], self.ip[2], self.ip[3], self.port
///         )
///     }
/// }
/// ```
pub mod example_custom_pickable {}
/// Example Dispatch Tree
///
///  > This example will introduce how to use `dispatch_tree`
///  > to optimize your command line lookup efficiency
///
///  When the number of commands in your project increases, you can use `dispatch_tree` to complete command registration at compile time.
///  It will generate a trie for quickly finding related commands by prefix.
///
///  Therefore, after enabling this feature,
///  `Program` will no longer store a Dispatcher list internally, and the `with_dispatcher` function will not be compiled.
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
/// dispatcher!("cmd1",         CMD1 => Entry1);
/// dispatcher!("cmd2.sub1",   CMD2Sub1 => Entry2Sub1);
/// dispatcher!("cmd2.sub2",   CMD2Sub2 => Entry2Sub2);
/// dispatcher!("cmd3.sub1.leaf1", CMD3Sub1Leaf1 => Entry3Sub1Leaf1);
/// dispatcher!("cmd3.sub1.leaf2", CMD3Sub1Leaf2 => Entry3Sub1Leaf2);
/// dispatcher!("cmd3.sub2",   CMD3Sub2 => Entry3Sub2);
/// dispatcher!("cmd4.sub1.subsub1.deep", CMD4Deep => Entry4Deep);
/// dispatcher!("cmd4.sub1.subsub2",      CMD4SubSub2 => Entry4SubSub2);
/// dispatcher!("cmd5",        CMD5 => Entry5);
/// dispatcher!("cmd5.extra",  CMD5Extra => Entry5Extra);
/// dispatcher!("nested.a.b.c", CMDA => EntryA);
/// dispatcher!("nested.a.b.d", CMDB => EntryB);
/// dispatcher!("nested.a.e",   CMDC => EntryC);
/// dispatcher!("nested.f",     CMDD => EntryD);
/// // --------- IMPORTANT ---------
///
/// fn main() {
///     let program = ThisProgram::new();
///
///     // --------- IMPORTANT ---------
///     // // You no longer need to use `with_dispatcher` anymore;
///     // // it'll be collected automatically once the `dispatch_tree` feature is enabled
///     // program.with_dispatcher(...);
///
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
///     "parser"
/// ]
///
/// [workspace]
/// ```
///
/// Source code (./src/main.rs)
/// ```ignore
/// use mingling::{
///     macros::suggest_enum, parser::PickableEnum, prelude::*, EnumTag, Grouped, ShellContext,
///     Suggest,
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
///     #[enum_desc(
///         "An object-oriented scripting language, famous for its concise and elegant syntax"
///     )]
///     Ruby,
///
///     #[default]
///     #[enum_desc("A systems programming language focused on performance, safety, and concurrency")]
///     Rust,
/// }
///
/// // --------- IMPORTANT ---------
/// // Implement the PickableEnum trait for ProgrammingLanguages,
/// // so that `Picker` can parse this enum
/// impl PickableEnum for ProgrammingLanguages {}
/// // --------- IMPORTANT ---------
///
/// dispatcher!("lang-select", CMDLanguageSelection => EntryLanguageSelection);
///
/// #[chain]
/// fn handle_language_selection(args: EntryLanguageSelection) -> Next {
///     // You can use Picker to directly parse ProgrammingLanguages
///     let lang: ProgrammingLanguages = args.pick(()).unpack();
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
///     let mut program = ThisProgram::new();
///     program.with_dispatcher(CMDCompletion);
///     program.with_dispatcher(CMDLanguageSelection);
///     program.exec_and_exit();
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
/// dispatcher!("hello", CMDHello => EntryHello);
///
/// // Define error types
/// pack!(ErrorNoNameProvided = ());
/// pack!(ErrorNameTooLong = u16);
/// pack!(ErrorNameNotAvailable = ());
///
/// // Define success type
/// pack!(ResultName = String);
///
/// // Pre-registered names
/// static VEC_REGISTERED_NAMES: &[&str] = &["Alice", "Bob", "Charlie", "David", "Eve"];
///
/// #[chain]
/// fn handle_hello(args: EntryHello) -> Next {
///     let Some(name) = args.inner.first().cloned() else {
///         // If no name is provided, pass ErrorNoNameProvided
///         return ErrorNoNameProvided::default().to_render();
///     };
///
///     if name.len() > 10 {
///         // If the name is too long, pass ErrorNameTooLong
///         return ErrorNameTooLong::new(name.len() as u16).to_render();
///     }
///
///     if VEC_REGISTERED_NAMES.contains(&name.as_str()) {
///         // If the name already exists, pass ErrorNameNotAvailable
///         return ErrorNameNotAvailable::default().to_render();
///     }
///
///     // If the name is valid, pass ResultName
///     ResultName::new(name).to_render()
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
///     writeln!(
///         render_result,
///         "Command not found: \"{}\"",
///         err.inner.join(" ")
///     )
///     .ok();
///     render_result
/// }
///
/// gen_program!();
///
/// fn main() {
///     let mut program = ThisProgram::new();
///     program.with_dispatcher(CMDHello);
///     program.exec_and_exit();
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
///     program.with_dispatcher(CMDHello);
///     program.exec_and_exit();
/// }
///
/// dispatcher!("hello", CMDHello => EntryHello);
///
/// pack!(ErrorNoNameProvided = ());
/// pack!(ResultName = String);
///
/// #[chain]
/// fn handle_hello(args: EntryHello) -> Next {
///     let Some(name) = args.inner.first().cloned() else {
///         // If no name is provided, pass ErrorNoNameProvided
///         return ErrorNoNameProvided::default().to_render();
///     };
///
///     // If the name is valid, pass ResultName
///     ResultName::new(name).to_render()
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
/// dispatcher!("greet", CMDGreet => EntryGreet);
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
///     program.with_dispatcher(CMDGreet);
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
/// dispatcher!("greet", CMDGreet => EntryGreet);
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
///     program.with_dispatcher(CMDGreet);
///     program.exec_and_exit();
/// }
///
/// pack!(ResultName = String);
///
/// #[chain]
/// fn handle_greet(args: EntryGreet) -> Next {
///     let name: ResultName = args
///         .inner
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
/// // When using implicit syntax, the entry and dispatcher names will be automatically derived
/// dispatcher!("remote.add" /*, CMDRemoteAdd    => EntryRemoteAdd */);
/// dispatcher!("remote.remove", CMDRemoteRemove => EntryRemoteRemove);
///
/// fn main() {
///     let mut program = ThisProgram::new();
///
///     // --------- IMPORTANT ---------
///     program.with_dispatcher(CMDRemoteAdd);
///     //                      ^^^^^^^^^^^^\_ CMDRemoteAdd is implicitly created
///     // --------- IMPORTANT ---------
///
///     program.with_dispatcher(CMDRemoteRemove);
///     program.exec_and_exit();
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
/// dispatcher!("show", CMDShow => EntryShow);
/// dispatcher!("none", CMDNone => EntryNone);
///
/// pack!(ResultShow = BTreeMap<Key, Value>);
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
///     program.with_dispatchers((CMDShow, CMDNone));
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
/// dispatcher!("greet", CMDGreet => EntryGreet);
///
/// // Define the `desc` subcommand, which queries metadata bound to EntryGreet
/// dispatcher!("desc", CMDDescription => EntryDescription);
///
/// // Define the `nodoc` subcommand, which queries metadata for an entry that has none
/// dispatcher!("nodoc", CMDNoDescription => EntryNoDescription);
///
/// fn main() {
///     let mut program = ThisProgram::new();
///     program.with_dispatcher(CMDGreet);
///     program.with_dispatcher(CMDDescription);
///     program.with_dispatcher(CMDNoDescription);
///     program.exec_and_exit();
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
/// pack!(ResultName = String);
/// pack!(DescResult = String);
///
/// /// Chain for `greet` — reads the name and produces a `ResultName`.
/// #[chain]
/// fn handle_greet(args: EntryGreet) -> Next {
///     let name: ResultName = args
///         .inner
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
///     DescResult::new(msg).to_render()
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
///     DescResult::new(msg).to_render()
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
/// pack!(ParsedNumber = i32);
///
/// /// Parse the first argument as an `i32`
/// ///
/// /// On success, routes to `render_number`.
/// /// On failure, routes to `render_parse_error` via the registered outside type.
/// #[chain]
/// fn parse_number(args: EntryParse) -> Next {
///     let input = args.inner.first().cloned().unwrap_or_default();
///     match input.parse::<i32>() {
///         Ok(num) => ParsedNumber::new(num).to_chain(),
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
///     let mut program = ThisProgram::new();
///     program.with_dispatcher(CMDParse);
///     program.with_dispatcher(CMDError);
///     program.exec_and_exit();
/// }
///
/// gen_program!();
/// ```
pub mod example_outside_type {}
/// Example `pack_err!`
///
///  > This example demonstrates how to use the `pack_err!` macro to define error types
///  > with automatic `name` field (set to snake_case at compile time) and optional `info` field.
///  > Also demonstrates `--json` serialization when `structural_renderer` is enabled.
///
///  Run:
///  ```bash
///  cargo run --manifest-path examples/example-pack-err/Cargo.toml --quiet -- find
///  cargo run --manifest-path examples/example-pack-err/Cargo.toml --quiet -- find Cargo.toml
///  cargo run --manifest-path examples/example-pack-err/Cargo.toml --quiet -- find src
///  cargo run --manifest-path examples/example-pack-err/Cargo.toml --quiet -- find-structural --json
///  cargo run --manifest-path examples/example-pack-err/Cargo.toml --quiet -- find-structural Cargo.toml --json
///  cargo run --manifest-path examples/example-pack-err/Cargo.toml --quiet -- find-structural src --json
///  ```
///
///  Output:
///  ```plaintext
///  Search path not provided
///  Not a directory: Cargo.toml
///  Found directory: src
///  {"name":"error_not_found"}
///  {"name":"error_not_dir","info":"Cargo.toml"}
///  {"inner":"src"}
///  {"name":"error_not_found_structural"}
///  {"name":"error_not_dir_structural","info":"Cargo.toml"}
///  ```
///
/// Source code (./Cargo.toml)
/// ```toml
/// [package]
/// name = "example-pack-err"
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
///     "extras",
/// ]
///
/// [workspace]
/// ```
///
/// Source code (./src/main.rs)
/// ```ignore
/// use mingling::prelude::*;
/// use mingling::setup::StructuralRendererSetup;
/// use std::io::Write;
/// use std::path::PathBuf;
///
/// dispatcher!("find", CMDFind => EntryFind);
/// dispatcher!("find-structural", CMDFindStructural => EntryFindStructural);
///
/// // --------- IMPORTANT ---------
/// // `pack_err!` is a convenient macro for defining error types.
/// //
/// //     Simple form:         pack_err!(ErrorNotFound);
/// //     Typed form:          pack_err!(ErrorNotDir = PathBuf);
/// //
/// // The simple form generates a struct with `name: String` and `impl Default`.
/// //   name = "error_not_found"  (automatically snake_cased at compile time)
/// //
/// // The typed form additionally generates `pub fn new(info)`.
/// //   name = "error_not_dir"
/// //
/// // When `structural_renderer` is enabled, the struct also gets
/// // `#[derive(serde::Serialize)]` for --json / --yaml output.
/// // --------- IMPORTANT ---------
///
/// // Simple form — name = "error_not_found"
/// pack_err!(ErrorNotFound);
///
/// // Typed form — name = "error_not_dir"
/// pack_err!(ErrorNotDir = PathBuf);
///
/// // Simple form — with StructuralData support for --json / --yaml
/// pack_err_structural!(ErrorNotFoundStructural);
///
/// // Typed form — with StructuralData support for --json / --yaml
/// pack_err_structural!(ErrorNotDirStructural = PathBuf);
///
/// // Success type with StructuralData support
/// pack_structural!(ResultPath = PathBuf);
///
/// #[chain]
/// fn handle_find(args: EntryFind) -> Next {
///     let Some(path_str) = args.inner.first().cloned() else {
///         // No path provided → use the simple error form (Default)
///         return ErrorNotFound::default().to_render();
///     };
///
///     let path = PathBuf::from(&path_str);
///     if path.is_dir() {
///         // Is a directory → success
///         ResultPath::new(path).to_render()
///     } else {
///         // Not a directory (or doesn't exist) → use the typed error form
///         ErrorNotDir::new(path).to_render()
///     }
/// }
///
/// #[chain]
/// fn handle_find_structural(args: EntryFindStructural) -> Next {
///     let Some(path_str) = args.inner.first().cloned() else {
///         // No path provided → use the simple error form (Default)
///         return ErrorNotFoundStructural::default().to_render();
///     };
///
///     let path = PathBuf::from(&path_str);
///     if path.is_dir() {
///         // Is a directory → success
///         ResultPath::new(path).to_render()
///     } else {
///         // Not a directory (or doesn't exist) → use the typed error form
///         ErrorNotDirStructural::new(path).to_render()
///     }
/// }
///
/// /// Renders the successful result with the found directory path.
/// #[renderer]
/// fn render_result_path(path: ResultPath) -> RenderResult {
///     let mut render_result = RenderResult::new();
///     writeln!(render_result, "Found directory: {}", path.display()).ok();
///     render_result
/// }
///
/// /// Renders the error when no search path is provided.
/// #[renderer]
/// fn render_error_not_found(_: ErrorNotFound) -> RenderResult {
///     let mut render_result = RenderResult::new();
///     writeln!(render_result, "Search path not provided").ok();
///     render_result
/// }
///
/// /// Renders the error when the given path is not a directory.
/// #[renderer]
/// fn render_error_not_dir(err: ErrorNotDir) -> RenderResult {
///     let mut render_result = RenderResult::new();
///     writeln!(render_result, "Not a directory: {}", err.info.display()).ok();
///     render_result
/// }
///
/// /// Renders the structural error when no search path is provided.
/// #[renderer]
/// fn render_error_not_found_structural(_: ErrorNotFoundStructural) -> RenderResult {
///     let mut render_result = RenderResult::new();
///     writeln!(render_result, "Search path not provided").ok();
///     render_result
/// }
///
/// /// Renders the structural error when the given path is not a directory.
/// #[renderer]
/// fn render_error_not_dir_structural(err: ErrorNotDirStructural) -> RenderResult {
///     let mut render_result = RenderResult::new();
///     writeln!(render_result, "Not a directory: {}", err.info.display()).ok();
///     render_result
/// }
///
/// gen_program!();
///
/// fn main() {
///     let mut program = ThisProgram::new();
///     // Add StructuralRendererSetup to support --json / --yaml flags
///     program.with_setup(StructuralRendererSetup);
///     program.with_dispatcher(CMDFind);
///     program.with_dispatcher(CMDFindStructural);
///     let _ = program.exec();
/// }
/// ```
pub mod example_pack_err {}
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
/// features = ["parser"]
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
/// use mingling::{hook::ProgramHook, prelude::*};
/// use std::io::Write;
///
/// dispatcher!("panic", CMDPanic => EntryPanic);
/// pack!(NotPanic = ());
///
/// fn main() {
///     let mut program = ThisProgram::new();
///     program.with_dispatcher(CMDPanic);
///
///     // --------- IMPORTANT ---------
///     // Enable silence_panic to suppress automatic Panic output
///     program.stdout_setting.silence_panic = true;
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
///     let panic_info = prev.pick::<Option<String>>(()).unpack();
///     match panic_info {
///         Some(s) => {
///             // Panic happens here, will be caught
///             panic!("{}", s)
///         }
///         None => NotPanic::default().into(),
///     }
/// }
///
/// /// Renders the message when no panic occurs.
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
/// use crate::sub::CMDGreet;
///
/// fn main() {
///     let mut program = ThisProgram::new();
///     program.with_dispatcher(CMDGreet);
///     program.exec_and_exit();
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
/// features = ["repl", "parser", "extras"]
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
///     // Dispatchers
///     program.with_dispatcher(CMDCd);
///     program.with_dispatcher(CMDLs);
///     program.with_dispatcher(CMDExit);
///     program.with_dispatcher(CMDClear);
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
/// pack!(ErrorDirectoryNotExist = PathBuf);
///
/// // Create commands: cd ls exit
/// dispatcher!("cd", CMDCd => EntryCd);
/// dispatcher!("ls", CMDLs => EntryLs);
/// dispatcher!("exit", CMDExit => EntryExit);
/// dispatcher!("clear", CMDClear => EntryClear);
///
/// // Define data needed for the cd command's execution phase
/// pack!(StateChangeDirectory = String);
///
/// // Define data needed for the ls command's rendering phase
/// pack!(ResultList = Vec<String>);
///
/// // Parse cd command arguments
/// #[chain]
/// fn parse_cd_args(prev: EntryCd) -> Next {
///     let join = prev.pick(()).unpack();
///     StateChangeDirectory::new(join).into()
/// }
///
/// // Execute directory change
/// #[chain]
/// fn handle_cd(prev: StateChangeDirectory, current_dir: &mut ResCurrentDir) -> Next {
///     use just_fmt::fmt_path::fmt_path;
///
///     let join = prev.inner;
///     let new_dir = fmt_path(current_dir.dir.join(join)).unwrap_or_default();
///
///     // If the path is not found, route to error handling
///     if !new_dir.exists() {
///         return ErrorDirectoryNotExist::new(new_dir).to_render();
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
///     ResultList::new(entries).to_render()
/// }
///
/// /// Render ResultList data
/// #[renderer]
/// fn render_list(list: ResultList) -> RenderResult {
///     let mut render_result = RenderResult::new();
///     for item in list.inner {
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
///     writeln!(
///         render_result,
///         "Directory not found: {}",
///         err.inner.display()
///     )
///     .ok();
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
/// features = ["parser"]
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
///     program.with_dispatchers((CMDCurrent, CMDModifyCurrent));
///     program.exec_and_exit();
/// }
///
/// dispatcher!("current", CMDCurrent => EntryCurrent);
/// dispatcher!("modify-current", CMDModifyCurrent => EntryModifyCurrent);
///
/// // Define chain for modifying current directory                  _________________ Injected muttable resource
/// //                                                              /
/// #[chain] //                                                     vvvvvvvvvvvvvvvvvv
/// fn render_modify_current(args: EntryModifyCurrent, current_dir: &mut ResCurrentDir) -> Next {
///     current_dir.current_dir = current_dir
///         .current_dir
///         .join(args.pick::<String>(()).unpack());
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
///  > This example demonstrates how to build a custom Setup for modular management of project components
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
/// use mingling::{macros::program_setup, prelude::*, Program};
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
/// // Package part of the program construction logic into this type for modular management
/// #[program_setup]
/// fn custom_setup(program: &mut Program<ThisProgram>) {
///     program.with_dispatchers((CMD1, CMD2, CMD3, CMD4, CMD5));
/// }
/// // --------- IMPORTANT ---------
///
/// dispatcher!("1", CMD1 => Entry1);
/// dispatcher!("2", CMD2 => Entry2);
/// dispatcher!("3", CMD3 => Entry3);
/// dispatcher!("4", CMD4 => Entry4);
/// dispatcher!("5", CMD5 => Entry5);
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
///     "parser",
/// ]
///
/// [workspace]
/// ```
///
/// Source code (./src/main.rs)
/// ```ignore
/// use mingling::prelude::*;
/// use mingling::{parser::Picker, setup::StructuralRendererSetup, Grouped, StructuralData};
/// use serde::Serialize;
/// use std::io::Write;
///
/// dispatcher!("render", CMDRender => EntryRender);
///
/// fn main() {
///     let mut program = ThisProgram::new();
///     // Add `StructuralRendererSetup` to receive user input `--json` `--yaml` parameters
///     program.with_setup(StructuralRendererSetup);
///     program.with_dispatcher(CMDRender);
///     let _ = program.exec();
/// }
///
/// // --------- IMPORTANT ---------
/// // For beautiful output structure, do not use `pack!` to wrap the types that need to be output.
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
/// // If using pack!(Info = (String, i32));
/// // Output: {"inner":["name", 32]}
///
/// // --------- IMPORTANT ---------
///
/// #[chain]
/// fn parse_render(prev: EntryRender) -> Next {
///     let (name, age) = Picker::new(prev.inner)
///         .pick::<String>(())
///         .pick::<i32>(())
///         .unpack();
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
///         assert_eq!(result_name.inner, "Peter");
///     }
///
///     #[test]
///     fn test_render_result_name() {
///         let r = render_result_name(ResultName::new("Peter".into()));
///         assert_eq!(r.to_string().as_str(), "Hello, Peter!")
///     }
///
///     #[test]
///     fn test_render_error_no_name_provided() {
///         let r = render_error_no_name_provided(ErrorNoNameProvided::default());
///         assert_eq!(r.to_string().as_str(), "No name provided")
///     }
///
///     #[test]
///     fn test_render_error_name_not_available() {
///         let r = render_error_name_not_available(ErrorNameNotAvailable::default());
///         assert_eq!(r.to_string().as_str(), "Name not available")
///     }
///
///     #[test]
///     fn test_render_error_name_too_long() {
///         let r = render_error_name_too_long(ErrorNameTooLong::new(17));
///         assert_eq!(r.to_string().as_str(), "Name too long: 17 > 10")
///     }
///     // --------- IMPORTANT ---------
/// }
///
/// dispatcher!("hello", CMDHello => EntryHello);
///
/// pack!(ErrorNoNameProvided = ());
/// pack!(ErrorNameTooLong = u16);
/// pack!(ErrorNameNotAvailable = ());
///
/// pack!(ResultName = String);
///
/// static VEC_REGISTERED_NAMES: &[&str] = &["Alice", "Bob", "Charlie", "David", "Eve"];
///
/// #[chain]
/// fn handle_hello(args: EntryHello) -> Next {
///     let Some(name) = args.inner.first().cloned() else {
///         return ErrorNoNameProvided::default().to_render();
///     };
///
///     if name.len() > 10 {
///         return ErrorNameTooLong::new(name.len() as u16).to_render();
///     }
///
///     if VEC_REGISTERED_NAMES.contains(&name.as_str()) {
///         return ErrorNameNotAvailable::default().to_render();
///     }
///
///     ResultName::new(name).to_render()
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
///     writeln!(
///         render_result,
///         "Command not found: \"{}\"",
///         err.inner.join(" ")
///     )
///     .ok();
///     render_result
/// }
///
/// gen_program!();
///
/// fn main() {
///     let mut program = ThisProgram::new();
///     program.with_dispatcher(CMDHello);
///     program.exec_and_exit();
/// }
/// ```
pub mod example_unit_test {}
