<h1 align="center">Parsing Arguments with Picker</h1>
<p align="center">
    Use Picker to perform basic argument parsing
</p>

In previous tutorials, we manually extracted parameters from `EntryGreet.inner` (`Vec<String>`).

```rust
@@@ fn main() {
@@@ let args : Vec<String> = vec![];
let name = args.first().cloned().unwrap_or_else(|| "World".to_string());
@@@ }
```
 
But this approach doesn't scale well when there are many params. Mingling provides `Picker` — a chained API for extracting and transforming params.

To enable `Picker`, update your `Cargo.toml`:

```toml
# Cargo.toml
[dependencies.mingling]
features = ["picker"]
```
 
Now let's see how `Picker` is written:

```rust
// Features: ["picker"]
@@@dispatcher!("greet", EntryGreet);
@@@pack!(ResultName = String);
 
#[chain]
fn handle_greet_entry(prev: EntryGreet) -> Next {
    let name = prev
        .pick_or(&arg![String], || "World".to_string())
        .unwrap();
    ResultName::new(name).into()
}
```
 
`EntryPicker` implements `pick`, `pick_or`, `pick_or_default`, and `pick_or_route` for all entry types. These functions semantically **pick** params from the string list and convert them into structured data, using the `arg!` macro to declare what to pick.

For the code above:

```rust
// Features: ["picker"]
@@@dispatcher!("greet", EntryGreet);
@@@pack!(ResultName = String);
@@@#[chain]
@@@fn handle_greet_entry(prev: EntryGreet) -> Next {
let name = prev
    .pick_or(&arg![String], || "World".to_string())
    .unwrap();
@@@ResultName::new(name).into()
@@@}
```
 
Its semantics are:

```rust
// Features: ["picker"]
@@@dispatcher!("greet", EntryGreet);
@@@pack!(ResultName = String);
@@@#[chain]
@@@fn handle_greet_entry(prev: EntryGreet) {
@@@let name: String =
   prev.pick_or(&arg![String], || "World".to_string()).unwrap();
// ~~~~ ~~~~~~~ ~~~~~~~~~~~~  ~~~~~~~~~~~~~~~~~~~~~~~~ ~~~~~~
// |    |       |             |                        |_ unwrap to String
// |    |       |             |__________________________ default value "World"
// |    |       |________________________________________ pick the first positional arg (declared as `String`)
// |    |________________________________________________ pick or use default
// |_____________________________________________________ from the previous input
@@@}
```
 
## Parsing Flag Arguments

If your program needs to parse flag arguments (e.g. `greet --name Alice`), declare a named flag in `arg!`:

```rust
// Features: ["picker"]
@@@dispatcher!("greet", EntryGreet);
@@@pack!(ResultName = String);
 
#[chain]
fn handle_greet_entry(prev: EntryGreet) -> Next {
    let name = prev
        .pick_or(&arg![name: String, 'n'], || "World".to_string())
        .unwrap();
    ResultName::new(name).into()
}
```
 
The `arg!` macro derives the long flag name (`--name`) from the field name, and `'n'` adds the short alias (`-n`).

Its semantics:

```rust
// Features: ["picker"]
@@@dispatcher!("greet", EntryGreet);
@@@pack!(ResultName = String);
@@@#[chain]
@@@fn handle_greet_entry(prev: EntryGreet) {
@@@let name: String =
   prev.pick_or(&arg![name: String, 'n'], || "World".to_string()).unwrap();
// ~~~~ ~~~~~~~ ~~~~~~~~~~~~~~~~~~~~~~~~~  ~~~~~~~~~~~~~~~~~~~~~~ ~~~~~~
// |    |       |                          |                      |_ unwrap to String
// |    |       |                          |________________________ default value "World"
// |    |       |___________________________________________________ pick the value after "--name" or "-n"
// |    |___________________________________________________________ pick or use default
// |________________________________________________________________ from the previous input
@@@}
```
 
## About `.unwrap()` and `route!`

You may have noticed that `Picker` calls `.unwrap()` (or `route!`) at the end of parsing. It converts the collected results into structured info.

For a single pick, `.unwrap()` returns the value directly; for multiple picks, it returns a tuple:

```rust
// Features: ["picker"]
@@@dispatcher!("test", EntryTest);
@@@pack!(ResultInfo = (String, u8, u32));
 
#[chain]
fn handle_test_entry(prev: EntryTest) -> Next {
    let (name, age, id) = prev
        .pick_or_default(&arg![name: String, 'n'])
        .pick_or_default(&arg![age: u8, 'a'])
        .pick_or_default(&arg![id: u32, 'I'])
        .unwrap();
 
    ResultInfo::new((name, age, id)).into()
}
```
 
> [!IMPORTANT]
> `Picker` is sensitive to parse order, especially for positional args — it parses sequentially. If you need to parse positional args, make sure all **flag arguments** are picked and consumed first.

## Handling Edge Cases with `pick_or_route`

As the saying goes: "never trust your users." To handle missing required params, type mismatches, etc., `pick_or_route` routes the chain to a dedicated error handler.

Here's a simple example:

```rust
// Features: ["picker", "extras"]
@@@use mingling::macros::buffer;
@@@use mingling::macros::route;
@@@dispatcher!("greet", EntryGreet);
@@@pack!(ResultName = String);
@@@pack!(ErrorNoName = ());
 
#[chain]
fn handle_greet_entry(prev: EntryGreet) -> Next {
    // Use route! macro to expand the Result<Value, Route>
    let name = route!(
        prev.pick_or_route(&arg![name: String, 'n'], || {
            ErrorNoName::default().to_chain()
        })
        .to_result()
    );
    ResultName::new(name).into()
}
 
#[renderer(buffer)]
fn render_greet(result: ResultName) {
    r_println!("Hello, {}!", *result);
}
```
 
With `pick_or_route`, `.to_result()` no longer returns the value directly, but `Result<Value, Route>`.

However, **Mingling**'s `extras` feature provides the `route!` macro for simplified expansion. It's not complex — it just reduces boilerplate:

```rust
// Features: ["picker", "extras"]
@@@ pack!(ErrorFail = ());
@@@ use mingling::macros::route;
@@@ use mingling::picker::IntoPicker;
@@@ fn func() -> mingling::ChainProcess<ThisProgram> {
@@@ let args: Vec<String> = vec![];
let name = route!(args.pick_or_route(&arg![String], || ErrorFail::new(()).to_chain()).to_result());
@@@ mingling::macros::empty_result!()
@@@ }
```
 
It expands to:

```rust
// Features: ["picker", "extras"]
@@@ pack!(ErrorFail = ());
@@@ use mingling::picker::IntoPicker;
@@@ fn func() -> mingling::ChainProcess<ThisProgram> {
@@@ let args: Vec<String> = vec![];
let name = match args.pick_or_route(&arg![String], || ErrorFail::new(()).to_chain()).to_result() {
    Ok(r) => r,
    Err(e) => return e,
};
@@@ mingling::macros::empty_result!()
@@@ }
```
 
## Post-processing Extracted Values

After picking user input with `pick`, you can use `post` to process it immediately:

```rust
// Features: ["picker"]
@@@dispatcher!("greet", EntryGreet);
@@@pack!(ResultName = String);
 
#[chain]
fn handle_greet_entry(prev: EntryGreet) -> Next {
    let name = prev
        .pick_or(&arg![name: String, 'n'], || "World".to_string())
        // Format immediately after picking --name
        .post(|name: String| {
            name.replace(['-', '_', '.'], " ")
                .to_lowercase()
                .trim()
                .to_string()
        })
        .unwrap();
 
    ResultName::new(name).into()
}
```
 
## Boolean Parsing

`Picker` parses booleans as **flags**: the flag being present means `true`.

```rust
// Features: ["picker"]
@@@use mingling::picker::value::Flag;
@@@dispatcher!("test", EntryTest);
@@@pack!(ResultDone = ());
 
#[chain]
fn handle_entry(prev: EntryTest) -> Next {
    // `--confirm` / `-C` present → true
    let _confirm: bool = *prev.pick(&arg![confirm: Flag, 'C']).unwrap();
    ResultDone::default().to_render()
}
```
 
> [!NOTE]
> For important confirmations, pair the flag with an explicit value check if the exact boolean semantics matter.

## Custom Pickable Types

You can make your types pickable by `Picker` using the `SinglePickable` trait — this is where `Picker`'s extensibility comes from.

```rust
// Features: ["picker"]
@@@use mingling::macros::buffer;
@@@use mingling::picker::{PickerArgResult, SinglePickable};
@@@use mingling::Flag;
#[derive(Default, Clone)]
pub struct Address {
    ip: String,
    port: u16,
}
 
impl SinglePickable for Address {
    fn pick_single(str: Option<&str>) -> PickerArgResult<Self> {
        let Some(raw) = str else {
            return PickerArgResult::NotFound;
        };
        let parts: Vec<&str> = raw.split(':').collect();
        let ip = parts.first().copied().unwrap_or_default().to_string();
        let port: u16 = match parts.get(1).and_then(|p| p.parse().ok()) {
            Some(p) => p,
            None => return PickerArgResult::NotFound,
        };
        PickerArgResult::Parsed(Address { ip, port })
    }
}
@@@dispatcher!("connect", EntryConnect);
@@@pack!(ResultConnected = Address);
 
#[chain]
fn handle_connect_entry(prev: EntryConnect) -> Next {
    let address: Address = prev.pick_or_default(&arg![Address]).unwrap();
    ResultConnected::new(address).into()
}
 
#[renderer(buffer)]
fn render_connected(addr: ResultConnected) {
    r_println!("Connected: IP: {} PORT: {}", addr.ip, addr.port);
}
```
 
Output:

```text
~# my-cli connect --addr 127.0.0.1:8080
Connected: IP: 127.0.0.1 PORT: 8080
```
 
## Implementing Pickable for Enums

To make an enum pickable, implement `SinglePickable` manually with a match on the input:

```rust
// Features: ["picker"]
@@@use mingling::macros::buffer;
@@@use mingling::picker::{PickerArgResult, SinglePickable};
@@@use mingling::EnumTag;
#[derive(Debug, Default, EnumTag)]
pub enum Fruits {
    #[default]
    Apple,
    Banana,
    Orange,
}
 
impl SinglePickable for Fruits {
    fn pick_single(str: Option<&str>) -> PickerArgResult<Self> {
        let Some(str) = str else {
            return PickerArgResult::NotFound;
        };
        let fruit = match str.to_lowercase().as_str() {
            "apple" => Self::Apple,
            "banana" => Self::Banana,
            "orange" => Self::Orange,
            _ => return PickerArgResult::NotFound,
        };
        PickerArgResult::Parsed(fruit)
    }
}
@@@dispatcher!("eat", EntryEat);
@@@pack!(ResultFruit = Fruits);
 
#[chain]
fn handle_eat_entry(prev: EntryEat) -> Next {
    let fruit: Fruits = prev.pick_or_default(&arg![Fruits]).unwrap();
    ResultFruit::new(fruit).into()
}
 
#[renderer(buffer)]
fn render_fruit(prev: ResultFruit) {
    r_println!("Picked fruit: {:?}", *prev);
}
```
 
That covers all the usages of `Picker`.

<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
