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
/// features = ["parser", "extra_macros"]
///
/// [workspace]
/// ```
///
/// Source code (./src/main.rs)
/// ```ignore
/// use mingling::{macros::route, prelude::*};
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
///     result
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
///             .pick_or_route::<String, _>((), ErrorNoNameProvided::default().to_chain())
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
/// fn render_result_file(result: ResultFile) {
///     let (is_dir, size, name) = result.into();
///     r_println!(
///         "{}: {} ({})",
///         if is_dir { "dir" } else { "file" },
///         name,
///         size
///     )
/// }
///
/// /// Renders the error when no name is provided.
/// #[renderer]
/// fn render_error_no_name_provided(_: ErrorNoNameProvided) {
///     r_println!("Error: name is not provided")
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
///  1. `&mut` resource injection is not available in async chain functions
///  2. The program will not be able to use panic unwind functionality
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
///
/// #[tokio::main]
/// async fn main() {
///     let mut program = ThisProgram::new();
///
///     program.with_dispatcher(CMDDownload);
///
///     // Add a hook to display when the download begins
///     program.with_hook(ProgramHook::empty().on_begin(|| println!("Download begin")));
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
///     fake_download(file_name).await
/// }
///
/// /// Renders the downloaded file name.
/// #[renderer]
/// // But renderers cannot use the `async` keyword
/// pub fn render_downloaded(result: ResultDownloaded) {
///     r_println!("\"{}\" downloaded.", *result);
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
///     name
/// }
///
/// // Define renderer `render_name`, used to render `ResultName`
/// /// Renders the greeting message with the provided name.
/// #[renderer]
/// fn render_name(name: ResultName) {
///     r_println!("Hello, {}!", *name);
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
/// use mingling::{macros::dispatcher_clap, prelude::*, setup::BasicProgramSetup, Groupped};
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
/// //       |        /              ________ Implement mingling::Groupped
/// //       |        |             /           to ensure Mingling can recognize the type
/// //       vvvvvvv  vvvvvvvvvvvv  vvvvvvvv
/// #[derive(Default, clap::Parser, Groupped)]
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
/// fn render_greet(greet: EntryGreet) {
///     let name = greet.name;
///     let count = greet.repeat.max(0) as usize;
///
///     r_print!("Hello, ");
///     for i in 0..count {
///         r_print!("{name}");
///         if i < count - 1 {
///             r_print!(", ");
///         }
///     }
///     r_println!("!");
/// }
///
/// /// Renders the error message when greet argument parsing fails.
/// #[renderer]
/// fn render_greet_parse_failed(err: ErrorGreetParsed) {
///     r_println!("{}", *err);
/// }
///
/// gen_program!();
/// ```
pub mod example_clap_binding {}
/// Example Completion
///
///  > This example demonstrates how to use **Mingling** to create fully dynamic command-line completions
///
///  ## About Completion Scripts
///
///  To make your completions work, you need to generate a completion script using Mingling's tools
///
///  1. Enable features
///     You need to enable the `builds` and `comp` features for `mingling` in `[build-dependencies]`
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
///     # enable `builds` features
///     "builds",
/// ]
///
/// [workspace]
/// ```
///
/// Source code (./src/main.rs)
/// ```ignore
/// use mingling::{macros::suggest, prelude::*, ShellContext, Suggest};
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
///     result
/// }
///
/// /// Renders the greeting with the result name and repeat count.
/// #[renderer]
/// fn render_name(result: ResultName) {
///     let (repeat, name) = result.inner;
///     let mut parts = Vec::with_capacity(repeat as usize);
///     for _ in 0..repeat {
///         parts.push(name.clone());
///     }
///     r_println!("Hello, {}!", parts.join(", "));
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
/// features = ["parser", "extra_macros"]
///
/// [workspace]
/// ```
///
/// Source code (./src/main.rs)
/// ```ignore
/// use mingling::{macros::route, parser::Pickable, prelude::*, Groupped};
///
/// // Define types that can be recognized by Mingling
/// //               ________________________ `Pickable` trait needs to implement Default
/// //              /                ________ The Groupped derive macro registers an ID for this type
/// //              |               /           Mingling uses this ID to identify the type
/// //              vvvvvvv         vvvvvvvv
/// #[derive(Debug, Default, Clone, Groupped)]
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
///         route! { prev.pick_or_route((), ErrorParseAddressFailed::default().to_chain()).unpack() };
///     connect.to_chain()
/// }
///
/// /// Renders the connected address.
/// #[renderer]
/// fn render_address(addr: Address) {
///     r_println!("Connected to \"{}\"", addr.to_string());
/// }
///
/// /// Renders the error message when address parsing fails.
/// #[renderer]
/// fn render_error_parse_address_failed(_: ErrorParseAddressFailed) {
///     r_println!("Failed to parse address");
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
/// fn render_cmd5(_: Entry5) {
///     r_println!("It's works!");
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
///     macros::suggest_enum, parser::PickableEnum, prelude::*, EnumTag, Groupped, ShellContext,
///     Suggest,
/// };
///
/// // Define the enum and derive the EnumTag trait
/// //                        ________ adds metadata to the enum, enabling it to:
/// //                       /         1. Be used by the `suggest_enum!(Enum)` macro under the `comp` feature for autocompletion
/// //                       vvvvvvv   2. Implement the `PickableEnum` trait
/// #[derive(Debug, Default, EnumTag, Groupped)]
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
///     lang
/// }
///
/// /// Renders the selected programming language with its name and description.
/// #[renderer]
/// fn render_programming_language(lang: ProgrammingLanguages) {
///     // You can use `enum_info()` to get the name and description of the current enum
///     let (name, desc) = lang.enum_info();
///     r_println!("Selected: {} ({})", name, desc)
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
/// fn render_result_name(name: ResultName) {
///     r_println!("Hello, {}", *name);
/// }
///
/// /// Renders the error when no name is provided.
/// #[renderer]
/// fn render_error_no_name_provided(_: ErrorNoNameProvided) {
///     // Prompt when no name is provided
///     r_println!("No name provided");
/// }
///
/// /// Renders the error when the name is already taken.
/// #[renderer]
/// fn render_error_name_not_available(_: ErrorNameNotAvailable) {
///     // Prompt when name is already taken
///     r_println!("Name not available");
/// }
///
/// /// Renders the error when the name exceeds the maximum length.
/// #[renderer]
/// fn render_error_name_too_long(len: ErrorNameTooLong) {
///     // Prompt when name is too long, showing actual length
///     r_println!("Name too long: {} > 10", *len);
/// }
///
/// /// Renders the error when the dispatcher (subcommand) is not found.
/// #[renderer]
/// fn render_dispatcher_not_found(err: ErrorDispatcherNotFound) {
///     // Prompt when command is not found, showing the input command
///     r_println!("Command not found: \"{}\"", err.inner.join(" "));
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
/// use mingling::{prelude::*, res::ResExitCode, setup::ExitCodeSetup};
///
/// fn main() {
///     let mut program = ThisProgram::new();
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
/// fn render_result_name(name: ResultName) {
///     r_println!("Hello, {}", *name);
/// }
///
/// // Define renderer, render error message                      _______________ Inject exit code resource
/// //                                                           /
/// /// Renders the error when no name is provided               |
/// #[renderer] //                                               vvvvvvvvvvvvvvvv
/// fn render_error_no_name_provided(_: ErrorNoNameProvided, ec: &mut ResExitCode) {
///     ec.exit_code = 1;
///
///     // Prompt when no name is provided
///     r_println!("No name provided (with exit code 1)");
/// }
///
/// gen_program!();
/// ```
pub mod example_exitcode {}
/// Example General Renderer
///
///  > This example demonstrates how to use the `general_renderer` feature to render data into structures such as json / yaml
///
///  Run
///  ```bash
///  cargo run --manifest-path examples/example-general-renderer/Cargo.toml --quiet -- render Bob 22
///  cargo run --manifest-path examples/example-general-renderer/Cargo.toml --quiet -- render Bob 22 --json
///  cargo run --manifest-path examples/example-general-renderer/Cargo.toml --quiet -- render Bob 22 --yaml
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
/// name = "example-general-renderer"
/// version = "0.1.0"
/// edition = "2024"
///
/// [dependencies]
/// serde = { version = "1.0.228", features = ["derive"] }
///
/// [dependencies.mingling]
/// path = "../../mingling"
/// features = [
///     "general_renderer",
///     "parser",
/// ]
///
/// [workspace]
/// ```
///
/// Source code (./src/main.rs)
/// ```ignore
/// use mingling::prelude::*;
/// use mingling::{parser::Picker, setup::GeneralRendererSetup, Groupped};
/// use serde::Serialize;
///
/// dispatcher!("render", CMDRender => EntryRender);
///
/// fn main() {
///     let mut program = ThisProgram::new();
///     // Add `GeneralRendererSetup` to receive user input `--json` `--yaml` parameters
///     program.with_setup(GeneralRendererSetup);
///     program.with_dispatcher(CMDRender);
///     let _ = program.exec();
/// }
///
/// // --------- IMPORTANT ---------
/// // For beautiful output structure, do not use `pack!` to wrap the types that need to be output.
/// // Instead, manually implement
/// //        ____________________ Implement serde::Serialize
/// //       /           _________ Implement mingling::Groupped
/// //       |          /            to ensure Mingling can recognize the type
/// //       vvvvvvvvv  vvvvvvvv
/// #[derive(Serialize, Groupped)]
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
/// /// Implement default renderer for when general_renderer is not specified
/// #[renderer]
/// fn render_info(prev: Info) {
///     r_println!("{} is {} years old", prev.name, prev.age);
/// }
///
/// gen_program!();
/// ```
pub mod example_general_renderer {}
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
///
/// dispatcher!("greet", CMDGreet => EntryGreet);
///
/// // Define help        _________ When `program.user_context.help` is `true`
/// //                   /            the command will not enter `#[chain]` / `#[renderer]`
/// #[help] //           vvvvvvvvvv   but instead enter this `#[help]` function
/// fn help_greet(_prev: EntryGreet) {
///     r_println!("Usage: greet <NAME>");
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
/// use mingling::{hook::ProgramHook, prelude::*};
///
/// dispatcher!("greet", CMDGreet => EntryGreet);
///
/// fn main() {
///     let mut program = ThisProgram::new();
///
///     // --------- IMPORTANT ---------
///     program.with_hook(
///         ProgramHook::<ThisProgram>::empty()
///             .on_begin(|| println!("[DEBUG] Program is begin"))
///             .on_pre_dispatch(|args| println!("[DEBUG] Pre dispatch: {args:?}"))
///             .on_post_dispatch(|c: &_| println!("[DEBUG] Post dispatch: {c:?}"))
///             .on_pre_chain(|c: &_, _| {
///                 println!("[DEBUG] Pre chain: {c}");
///             })
///             .on_post_chain(|any_output| println!("[DEBUG] Post chain: {}", any_output.member_id))
///             .on_finish(|| {
///                 println!("[DEBUG] Loop end");
///                 0 // Override exit code
///             })
///             .on_pre_render(|c: &_, _| println!("[DEBUG] Pre render: {c}"))
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
///     name
/// }
///
/// /// Renders the greeting message with the provided name.
/// #[renderer]
/// fn render_name(name: ResultName) {
///     r_println!("Hello, {}!", *name);
/// }
///
/// gen_program!();
/// ```
pub mod example_hook {}
/// Example Implicit Dispatcher
///
///  > This example demonstrates how to use the implicit `dispatcher!` definition syntax enabled by `extra_macros`
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
/// features = ["extra_macros"]
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
/// fn render_entry_show(_args: EntryShow, res: &mut LazyRes<ResLargeData>) {
///     //             _______ Initialization happens here
///     //            /
///     //            vvvvvvv
///     let res = res.get_ref();
///     for (key, value) in &res.data {
///         r_println!("{}: {}", key, value);
///     }
/// }
///
/// // When not using LazyRes<ResLargeData>, it will not be initialized
/// #[renderer]
/// fn render_entry_none(_args: EntryNone) {
///     r_println!("None");
/// }
///
/// gen_program!();
/// ```
pub mod example_lazy_resources {}
/// Example `pack_err!`
///
///  > This example demonstrates how to use the `pack_err!` macro to define error types
///  > with automatic `name` field (set to snake_case at compile time) and optional `info` field.
///  > Also demonstrates `--json` serialization when `general_renderer` is enabled.
///
///  Run:
///  ```bash
///  cargo run --manifest-path examples/example-pack-err/Cargo.toml --quiet -- find
///  cargo run --manifest-path examples/example-pack-err/Cargo.toml --quiet -- find --json
///  cargo run --manifest-path examples/example-pack-err/Cargo.toml --quiet -- find Cargo.toml
///  cargo run --manifest-path examples/example-pack-err/Cargo.toml --quiet -- find Cargo.toml --json
///  cargo run --manifest-path examples/example-pack-err/Cargo.toml --quiet -- find src
///  cargo run --manifest-path examples/example-pack-err/Cargo.toml --quiet -- find src --json
///  ```
///
///  Output:
///  ```plaintext
///  Search path not provided
///  {"name":"error_not_found"}
///  Not a directory: Cargo.toml
///  {"name":"error_not_dir","info":"Cargo.toml"}
///  Found directory: src
///  {"inner":"src"}
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
///     "general_renderer",
///     "extra_macros",
/// ]
///
/// [workspace]
/// ```
///
/// Source code (./src/main.rs)
/// ```ignore
/// use mingling::prelude::*;
/// use mingling::setup::GeneralRendererSetup;
/// use std::path::PathBuf;
///
/// dispatcher!("find", CMDFind => EntryFind);
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
/// // When `general_renderer` is enabled, the struct also gets
/// // `#[derive(serde::Serialize)]` for --json / --yaml output.
/// // --------- IMPORTANT ---------
///
/// // Simple form — name = "error_not_found"
/// pack_err!(ErrorNotFound);
///
/// // Typed form — name = "error_not_dir"
/// pack_err!(ErrorNotDir = PathBuf);
///
/// // Success type using traditional pack!
/// pack!(ResultPath = PathBuf);
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
/// /// Renders the successful result with the found directory path.
/// #[renderer]
/// fn render_result_path(path: ResultPath) {
///     r_println!("Found directory: {}", path.display());
/// }
///
/// /// Renders the error when no search path is provided.
/// #[renderer]
/// fn render_error_not_found(_: ErrorNotFound) {
///     r_println!("Search path not provided");
/// }
///
/// /// Renders the error when the given path is not a directory.
/// #[renderer]
/// fn render_error_not_dir(err: ErrorNotDir) {
///     r_println!("Not a directory: {}", err.info.display());
/// }
///
/// gen_program!();
///
/// fn main() {
///     let mut program = ThisProgram::new();
///     // Add GeneralRendererSetup to support --json / --yaml flags
///     program.with_setup(GeneralRendererSetup);
///     program.with_dispatcher(CMDFind);
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
///     program.with_hook(ProgramHook::empty().on_exec_panic(|info| println!("Program panic: {info}")));
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
///         None => NotPanic::default(),
///     }
/// }
///
/// /// Renders the message when no panic occurs.
/// #[renderer]
/// fn render(_: NotPanic) {
///     r_println!("Program not panic");
/// }
///
/// gen_program!();
/// ```
pub mod example_panic_unwind {}
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
/// features = ["repl", "parser", "extra_macros"]
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
///     program.with_hook(ProgramHook::empty().on_repl_begin(|| {
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
///     StateChangeDirectory::new(join)
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
/// fn render_list(list: ResultList) {
///     for item in list.inner {
///         r_println!("{}", item);
///     }
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
/// fn render_error_directory_not_exist(err: ErrorDirectoryNotExist) {
///     r_println!("Directory not found: {}", err.inner.display())
/// }
///
/// /// Handle dispatcher not found event
/// /// Renders the error when a command is not found.
/// #[renderer]
/// fn dispatcher_not_found(prev: ErrorDispatcherNotFound) {
///     r_println!("Command not found: \"{}\"", prev.join(", "))
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
/// use std::path::PathBuf;
///
/// use mingling::prelude::*;
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
///     EntryCurrent::default()
/// }
///
/// // Define renderer for output current path       _____________ Injected resource
/// //                                              /
/// /// Renders the current directory path.         |
/// #[renderer] //                                  vvvvvvvvvvvvvv
/// fn render_current(_: EntryCurrent, current_dir: &ResCurrentDir) {
///     r_println!("Current directory: {}", current_dir.current_dir.display());
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
/// mingling = { path = "../../mingling", features = ["extra_macros"] }
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
/// mingling = { path = "../../mingling", features = ["extra_macros"] }
///
/// [workspace]
/// ```
///
/// Source code (./src/main.rs)
/// ```ignore
/// use mingling::prelude::*;
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
///         assert_eq!(r, "Hello, Peter!\n")
///     }
///
///     #[test]
///     fn test_render_error_no_name_provided() {
///         let r = render_error_no_name_provided(ErrorNoNameProvided::default());
///         assert_eq!(r, "No name provided\n")
///     }
///
///     #[test]
///     fn test_render_error_name_not_available() {
///         let r = render_error_name_not_available(ErrorNameNotAvailable::default());
///         assert_eq!(r, "Name not available\n")
///     }
///
///     #[test]
///     fn test_render_error_name_too_long() {
///         let r = render_error_name_too_long(ErrorNameTooLong::new(17));
///         assert_eq!(r, "Name too long: 17 > 10\n")
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
/// fn render_result_name(name: ResultName) -> String {
///     r_println!("Hello, {}!", *name);
/// }
///
/// /// Renders the error when no name is provided.
/// #[renderer]
/// fn render_error_no_name_provided(_: ErrorNoNameProvided) -> String {
///     r_println!("No name provided");
/// }
///
/// /// Renders the error when the name is already taken.
/// #[renderer]
/// fn render_error_name_not_available(_: ErrorNameNotAvailable) -> String {
///     r_println!("Name not available");
/// }
///
/// /// Renders the error when the name exceeds the maximum length.
/// #[renderer]
/// fn render_error_name_too_long(len: ErrorNameTooLong) -> String {
///     r_println!("Name too long: {} > 10", *len);
/// }
///
/// /// Renders the error when the dispatcher (subcommand) is not found.
/// #[renderer]
/// fn render_dispatcher_not_found(err: ErrorDispatcherNotFound) {
///     r_println!("Command not found: \"{}\"", err.inner.join(" "));
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
