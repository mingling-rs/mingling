<h1 align="center">使用 Picker 完成参数解析</h1>
<p align="center">
    用 Picker 完成基本的参数解析
</p>

前面教程中我们都是手动从 `EntryGreet.inner`（`Vec<String>`）中提取参数。

```rust
@@@ fn main() {
@@@ let args : Vec<String> = vec![];
let name = args.first().cloned().unwrap_or_else(|| "World".to_string());
@@@ }
```
 
但是，对于参数较多的场景，这个方案就不够用了：Mingling 提供了 `Picker` —— 通过链式调用来提取和转换参数。

要启用 `Picker`，你需要修改 `Cargo.toml`

```toml
# Cargo.toml
[dependencies.mingling]
features = ["parser"]
```
 
好了，让我们看看 `Picker` 的写法：

```rust
// Features: ["parser"]
@@@dispatcher!("greet", CMDGreet => EntryGreet);
@@@pack!(ResultName = String);
 
#[chain]
fn handle_greet_entry(prev: EntryGreet) -> Next {
    let name = prev.pick_or((), "World").unpack();
    ResultName::new(name)
}
```
 
`AsPicker` 为所有可以转换为 `Vec<String>` 的类型实现了 `pick`、`pick_or`、`pick_or_route` 函数：它们可以语义化地从字符串列表中 **拾取 (Pick)** 参数，并转换为结构化数据。

对于上述示例中的代码：

```rust
// Features: ["parser"]
@@@dispatcher!("greet", CMDGreet => EntryGreet);
@@@pack!(ResultName = String);
@@@#[chain]
@@@fn handle_greet_entry(prev: EntryGreet) -> Next {
let name = prev.pick_or((), "World").unpack();
@@@ResultName::new(name)
@@@}
```
 
它的语义为：

```rust
// Features: ["parser"]
@@@dispatcher!("greet", CMDGreet => EntryGreet);
@@@pack!(ResultName = String);
@@@#[chain]
@@@fn handle_greet_entry(prev: EntryGreet) {
@@@let name: String =
   prev.pick_or((), "World").unpack();
// ~~~~ ~~~~~~~ ~~  ~~~~~~~  ~~~~~~~~
// |    |       |   |        |_ 解包为 String
// |    |       |   |__________ 默认值为 "World"
// |    |       |______________ 取出第一个位置参数（不指定标志）
// |    |______________________ 拾取或使用默认
// |___________________________ 从前一个输入中
@@@}
```
 
## 解析标志参数

若你的程序需要解析标志参数（例如 `greet --name Alice`），可以使用如下方式

```rust
// Features: ["parser"]
@@@dispatcher!("greet", CMDGreet => EntryGreet);
@@@pack!(ResultName = String);
 
#[chain]
fn handle_greet_entry(prev: EntryGreet) -> Next {
    let name = prev.pick_or(["--name", "-n"], "World").unpack();
    ResultName::new(name)
}
```
 
同理，它的语义为：

```rust
// Features: ["parser"]
@@@dispatcher!("greet", CMDGreet => EntryGreet);
@@@pack!(ResultName = String);
@@@#[chain]
@@@fn handle_greet_entry(prev: EntryGreet) {
@@@let name: String =
   prev.pick_or(["--name", "-n"], "World").unpack();
// ~~~~ ~~~~~~~ ~~~~~~~~~~~~~~~~  ~~~~~~~  ~~~~~~~~
// |    |       |                 |        |_ 解包为 String
// |    |       |                 |__________ 默认值为 "World"
// |    |       |____________________________ 取出 "--name" 或 "-n" 后面的参数
// |    |____________________________________ 拾取或使用默认
// |_________________________________________ 从前一个输入中
@@@}
```
 
## 关于 `.unpack()`

你可能注意到了，`Picker` 在命令解析的最后，会执行一个 `.unpack()` 函数，它的作用是将前面解析出来的结果，转换为结构化信息。

对于只拾取了一次的数据来说，`.unpack()` 会返回单个数据，而对于多次拾取，`Picker` 则会返回元组：

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
 
    ResultInfo::new((name, age, id))
}
```
 
> [!IMPORTANT]
> `Picker` 对解析顺序极其敏感，特别是位置参数：因为它是顺序解析的。若你需要解析位置参数，请确保解析前已拾取并消费所有 **标志参数**。

## 使用 `pick_or_route` 处理边界情况

就像那句老话："永远不要相信你的用户"。为了应对必要参数缺失、输入类型不匹配等错误情况，`pick_or_route` 能将执行链路由到专门的错误处理类型上。

先来看一个简单示例

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
 
    // 使用 route! 宏展开 pick_result
    let name = route!(pick_result);
    ResultName::new(name).into()
}
 
#[renderer]
fn render_no_name(_prev: ErrorNoName) {
    r_println!("Error: No name provided.");
}
 
#[renderer]
fn render_name(prev: ResultName) {
    r_println!("Hello, {}!", *prev);
}
```
 
若使用 `pick_or_route`，写法会变得相对复杂：因为 `.unpack()` 不再直接返回参数，而是 `Result<Value, Route>`。

不过 **Mingling** 的 `extra_macros` 特性提供了简化展开的宏 `route!`，它不复杂，只是省略了一部分样板代码：

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
 
它展开为：

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
 
## 提取值的后处理

在您使用 `pick` 提取了用户输入后，可以使用 `after` 立刻处理该参数

```rust
// Features: ["parser"]
@@@dispatcher!("greet", CMDGreet => EntryGreet);
@@@pack!(ResultName = String);
 
#[chain]
fn handle_greet_entry(prev: EntryGreet) -> Next {
    let name = prev
        .pick_or(["--name", "-n"], "World")
        // 在提取出 --name 后，立刻格式化
        .after(|name: String| {
            name.replace(['-', '_', '.'], " ")
                .to_lowercase()
                .trim()
                .to_string()
        })
        .unpack();
 
    ResultName::new(name)
}
```
 
同样，你可以使用 `after_or_route` 来处理输入参数的格式错误

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
fn render_name_too_long(prev: ErrorNameTooLong) {
    let len = *prev;
    r_println!("Error: name too long (length: {} > 32)", len);
}
 
#[renderer]
fn render_name(prev: ResultName) {
    r_println!("Hello, {}!", *prev);
}
```
 
## 布尔值解析

`Picker` 当然也可以解析布尔类型，但是布尔类型分为显式和隐式模式：

| 模式 | 格式                                |
| ---- | ----------------------------------- |
| 隐式 | `--confirmed`                       |
| 显式 | `--confirm true` 或 `--confirm yes` |

- 使用 `.pick::<bool>(flag)` 时，采用隐式解析：只要标志存在即为 `true`
- 使用 `.pick::<Yes>(flag)` 或 `.pick::<True>(flag)` 时，采用显式解析

一般来说使用隐式解析即可，但在处理重要的确认行为时，显式逻辑更符合语义。

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
 
## 特殊用法：`usize` 解析

**Mingling** 为 `usize` 提供了一个特殊的用法：解析类似 `25G`、`32mib` 等字样

```rust
// Features: ["parser"]
 
#[test]
fn parse_size() {
    let vec = vec!["--size".to_string(), "25mib".to_string()];
    let size: usize = vec.pick(["--size", "-S"]).unpack();
    assert_eq!(size, 25 * 1024 * 1024);
}
```
 
## 自定义可解析类型

你可以使用 `Pickable` trait 使你的类型支持被 `Picker` 解析，这也是 `Picker` 拓展性的来源

```rust
// Features: ["parser"]
@@@use mingling::parser::{Pickable, Argument};
@@@use mingling::Flag;
#[derive(Default)]
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
    ResultConnected::new(address)
}
 
#[renderer]
fn render_connected(prev: ResultConnected) {
    let addr = prev.inner;
    r_println!("Connected: IP: {} PORT: {}", addr.ip, addr.port);
}
```
 
执行效果如下：

```text
~# my-cli connect --addr 127.0.0.1:8080
Connected: IP: 127.0.0.1 PORT: 8080
```
 
## 自动为枚举实现 Pickable

要为枚举类型实现 `Pickable`，只需该枚举实现了 `EnumTag`，然后为其实现 `PickableEnum` 即可

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
    ResultFruit::new(fruit)
}
 
#[renderer]
fn render_fruit(prev: ResultFruit) {
    r_println!("Picked fruit: {:?}", *prev);
}
```
 
以上便是 `Picker` 的所有用法。

<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
