<h1 align="center">Parsing Arguments with Picker</h1>
<p align="center">
    Use Picker to perform basic argument parsing
</p>

In previous tutorials, we extracted args manually from `EntryGreet.inner` (`Vec<String>`).

```rust
@@@ fn main() {
@@@ let args : Vec<String> = vec![];
let name = args.first().cloned().unwrap_or_else(|| "World".to_string());
@@@ }
```
 
But this approach doesn't scale well for many params. Mingling provides `Picker` — a chaining API to extract and convert args.

To enable `Picker`, update your `Cargo.toml`:

```toml
# Cargo.toml
[dependencies.mingling]
features = ["parser"]
```
 
Now let's look at `Picker` in action:

```rust
// Features: ["parser"]
@@@dispatcher!("greet", CMDGreet => EntryGreet);
@@@pack!(ResultName = String);
 
#[chain]
fn handle_greet_entry(prev: EntryGreet) -> Next {
    let name = prev.pick_or((), "World").unpack();
    ResultName::new(name).into()
}
```
 
`AsPicker` implements `pick`, `pick_or`, and `pick_or_route` for any type that can convert to `Vec<String>`: they semantically **pick** args from a string list and convert them to structured data.

Breaking down the example above:

```rust
// Features: ["parser"]
@@@dispatcher!("greet", CMDGreet => EntryGreet);
@@@pack!(ResultName = String);
@@@#[chain]
@@@fn handle_greet_entry(prev: EntryGreet) -> Next {
let name = prev.pick_or((), "World").unpack();
@@@ResultName::new(name).into()
@@@}
```
 
Its semantics are:

```rust
// Features: ["parser"]
@@@dispatcher!("greet", CMDGreet => EntryGreet);
@@@pack!(ResultName = String);
@@@#[chain]
@@@fn handle_greet_entry(prev: EntryGreet) {
@@@let name: String =
   prev.pick_or((), "World").unpack();
// ~~~~ ~~~~~~~ ~~  ~~~~~~~  ~~~~~~~~
// |    |       |   |        |_ unpack to String
// |    |       |   |__________ default value "World"
// |    |       |______________ pick the first positional arg (no flag)
// |    |______________________ pick or use default
// |___________________________ from previous input
@@@}
```
 
## Parsing Flag Args

If your program needs to parse flag args (e.g., `greet --name Alice`), do this:

```rust
// Features: ["parser"]
@@@dispatcher!("greet", CMDGreet => EntryGreet);
@@@pack!(ResultName = String);
 
#[chain]
fn handle_greet_entry(prev: EntryGreet) -> Next {
    let name = prev.pick_or(["--name", "-n"], "World").unpack();
    ResultName::new(name).into()
}
```
 
Its semantics:

```rust
// Features: ["parser"]
@@@dispatcher!("greet", CMDGreet => EntryGreet);
@@@pack!(ResultName = String);
@@@#[chain]
@@@fn handle_greet_entry(prev: EntryGreet) {
@@@let name: String =
   prev.pick_or(["--name", "-n"], "World").unpack();
// ~~~~ ~~~~~~~ ~~~~~~~~~~~~~~~~  ~~~~~~~  ~~~~~~~~
// |    |       |                 |        |_ unpack to String
// |    |       |                 |__________ default value "World"
// |    |       |____________________________ pick arg after "--name" or "-n"
// |    |____________________________________ pick or use default
// |_________________________________________ from previous input
@@@}
```
 
## About `.unpack()`

You may have noticed that `Picker` calls `.unpack()` at the end of parsing. It converts the accumulated parse results into structured info.

For a single pick, `.unpack()` returns a single value; for multiple picks, it returns a tuple:

```rust
// Features: ["parser"]
@@@dispatcher!("test", CMDTest => EntryTest);
@@@pack!(ResultInfo = (String, u8, u32));
 
#[chain]
fn handle_test_entry(prev: EntryTest) -> Next {
    let (name, age, id) = prev
        .pick::<String>(["--name", "-n"])
        .pick::<u8>(["--age", "-a"])
        .pick::<u32>(["--id", "-I"])
        .unpack();
 
    ResultInfo::new((name, age, id)).into()
}
```
 
> [!IMPORTANT]
> `Picker` is very sensitive to parse order, esp. for positional args (they're parsed sequentially). If you need to parse positional args, make sure all **flag args** have been picked and consumed first.

## Handling Edge Cases with `pick_or_route`

As the old saying goes: "Never trust your users." To handle missing required args, type mismatches, etc., `pick_or_route` routes the execution chain to a dedicated error-handling type.

A simple example:

```rust
// Features: ["parser", "extra_macros"]
@@@use mingling::macros::route;
@@@dispatcher!("greet", CMDGreet => EntryGreet);
@@@pack!(ResultName = String);
@@@pack!(ErrorNoName = ());
 
#[chain]
fn handle_greet_entry(prev: EntryGreet) -> Next {
    let pick_result = prev
        .pick_or_route(["--name", "-n"], ErrorNoName::default())
        .unpack();
 
    // Use route! macro to unpack pick_result
    let name = route!(pick_result);
    ResultName::new(name).into()
}
 
#[renderer]
fn render_no_name(_prev: ErrorNoName) -> RenderResult {
    let mut r = RenderResult::new();
    writeln!(r, "Error: No name provided.").ok();
    r
}
 
#[renderer]
fn render_name(prev: ResultName) -> RenderResult {
    let mut r = RenderResult::new();
    writeln!(r, "Hello, {}!", *prev).ok();
    r
}
```
 
With `pick_or_route`, the code gets a bit more complex: `.unpack()` no longer returns the value directly, but `Result<Value, Route>`.

However, Mingling's `extra_macros` feature provides the `route!` macro to simplify unwrapping — it just omits some boilerplate:

```rust
// Features: ["parser", "extra_macros"]
@@@ pack!(ErrorFail = ());
@@@ use mingling::macros::route;
@@@ fn func() -> mingling::ChainProcess<ThisProgram> {
@@@ let args: Vec<String> = vec![];
@@@ let pick_result = args.pick_or_route::<String, _>((), ErrorFail::new(())).unpack();
let name = route!(pick_result);
@@@ mingling::macros::empty_result!()
@@@ }
```
 
It expands to:

```rust
// Features: ["parser", "extra_macros"]
@@@ pack!(ErrorFail = ());
@@@ fn func() -> mingling::ChainProcess<ThisProgram> {
@@@ let args: Vec<String> = vec![];
@@@ let pick_result = args.pick_or_route::<String, _>((), ErrorFail::new(())).unpack();
let name = match pick_result {
    Ok(r) => r,
    Err(e) => return e.to_chain(),
};
@@@ mingling::macros::empty_result!()
@@@ }
```
 
## Post-processing Extracted Values

After picking user input, you can use `after` to process the value immediately:

```rust
// Features: ["parser"]
@@@dispatcher!("greet", CMDGreet => EntryGreet);
@@@pack!(ResultName = String);
 
#[chain]
fn handle_greet_entry(prev: EntryGreet) -> Next {
    let name = prev
        .pick_or(["--name", "-n"], "World")
        // Format immediately after extracting --name
        .after(|name: String| {
            name.replace(['-', '_', '.'], " ")
                .to_lowercase()
                .trim()
                .to_string()
        })
        .unpack();
 
    ResultName::new(name).into()
}
```
 
Similarly, use `after_or_route` to handle format errors in input args:

```rust
// Features: ["parser", "extra_macros"]
@@@use mingling::macros::route;
@@@dispatcher!("greet", CMDGreet => EntryGreet);
@@@pack!(ResultName = String);
@@@pack!(ErrorNameTooLong = usize);
 
#[chain]
fn handle_greet_entry(prev: EntryGreet) -> Next {
    let pick_result = prev
        .pick_or(["--name", "-n"], "World")
        .after_or_route(|name: &String| {
            if name.len() < 32 {
                Ok(name.clone())
            } else {
                Err(ErrorNameTooLong::new(name.len()))
            }
        })
        .unpack();
    let name = route!(pick_result);
 
    ResultName::new(name).into()
}
 
#[renderer]
fn render_name_too_long(prev: ErrorNameTooLong) -> RenderResult {
    let mut r = RenderResult::new();
    let len = *prev;
    writeln!(r, "Error: name too long (length: {} > 32)", len).ok();
    r
}
 
#[renderer]
fn render_name(prev: ResultName) -> RenderResult {
    let mut r = RenderResult::new();
    writeln!(r, "Hello, {}!", *prev).ok();
    r
}
```
 
## Boolean Parsing

`Picker` can parse bools too, with two modes: implicit and explicit.

| Mode     | Format                              |
| -------- | ----------------------------------- |
| Implicit | `--confirmed`                       |
| Explicit | `--confirm true` or `--confirm yes` |

- Using `.pick::<bool>(flag)` → implicit: flag present means `true`
- Using `.pick::<Yes>(flag)` or `.pick::<True>(flag)` → explicit

Implicit is fine for most cases, but for important confirmations, explicit logic is more semantic.

```rust
// Features: ["parser"]
@@@use mingling::parser::Yes;
@@@dispatcher!("test", CMDTest => EntryTest);
@@@pack!(ResultDone = ());
 
#[chain]
fn handle_entry(prev: EntryTest) -> Next {
@@@ let prev1 = prev.clone();
    let _confirmed: bool = prev.pick::<Yes>(()).unpack().is_yes();
@@@ let prev = prev1;
    let _confirm: bool = prev.pick::<bool>(["--confirm", "-C"]).unpack();
    ResultDone::default().to_render()
}
```
 
## Special Usage: `usize` Parsing

Mingling provides a special `usize` feature: parsing strings like `25G`, `32mib`, etc.

```rust
// Features: ["parser"]
 
#[test]
fn parse_size() {
    let vec = vec!["--size".to_string(), "25mib".to_string()];
    let size: usize = vec.pick(["--size", "-S"]).unpack();
    assert_eq!(size, 25 * 1024 * 1024);
}
```
 
## Custom Parseable Types

Implement the `Pickable` trait to make your type parseable by `Picker` — this is where Picker's extensibility comes from.

```rust
// Features: ["parser"]
@@@use mingling::parser::{Pickable, Argument};
@@@use mingling::Flag;
#[derive(Default, Clone)]
pub struct Address {
    ip: String,
    port: u16,
}
 
impl Pickable for Address {
    type Output = Self;
    fn pick(args: &mut Argument, flag: Flag) -> Option<Self::Output> {
        let raw = args.pick_argument(flag)?;
        let parts: Vec<&str> = raw.split(':').collect();
        let ip = parts.first()?.to_string();
        let port: u16 = parts.get(1)?.parse().ok()?;
        Some(Address { ip, port })
    }
}
@@@dispatcher!("connect", CMDConnect => EntryConnect);
@@@pack!(ResultConnected = Address);
 
#[chain]
fn handle_connect_entry(prev: EntryConnect) -> Next {
    let address: Address = prev.pick("--addr").unpack();
    ResultConnected::new(address).into()
}
 
#[renderer]
fn render_connected(prev: ResultConnected) -> RenderResult {
    let mut r = RenderResult::new();
    let addr = prev.inner;
    writeln!(r, "Connected: IP: {} PORT: {}", addr.ip, addr.port).ok();
    r
}
```
 
Output:

```text
~# my-cli connect --addr 127.0.0.1:8080
Connected: IP: 127.0.0.1 PORT: 8080
```
 
## Auto-implementing Pickable for Enums

To implement `Pickable` for an enum, just make it implement `EnumTag`, then implement `PickableEnum`:

```rust
// Features: ["parser"]
@@@use mingling::parser::PickableEnum;
@@@use mingling::EnumTag;
#[derive(Debug, Default, EnumTag)]
pub enum Fruits {
    #[default]
    Apple,
    Banana,
    Orange,
}
 
impl PickableEnum for Fruits {}
@@@dispatcher!("eat", CMDEat => EntryEat);
@@@pack!(ResultFruit = Fruits);
 
#[chain]
fn handle_eat_entry(prev: EntryEat) -> Next {
    let fruit: Fruits = prev.pick("--fruit").unpack();
    ResultFruit::new(fruit).into()
}
 
#[renderer]
fn render_fruit(prev: ResultFruit) -> RenderResult {
    let mut r = RenderResult::new();
    writeln!(r, "Picked fruit: {:?}", *prev).ok();
    r
}
```
 
That covers all the features of `Picker`.

<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
