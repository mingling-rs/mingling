<h1 align="center">Error Handling</h1>
<p align="center">
    Gracefully present errors to the user
</p>

A pipeline isn't just the happy path. When input is invalid, a resource isn't found, or an operation fails, you need a place to handle these "surprises" instead of letting the program panic.

## Two Paths: Success vs. Error

Recall the pipeline model: Chain's return value is `Next`, which has two destinations:

| Route          | Meaning                                     |
| -------------- | ------------------------------------------- |
| `.to_render()` | Got a result, hand it to a Renderer to show |
| `.to_chain()`  | Not done yet, hand it to the next Chain     |

Error values can also take either path—you can render the error msg directly, or pass it to the next Chain for potential recovery.

## Distinguish Errors with Dedicated Types

```rust
@@@dispatcher!("greet", EntryGreet);
#[derive(Grouped, Wrap)]
pub struct ResultGreeting(String);
#[derive(Grouped, Wrap)]
pub struct ErrorNameEmpty(String);
 
#[chain]
fn handle_greet(args: EntryGreet) -> Next {
    let name = args.0.first().cloned().unwrap_or_default();
 
    if name.is_empty() {
        ErrorNameEmpty("name is required".to_string()).to_render()
    } else {
        ResultGreeting(name).to_render()
    }
}
```
 
Then write separate Renderers:

```rust
@@@use mingling::macros::buffer;
@@@dispatcher!("greet", EntryGreet);
@@@#[derive(Grouped, Wrap)]
@@@pub struct ResultGreeting(String);
@@@#[derive(Grouped, Wrap)]
@@@pub struct ErrorNameEmpty(String);
@@@#[chain] fn handle_greet(args: EntryGreet) -> Next { ResultGreeting(args.0.first().cloned().unwrap_or_default()).to_render() }
 
#[renderer(buffer)]
fn render_greet(result: ResultGreeting) {
    r_println!("Hello, {}!", *result);
}
 
#[renderer(buffer)]
fn render_error_name_empty(err: ErrorNameEmpty) {
    r_println!("Error: {}", *err);
}
```
 
Each Renderer does its own job; what the user sees depends on what the Chain returned.

## Complete Example

```rust
@@@use mingling::macros::buffer;
dispatcher!("greet", EntryGreet);
 
#[derive(Grouped, Wrap)]
pub struct ResultGreeting(String);
#[derive(Grouped, Wrap)]
pub struct ErrorNameEmpty(String);
 
#[chain]
fn handle_greet(args: EntryGreet) -> Next {
    let name = args.0.first().cloned().unwrap_or_default();
    if name.is_empty() {
        ErrorNameEmpty("name is required".to_string()).to_render()
    } else {
        ResultGreeting(name).to_render()
    }
}
 
#[renderer(buffer)]
fn render_greet(result: ResultGreeting) {
    r_println!("Hello, {}!", *result);
}
 
#[renderer(buffer)]
fn render_error_name_empty(err: ErrorNameEmpty) {
    r_println!("Error: {}", *err);
}
 
fn main() {
    let mut program = ThisProgram::new();
    program.exec_and_exit();
}
 
gen_program!();
```
 
Output:

```text
~# my-cli greet Alice
Hello, Alice!
 
~# my-cli greet
Error: name is required
```
 
## Declaring Error Types

You can use `#[derive(Grouped, Default)]` to quickly declare an error type with no payload:

```rust
#[derive(Grouped, Default)]
pub struct ErrorNotFound;
```
 
See [Feature List](pages/other/features) for details.

<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
