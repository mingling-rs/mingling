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
features = ["parser"]
```
 
Now let's see how `Picker` is written:

```rust
// Features: ["parser"]
@@@dispatcher!("greet", EntryGreet);
@@@pack!(ResultName = String);
 
#[chain]
fn handle_greet_entry(prev: EntryGreet) -> Next {
    let name = prev.pick_or((), "World").unpack();
    ResultName::new(name).into()
}
```
 
`AsPicker` implements `pick`, `pick_or`, and `pick_or_route` for all types convertible to `Vec<String>`. These functions semantically **pick** params from the string list and convert them into structured data.

For the code above:

```rust
// Features: ["parser"]
@@@dispatcher!("greet", EntryGreet);
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
@@@dispatcher!("greet", EntryGreet);
@@@pack!(ResultName = String);
@@@#[chain]
@@@fn handle_greet_entry(prev: EntryGreet) {
@@@let name: String =
   prev.pick_or((), "World").unpack();
// ~~~~ ~~~~~~~ ~~  ~~~~~~~  ~~~~~~~~
// |    |       |   |        |_ unpack as String
// |    |       |   |__________ default value "World"
// |    |       |______________ pick the first positional arg (no flag)
// |    |______________________ pick or use default
// |___________________________ from the previous input
@@@}
```
 
## Parsing Flag Arguments

If your program needs to parse flag arguments (e.g. `greet --name Alice`), do this:

```rust
// Features: ["parser"]
@@@dispatcher!("greet", EntryGreet);
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
@@@dispatcher!("greet", EntryGreet);
@@@pack!(ResultName = String);
@@@#[chain]
@@@fn handle_greet_entry(prev: EntryGreet) {
@@@let name: String =
   prev.pick_or(["--name", "-n"], "World").unpack();
// ~~~~ ~~~~~~~ ~~~~~~~~~~~~~~~~  ~~~~~~~  ~~~~~~~~
// |    |       |                 |        |_ unpack as String
// |    |       |                 |__________ default value "World"
// |    |       |____________________________ pick the value after "--name" or "-n"
// |    |____________________________________ pick or use default
// |_________________________________________ from the previous input
@@@}
```
 
## About `.unpack()`

You may have noticed that `Picker` calls `.unpack()` at the end of parsing. It converts the collected results into structured info.

For a single pick, `.unpack()` returns the value directly; for multiple picks, it returns a tuple:

```rust
// Features: ["parser"]
@@@dispatcher!("test", EntryTest);
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
> `Picker` is sensitive to parse order, especially for positional args — it parses sequentially. If you need to parse positional args, make sure all **flag arguments** are picked and consumed first.

## Handling Edge Cases with `pick_or_route`

As the saying goes: "never trust your users." To handle missing required params, type mismatches, etc., `pick_or_route` routes the chain to a dedicated error handler.

Here's a simple example:

```rust
// Features: ["parser", "extras"]
@@@use mingling::macros::buffer;
@@@use mingling::macros::route;
@@@dispatcher!("greet", EntryGreet);
@@@pack!(ResultName = String);
@@@pack!(ErrorNoName = ());
 
#[chain]
fn handle_greet_entry(prev: EntryGreet) -> Next {
    let pick_result = prev
        .pick_or_route(["--name", "-n"], ErrorNoName::default())
        .unpack();
 
    // Use route! macro to expand pick_result
    let name = route!(pick_result);
    ResultName::new(name).into()
}
 
#[renderer(buffer)]
fn render_greet(result: ResultName) {
    r_println!("Hello, {}!", *result);
}
```
 
With `pick_or_route`, the code becomes more involved: `.unpack()` no longer returns the value directly, but `Result<Value, Route>`.

However, **Mingling**'s `extras` feature provides the `route!` macro for simplified expansion. It's not complex — it just reduces boilerplate:

```rust
// Features: ["parser", "extras"]
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
// Features: ["parser", "extras"]
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

After picking user input with `pick`, you can use `after` to process it immediately:

```rust
// Features: ["parser"]
@@@dispatcher!("greet", EntryGreet);
@@@pack!(ResultName = String);
 
#[chain]
fn handle_greet_entry(prev: EntryGreet) -> Next {
    let name = prev
        .pick_or(["--name", "-n"], "World")
        // Format immediately after picking --name
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
 
Similarly, you can use `after_or_route` to handle input format errors:

```rust
// Features: ["parser", "extras"]
@@@use mingling::macros::buffer;
@@@use mingling::macros::route;
@@@dispatcher!("greet", EntryGreet);
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
 
#[renderer(buffer)]
fn render_name_too_long(prev: ErrorNameTooLong) {
    let len = *prev;
    r_println!("Error: name too long (length: {} > 32)", len);
}
 
#[renderer(buffer)]
fn render_name(prev: ResultName) {
    r_println!("Hello, {}!", *prev);
}
```
 
## Boolean Parsing

`Picker` can also parse booleans, in two modes:

| Mode     | Format                              |
| -------- | ----------------------------------- |
| Implicit | `--confirmed`                       |
| Explicit | `--confirm true` or `--confirm yes` |

- `.pick::<bool>(flag)` uses implicit mode: the flag being present means `true`
- `.pick::<Yes>(flag)` or `.pick::<True>(flag)` uses explicit mode

Implicit mode is generally sufficient, but for important confirmations, explicit logic is more idiomatic.

```rust
// Features: ["parser"]
@@@use mingling::parser::Yes;
@@@dispatcher!("test", EntryTest);
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

**Mingling** provides a special `usize` feature: parsing strings like `25G`, `32mib`, etc.

```rust
// Features: ["parser"]
 
#[test]
fn parse_size() {
    let vec = vec!["--size".to_string(), "25mib".to_string()];
    let size: usize = vec.pick(["--size", "-S"]).unpack();
    assert_eq!(size, 25 * 1024 * 1024);
}
```
 
## Custom Pickable Types

You can make your types pickable by `Picker` using the `Pickable` trait — this is where `Picker`'s extensibility comes from.

```rust
// Features: ["parser"]
@@@use mingling::macros::buffer;
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
@@@dispatcher!("connect", EntryConnect);
@@@pack!(ResultConnected = Address);
 
#[chain]
fn handle_connect_entry(prev: EntryConnect) -> Next {
    let address: Address = prev.pick("--addr").unpack();
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
 
## Auto-implementing Pickable for Enums

To make an enum `Pickable`, just implement `EnumTag` on it, then implement `PickableEnum`:

```rust
// Features: ["parser"]
@@@use mingling::macros::buffer;
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
@@@dispatcher!("eat", EntryEat);
@@@pack!(ResultFruit = Fruits);
 
#[chain]
fn handle_eat_entry(prev: EntryEat) -> Next {
    let fruit: Fruits = prev.pick("--fruit").unpack();
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
