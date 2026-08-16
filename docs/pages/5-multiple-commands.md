<h1 align="center">Multi-Command Program</h1>
<p align="center">
    Adding multiple commands to a single program
</p>

Real-world CLIs rarely have just one command. Let's extend our previous greet program by adding a second command, and see what a multi-command program looks like.

## Adding a Second Command

Work in the same project:

```rust
@@@use mingling::macros::buffer;
// Declare two commands
dispatcher!("greet", EntryGreet);
dispatcher!("add",   EntryAdd);
 
#[derive(Grouped, Wrap)]
pub struct ResultGreeting(String);
#[derive(Grouped, Wrap)]
pub struct ResultSum(i32);
 
#[chain]
fn handle_greet(args: EntryGreet) -> Next {
    let name = args.0.first().cloned().unwrap_or_else(|| "World".to_string());
    ResultGreeting(name).into()
}
 
#[chain]
fn handle_add(args: EntryAdd) -> Next {
    let sum: i32 = args.0.iter().filter_map(|s| s.parse::<i32>().ok()).sum();
    ResultSum(sum).into()
}
 
#[renderer(buffer)]
fn render_greet(result: ResultGreeting) {
    r_println!("Hello, {}!", *result);
}
 
#[renderer(buffer)]
fn render_sum(result: ResultSum) {
    r_println!("Sum: {}", *result);
}
 
fn main() {
    let mut program = ThisProgram::new();
    program.exec_and_exit();
}
 
gen_program!();
```
 
Both commands share the same pipeline model, but each has its own path:

```text
> my-cli greet Alice
Hello, Alice!
> my-cli add 1 2 3
Sum: 6
```
 
## Subcommands

Multi-level commands work the same way—each dot-separated level is just part of the name:

```rust
dispatcher!("remote.add", EntryRemoteAdd);
dispatcher!("remote.rm",  EntryRemoteRm);
```
 
Each subcommand's Entry, Chain, and Renderer are completely independent and don't interfere.

## Type Independence

Notice we used two different `#[derive(Grouped, Wrap)]` structs:

- `#[derive(Grouped, Wrap)] pub struct ResultGreeting(String);`
- `#[derive(Grouped, Wrap)] pub struct ResultSum(i32);`

They are independent types, and `gen_program!()` assigns them different enum variants.

The dispatcher will never route `ResultGreeting` data to `render_sum` — **type safety is guaranteed from the naming stage**.

<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
