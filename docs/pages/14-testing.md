<h1 align="center">Testing Your Program</h1>
<p align="center">
    Writing unit tests for Chain and Renderer
</p>

A built-in benefit of the pipeline model is **testability**.

A Chain is just a function that takes input and returns output; a Renderer is just a function that takes input and writes content — no global state magic, testing is straightforward.

## Testing Renderer

Renderer is the easiest to test — call the function, assert the result:

```rust
@@@#[derive(Grouped, Wrap)]
@@@pub struct ResultName(String);
#[renderer]
fn render_greet(result: ResultName) -> RenderResult {
    let mut r = RenderResult::new();
    r_println!(r, "Hello, {}!", *result);
    r
}
 
#[test]
fn test_render_name() {
    let result = render_name(ResultName("Alice".to_string()));
    assert_eq!(result.to_string().as_str(), "Hello, Alice!\n");
}
```
 
Note the Renderer's return type changed to `-> String` — `#[renderer]` auto-converts `RenderResult` to whatever return type you specify (default is `()`). By returning `String`, you can directly assert on the output content.

## Testing Chain

Testing a Chain is slightly more complex because its return value is `Next` (actually `impl Into<ChainProcess<ThisProgram>>`). You'll need the assertion macros provided by the framework:

```rust
@@@use mingling::{assert_member_id, assert_render_result, unpack_chain_process};
@@@dispatcher!("hello", EntryHello);
@@@#[derive(Grouped, Wrap)]
@@@pub struct ResultName(String);
@@@#[derive(Grouped, Wrap, Default)]
@@@pub struct ErrorNoName(());
@@@#[chain]
@@@fn handle_hello(args: EntryHello) -> Next {
@@@    let name = args.0.first().cloned().unwrap_or_default();
@@@    if name.is_empty() {
@@@        ErrorNoName::default().to_render()
@@@    } else {
@@@        ResultName(name).to_render()
@@@    }
@@@}
#[test]
fn test_handle_hello_with_name() {
    let chain_process = handle_hello(EntryGreet(vec!["Alice".to_string()])).into();
    // Asserts this is a render result (not continuing the chain)
    assert_render_result!(chain_process);
    // Asserts member_id is ResultName
    assert_member_id!(chain_process, ResultName);
    // Unpacks the inner value
    let result_name = unpack_chain_process!(chain_process, ResultName);
    assert_eq!(result_name.0, "Alice");
}
```
 
What the three test macros do:

| Macro                   | Function                                                          |
| ----------------------- | ----------------------------------------------------------------- |
| `assert_render_result!` | Asserts Chain returned the render path (not continuing the chain) |
| `assert_member_id!`     | Asserts the return value's member ID is a certain type            |
| `unpack_chain_process!` | Unpacks the original type from ChainProcess                       |

## Constructing Data with the entry! Macro

You can use `entry!` to quickly construct an Entry:

```rust
@@@use mingling::{assert_member_id, unpack_chain_process};
@@@use mingling::macros::entry;
@@@dispatcher!("hello", EntryHello);
@@@#[derive(Grouped, Wrap)]
@@@pub struct ResultName(String);
@@@#[chain]
@@@fn handle_hello(args: EntryHello) -> Next {
@@@    let name = args.0.first().cloned().unwrap_or_default();
@@@    ResultName(name).to_render()
@@@}
#[test]
fn test_with_entry_macro() {
    // entry! constructs an Entry from string literals
    let entry = entry!("--name", "Alice");
    let chain_process = handle_hello(entry).into();
    let result_name = unpack_chain_process!(chain_process, ResultName);
    assert_eq!(result_name.0, "Alice");
}
```
 
## Testing Resource Injection

If a Chain uses resources, you need to provide resource instances in the test:

```rust
@@@use mingling::{assert_render_result, unpack_chain_process};
@@@#[derive(Default, Clone)]
@@@struct ResPrefix(String);
@@@dispatcher!("hello", EntryHello);
@@@#[derive(Grouped, Wrap)]
@@@pub struct ResultGreeting(String);
@@@
#[chain]
fn handle_hello(args: EntryHello, prefix: &ResPrefix) -> Next {
    let name = args.0.first().cloned().unwrap_or_default();
    ResultGreeting(format!("{}, {}", prefix.0, name)).to_render()
}
 
#[test]
fn test_handle_with_resource() {
    // Resources need to be passed manually in tests
    let result = handle_hello(
        EntryHello(vec!["World".to_string()]),
        &ResPrefix("Hello".to_string()),
    );
    let greeting = unpack_chain_process!(result, ResultGreeting, ThisProgram);
    assert_eq!(greeting.0, "Hello, World");
}
```
 
The pipeline model makes testing simple: each Chain and Renderer is a relatively independent function — construct input, assert output.

<p align="center" style="font-size: 0.85em; color: clear;">
    Written by @Weicao-CatilGrass
</p>
