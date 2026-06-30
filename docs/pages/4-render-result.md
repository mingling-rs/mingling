<h1 align="center">Rendering Results</h1>
<p align="center">
    Use the <code>#[renderer]</code> macro to declare a renderer and output results
</p>

Now that we've created a Dispatcher and a Chain, and produced a Result type via `pack!`, there's one final step: **presenting the result to the user**.

## The `#[renderer]` Macro

Similar to `#[chain]`, `#[renderer]` marks an output function:

```rust
pack!(ResultName = String);
#[renderer]
fn render_name(name: ResultName) {
    r_println!("Hello, {}!", *name);
}
```
 
A Renderer takes the result produced by a Chain and outputs it using `r_println!`. What's the difference between `r_println!` and the usual `println!`?

## The `r_println!` and `r_print!` Macros

`r_println!` and `r_print!` are printing macros provided by Mingling. They write content into a `RenderResult` instead of printing directly to the terminal. This offers several benefits:

1. **RenderResult holds an exit code** — you can make the program exit with a specific code
2. **Easier testing** — you can capture rendered output and make assertions
3. **Post-processing** — you can capture results and apply uniform text post-processing

> [!TIP]
> For simple printing, you can think of it as a drop-in replacement for `println!`. Using `r_println!` instead of `println!` is a safe choice.

## A Complete Runnable Program

Putting the content of all three tutorials together, here's your first complete Mingling program:

```rust
// 1. Declare a command with Dispatcher
dispatcher!("greet", CMDGreet => EntryGreet);
 
// 2. Declare result data with pack!
pack!(ResultName = String);
 
// 3. Handle logic with Chain
#[chain]
fn handle_greet(args: EntryGreet) -> Next {
    let name = args.inner
        .first()
        .cloned()
        .unwrap_or_else(|| "World".to_string());
    ResultName::new(name)
}
 
// 4. Output results with Renderer
#[renderer]
fn render_name(name: ResultName) {
    r_println!("Hello, {}!", *name);
}
 
// 5. Assemble and run the program in main
fn main() {
    let mut program = ThisProgram::new();
    program.with_dispatcher(CMDGreet);
    program.exec_and_exit();
}
 
// 6. Generate the complete program with gen_program!
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
 
Try an unknown command:

```bash
cargo run -- great
```
 
```text
# No output!
```
 
## Add a Fallback

`gen_program!()` auto-generates an `ErrorDispatcherNotFound` type that wraps `Vec<String>` — it holds the user's unmatched input. You just need to write a Renderer for it:

```rust
#[renderer]
fn render_dispatcher_not_found(err: ErrorDispatcherNotFound) {
    if err.inner.is_empty() {
        r_println!("Unknown command");
    } else {
        r_println!("Command not found: \"{}\"", err.inner.join(" "));
    }
}
```
 
With that added, try the unknown command again:

```bash
cargo run -- great
```
 
```text
Command not found: "great"
```
 
## Congratulations

You've completed your first full Mingling program! Here's a recap of what you've learned:

| Concept         | Macro/Function   | In a Nutshell                         |
| --------------- | ---------------- | ------------------------------------- |
| Declare command | `dispatcher!`    | Tell the program what users can input |
| Handle logic    | `#[chain]`       | What to do with the arguments         |
| Output results  | `#[renderer]`    | How to present results to users       |
| Type wrapper    | `pack!`          | Give your data a meaningful name      |
| Program entry   | `gen_program!()` | Auto-generate the pipeline wiring     |

In real projects you'll also use advanced features like resource injection, hooks, autocompletion, REPL, etc., but the core skeleton stays the same: **Dispatcher → Chain → Renderer**.

<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
