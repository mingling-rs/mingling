<h1 align="center">Multi-Command Program</h1>
<p align="center">
    Adding multiple commands to a single program
</p>

Real-world CLIs rarely have just one command. Let's extend our previous greet program by adding a second command, and see what a multi-command program looks like.

## Adding a Second Command

Work in the same project:

```rust
// Declare two commands
dispatcher!("greet", CMDGreet => EntryGreet);
dispatcher!("add",   CMDAdd   => EntryAdd);
 
pack!(ResultGreeting = String);
pack!(ResultSum = i32);
 
#[chain]
fn handle_greet(args: EntryGreet) -> Next {
    let name = args.inner.first().cloned().unwrap_or_else(|| "World".to_string());
    ResultGreeting::new(name)
}
 
#[chain]
fn handle_add(args: EntryAdd) -> Next {
    let sum: i32 = args.inner.iter().filter_map(|s| s.parse::<i32>().ok()).sum();
    ResultSum::new(sum)
}
 
#[renderer]
fn render_greet(result: ResultGreeting) -> RenderResult {
    let mut r = RenderResult::new();
    writeln!(r, "Hello, {}!", *result).ok();
    r
}
 
#[renderer]
fn render_sum(result: ResultSum) -> RenderResult {
    let mut r = RenderResult::new();
    writeln!(r, "Sum: {}", *result).ok();
    r
}
 
fn main() {
    let mut program = ThisProgram::new();
    program.with_dispatchers((CMDGreet, CMDAdd));
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
 
## Registering Multiple Dispatchers

Notice `with_dispatchers`? When you need to register multiple dispatchers, just pass them as a tuple:

```rust
@@@dispatcher!("greet", CMDGreet => EntryGreet);
@@@dispatcher!("add", CMDAdd => EntryAdd);
@@@pack!(ResultGreeting = String);
@@@pack!(ResultSum = i32);
@@@#[chain] fn handle_greet(_args: EntryGreet) -> Next { ResultGreeting::new("ok".into()) }
@@@#[renderer] fn render_greet(_greeting: ResultGreeting) -> RenderResult { RenderResult::new() }
@@@#[chain] fn handle_add(_args: EntryAdd) -> Next { ResultSum::new(0) }
@@@#[renderer] fn render_sum(_sum: ResultSum) -> RenderResult { RenderResult::new() }
fn main() {
    let mut program = ThisProgram::new();
    program.with_dispatchers((CMDGreet, CMDAdd));
    program.exec_and_exit();
}
```
 
This is equivalent to registering them one by one, same effect.

> [!TIP]
> The tuple supports up to 7 dispatchers. For more than 7, chain `with_dispatcher` calls instead.

## Subcommands

Multi-level commands work the same way—each dot-separated level is just part of the name:

```rust
dispatcher!("remote.add", CMDRemoteAdd => EntryRemoteAdd);
dispatcher!("remote.rm",  CMDRemoteRm  => EntryRemoteRm);
```
 
Each subcommand's Entry, Chain, and Renderer are completely independent and don't interfere.

## Type Independence

Notice we used two different `pack!` macros:

- `pack!(ResultGreeting = String)`
- `pack!(ResultSum = i32)`

They are independent types, and `gen_program!()` assigns them different enum variants.

The dispatcher will never route `ResultGreeting` data to `render_sum` — **type safety is guaranteed from the naming stage**.

<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
