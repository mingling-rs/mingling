<h1 align="center">Rendering Results</h1>
<p align="center">
    Declare a renderer using the <code>#[renderer]</code> macro to output results.
</p>

Now we've created a Dispatcher and a Chain, and produced a Result type via `pack!`. The final step: **present the result to the user**.

## The `#[renderer]` Macro

Similar to `#[chain]`, `#[renderer]` marks a function that produces output:

```rust
@@@use mingling::macros::buffer;
@@@pack!(ResultName = String);
#[renderer(buffer)]
fn render_name(name: ResultName) {
    r_println!("Hello, {}!", *name);
}
```
 
A Renderer receives the result produced by a Chain and returns a `RenderResult`. Inside the function, you create a `RenderResult`, write content with `r_print!` / `r_println!`, and finally return it.

## The `buffer` Extension

If you find explicitly creating and returning a `RenderResult` too verbose, you can use `#[renderer(buffer)]` to inject a buffer directly into your function.

```rust
use mingling::macros::buffer;
 
@@@pack!(ResultName = String);
#[renderer(buffer)]
fn render_name(name: ResultName) {
    r_println!("Hello, {}!", *name);
}
```
 
This gives your renderer function a more concise syntax, but also introduces an implicit mechanism: it injects a mutable `RenderResult` variable named `__render_result_buffer` into the function. When the `r_print!` macro is used without an explicit `RenderResult`, it will append output to that buffer by convention.

## The `RenderResult` Type

`RenderResult` is a buffer type that holds rendered text and an exit code. Instead of writing directly to the terminal, it writes content into an internal buffer. This approach gives us:

1. **Exit code support**—you can set the program to exit with a specific exit code
2. **Testability**—rendered output can be captured and asserted against
3. **Post-processing**—the result can be captured and further processed uniformly

## A Complete Runnable Program

Putting all three tutorials together, here's your first complete Mingling program:

```rust
use mingling::macros::buffer;
 
// 1. Declare commands with a Dispatcher
dispatcher!("greet", CMDGreet => EntryGreet);
 
// 2. Declare result data with pack!
pack!(ResultName = String);
 
// 3. Handle logic with a Chain
#[chain]
fn handle_greet(args: EntryGreet) -> Next {
    let name = args.inner
        .first()
        .cloned()
        .unwrap_or_else(|| "World".to_string());
    ResultName::new(name).into()
}
 
// 4. Output results with a Renderer
#[renderer(buffer)]
fn render_name(name: ResultName) {
    r_println!("Hello, {}!", *name);
}
 
// 5. Assemble and run the program in main
fn main() {
    let mut program = ThisProgram::new();
    program.with_dispatcher(CMDGreet);
    program.exec_and_exit();
}
 
// 6. Use gen_program! to generate the full program
gen_program!();
```
 
## Try It Out

```bash
~# cargo run -- greet Alice
```
 
```text
Hello, Alice!
```
 
Try without arguments:

```bash
~# cargo run -- greet
```
 
```text
Hello, World!
```
 
Try a non-existent command:

```bash
cargo run -- great
```
 
```text
# No output!
```
 
## Adding a Fallback

`gen_program!()` auto-generates an `ErrorDispatcherNotFound` type wrapping `Vec<String>`—it holds the user input that didn't match any command. You just need to write a Renderer for it:

```rust
use mingling::macros::buffer;
 
#[renderer(buffer)]
fn render_dispatcher_not_found(err: ErrorDispatcherNotFound) {
    if err.inner.is_empty() {
        r_println!("Unknown command");
    } else {
        r_println!("Command not found: \"{}\"", err.inner.join(" "));
    }
}
```
 
With that added, try the non-existent command again:

```bash
cargo run -- great
```
 
```text
Command not found: "great"
```
 
## Congratulations

You've completed your first full Mingling program! Let's recap what you've learned:

| Concept        | Macro / Function | One-liner                               |
| -------------- | ---------------- | --------------------------------------- |
| Declare cmds   | `dispatcher!`    | Tell the program what the user can type |
| Handle logic   | `#[chain]`       | What to do when args are received       |
| Output results | `#[renderer]`    | How to present results to the user      |
| Type wrapping  | `pack!`          | Give your data a meaningful name        |
| Program entry  | `gen_program!()` | Auto-generate the pipeline wiring       |

In real projects you'll also use advanced features like resource injection, hooks, completions, REPL, etc., but the core skeleton stays the same: **Dispatcher → Chain → Renderer**.

<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
