<h1 align="center">Declare a Chain</h1>
<p align="center">
    Use the <code>chain</code> macro to declare a chain and handle Entry input
</p>

In the previous section, we declared `dispatcher!("greet", EntryGreet)`.

Now when a user types `greet`, it gets matched and wrapped into `EntryGreet`.

But what happens after we get the Entry?

We need a Chain to process it.

## The `#[chain]` Macro

`#[chain]` marks a handler function. The format is straightforward:

```rust
@@@dispatcher!("greet", EntryGreet);
pack!(ResultName = String);
 
#[chain]
fn handle_greet(args: EntryGreet) -> Next {
    // args contains the remaining params after matching user input
    let name = args.inner.first().cloned().unwrap_or_else(|| "World".to_string());
    // Wrap the result into Next, telling the dispatcher where to go next
    ResultName::new(name).into()
}
```
 
Notice anything?

The Chain function signature declares what it needs — `args: EntryGreet`.

Then it returns a newtype via `ResultName::new(name)`.

This returned `Next` expands into `impl Into<ChainProcess<ThisProgram>>`.

> [!TIP]
> Wondering how `Into<ChainProcess<G>>` works?
>
> Check out the [Any Output Mechanism](pages/concepts/3-any-output) chapter to learn about `ChainProcess`.

## The `pack!` Macro

You've probably guessed it — `pack!(ResultName = String)` defines a type that flows through the pipeline:

```rust
// pack!(ResultName = String) generates code roughly like this
 
#[derive(Grouped)]
pub struct ResultName {
    pub inner: String,
}
```
 
Think of it as a **tagged** `String`.

The dispatcher uses this tag for precise routing, ensuring data doesn't get mixed up — e.g., data sent to `RenderGreet` won't be misdelivered to `RenderError`.

> [!NOTE]
> Unlike a simple type alias (`type`), `pack!` generates a completely new type with its own `TypeId`.

Here's a recommended naming convention:

| Role         | Naming Pattern         | Example              |
| ------------ | ---------------------- | -------------------- |
| Entry        | `Entry` + command      | `EntryGreet`         |
| Intermediate | `State` + description  | `StateParsedArgs`    |
| Result       | `Result` + description | `ResultGreetSomeone` |
| Error        | `Error` + description  | `ErrorUserNotFound`  |

See [Naming Convention](pages/other/naming_rule) for details, but for now just remember: **use `pack!` to give your data a meaningful name**.

## Extracting Params from Entry

`EntryGreet`'s `inner` is a `Vec<String>`, which you can freely process inside a Chain:

```rust
@@@dispatcher!("greet", EntryGreet);
@@@pack!(ResultName = String);
#[chain]
fn handle_greet(args: EntryGreet) -> Next {
    // Take the first param, or use a default
    let name = args
        .inner
        .first()
        .cloned()
        .unwrap_or_else(|| "World".to_string());
 
    ResultName::new(name).into()
}
```
 
If you enable the `picker` feature, you can also use `Picker` for more flexible param extraction — but that's a topic for later.

## Putting It Together

Now let's connect the Dispatcher and Chain:

```rust
// 1. Declare the command
dispatcher!("greet", EntryGreet);
 
// 2. Declare the pipeline data type
pack!(ResultName = String);
 
// 3. Processing logic
#[chain]
fn handle_greet(args: EntryGreet) -> Next {
    let name = args.inner
        .first()
        .cloned()
        .unwrap_or_else(|| "World".to_string());
    ResultName::new(name).into()
}
 
fn main() {
    let mut program = ThisProgram::new();
    program.exec_and_exit();
}
 
gen_program!();
```
 
But this code isn't complete yet — we only have the Dispatcher and Chain. One last step remains: **rendering the result**. That's what the next chapter, Renderer, covers.

<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
