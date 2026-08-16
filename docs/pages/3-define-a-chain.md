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
#[derive(Grouped, Wrap)]
pub struct ResultName(String);
 
#[chain]
fn handle_greet(args: EntryGreet) -> Next {
    // args contains the remaining params after matching user input
    let name = args.0.first().cloned().unwrap_or_else(|| "World".to_string());
    // Wrap the result into Next, telling the dispatcher where to go next
    ResultName(name).into()
}
```
 
Notice anything?

The Chain function signature declares what it needs — `args: EntryGreet`.

Then it returns a newtype via `ResultName(name)`.

This returned `Next` expands into `impl Into<ChainProcess<ThisProgram>>`.

> [!TIP]
> Wondering how `Into<ChainProcess<G>>` works?
>
> Check out the [Any Output Mechanism](pages/concepts/3-any-output) chapter to learn about `ChainProcess`.

## Declaring Types with `#[derive(Grouped, Wrap)]`

You've probably guessed it — `#[derive(Grouped, Wrap)] pub struct ResultName(String);` defines a type that flows through the pipeline:

```rust
// #[derive(Grouped, Wrap)] generates code roughly like this
 
pub struct ResultName(String);
 
impl From<String> for ResultName {
    fn from(inner: String) -> Self {
        ResultName(inner)
    }
}
 
impl std::ops::Deref for ResultName {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
 
// Grouped generates member_id() → ThisProgram::ResultName,
// giving the type its routing identity and Into<ChainProcess> conversion.
```
 
Think of it as a **tagged** `String`.

The dispatcher uses this tag for precise routing, ensuring data doesn't get mixed up — e.g., data sent to `RenderGreet` won't be misdelivered to `RenderError`.

> [!NOTE]
> Unlike a simple type alias (`type`), `#[derive(Grouped, Wrap)]` declares a completely new type with its own `TypeId`.

Here's a recommended naming convention:

| Role         | Naming Pattern         | Example              |
| ------------ | ---------------------- | -------------------- |
| Entry        | `Entry` + command      | `EntryGreet`         |
| Intermediate | `State` + description  | `StateParsedArgs`    |
| Result       | `Result` + description | `ResultGreetSomeone` |
| Error        | `Error` + description  | `ErrorUserNotFound`  |

See [Naming Convention](pages/other/naming_rule) for details, but for now just remember: **use `#[derive(Grouped)]` (optionally with `Wrap`) to give your data a meaningful name**.

## Extracting Params from Entry

`EntryGreet`'s `.0` is a `Vec<String>`, which you can freely process inside a Chain:

```rust
@@@dispatcher!("greet", EntryGreet);
@@@#[derive(Grouped, Wrap)]
@@@pub struct ResultName(String);
#[chain]
fn handle_greet(args: EntryGreet) -> Next {
    // Take the first param, or use a default
    let name = args
        .0
        .first()
        .cloned()
        .unwrap_or_else(|| "World".to_string());
 
    ResultName(name).into()
}
```
 
If you enable the `picker` feature, you can also use `Picker` for more flexible param extraction — but that's a topic for later.

## Putting It Together

Now let's connect the Dispatcher and Chain:

```rust
// 1. Declare the command
dispatcher!("greet", EntryGreet);
 
// 2. Declare the pipeline data type
#[derive(Grouped, Wrap)]
pub struct ResultName(String);
 
// 3. Processing logic
#[chain]
fn handle_greet(args: EntryGreet) -> Next {
    let name = args.0
        .first()
        .cloned()
        .unwrap_or_else(|| "World".to_string());
    ResultName(name).into()
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
