<h1 align="center">使用 Picker 完成参数解析</h1>
<p align="center">
    用 Picker 完成基本的参数解析
</p>

前面教程中我们都是手动从 `EntryGreet.0`（`Vec<String>`）中提取参数。

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
features = ["picker"]
```
 
好了，让我们看看 `Picker` 的写法：

```rust
// Features: ["picker"]
@@@dispatcher!("greet", EntryGreet);
@@@#[derive(Grouped, Wrap)]
@@@pub struct ResultName(String);
 
#[chain]
fn handle_greet_entry(prev: EntryGreet) -> Next {
    let name = prev
        .pick_or(&arg![String], || "World".to_string())
        .unwrap();
    ResultName(name).into()
}
```
 
`EntryPicker` 为所有入口类型实现了 `pick`、`pick_or`、`pick_or_default` 和 `pick_or_route` 函数：它们可以通过 `arg!` 宏声明要拾取的内容，语义化地从字符串列表中 **拾取 (Pick)** 参数，并转换为结构化数据。

对于上述示例中的代码：

```rust
// Features: ["picker"]
@@@dispatcher!("greet", EntryGreet);
@@@#[derive(Grouped, Wrap)]
@@@pub struct ResultName(String);
@@@#[chain]
@@@fn handle_greet_entry(prev: EntryGreet) -> Next {
let name = prev
    .pick_or(&arg![String], || "World".to_string())
    .unwrap();
@@@ResultName(name).into()
@@@}
```
 
它的语义为：

```rust
// Features: ["picker"]
@@@dispatcher!("greet", EntryGreet);
@@@#[derive(Grouped, Wrap)]
@@@pub struct ResultName(String);
@@@#[chain]
@@@fn handle_greet_entry(prev: EntryGreet) {
@@@let name: String =
   prev.pick_or(&arg![String], || "World".to_string()).unwrap();
// ~~~~ ~~~~~~~ ~~~~~~~~~~~~  ~~~~~~~~~~~~~~~~~~~~~~~  ~~~~~~
// |    |       |             |                        |_ 解包为 String
// |    |       |             |__________________________ 默认值为 "World"
// |    |       |________________________________________ 取出第一个位置参数（声明为 String）
// |    |________________________________________________ 拾取或使用默认
// |_____________________________________________________ 从前一个输入中
@@@}
```
 
## 解析标志参数

若你的程序需要解析标志参数（例如 `greet --name Alice`），可以在 `arg!` 中声明一个具名标志：

```rust
// Features: ["picker"]
@@@dispatcher!("greet", EntryGreet);
@@@#[derive(Grouped, Wrap)]
@@@pub struct ResultName(String);
 
#[chain]
fn handle_greet_entry(prev: EntryGreet) -> Next {
    let name = prev
        .pick_or(&arg![name: String, 'n'], || "World".to_string())
        .unwrap();
    ResultName(name).into()
}
```
 
`arg!` 宏会从字段名推导长标志名（`--name`），`'n'` 则添加短别名（`-n`）。

同理，它的语义为：

```rust
// Features: ["picker"]
@@@dispatcher!("greet", EntryGreet);
@@@#[derive(Grouped, Wrap)]
@@@pub struct ResultName(String);
@@@#[chain]
@@@fn handle_greet_entry(prev: EntryGreet) {
@@@let name: String =
   prev.pick_or(&arg![name: String, 'n'], || "World".to_string()).unwrap();
// ~~~~ ~~~~~~~ ~~~~~~~~~~~~~~~~~~~~~~~~~  ~~~~~~~~~~~~~~~~~~~~~  ~~~~~~
// |    |       |                          |                      |_ 解包为 String
// |    |       |                          |________________________ 默认值为 "World"
// |    |       |___________________________________________________ 取出 "--name" 或 "-n" 后面的参数
// |    |___________________________________________________________ 拾取或使用默认
// |________________________________________________________________ 从前一个输入中
@@@}
```
 
## 关于 `.unwrap()` 与 `route!`

你可能注意到了，`Picker` 在命令解析的最后，会执行一个 `.unwrap()`（或 `route!`）函数，它的作用是将前面解析出来的结果，转换为结构化信息。

对于只拾取了一次的数据来说，`.unwrap()` 会返回单个数据，而对于多次拾取，`Picker` 则会返回元组：

```rust
// Features: ["picker"]
@@@dispatcher!("test", EntryTest);
@@@#[derive(Grouped, Wrap)]
@@@pub struct ResultInfo((String, u8, u32));
 
#[chain]
fn handle_test_entry(prev: EntryTest) -> Next {
    let (name, age, id) = prev
        .pick_or_default(&arg![name: String, 'n'])
        .pick_or_default(&arg![age: u8, 'a'])
        .pick_or_default(&arg![id: u32, 'I'])
        .unwrap();
 
    ResultInfo((name, age, id)).into()
}
```
 
> [!IMPORTANT]
> `Picker` 对解析顺序极其敏感，特别是位置参数：因为它是顺序解析的。若你需要解析位置参数，请确保解析前已拾取并消费所有 **标志参数**。

## 使用 `pick_or_route` 处理边界情况

就像那句老话："永远不要相信你的用户"。为了应对必要参数缺失、输入类型不匹配等错误情况，`pick_or_route` 能将执行链路由到专门的错误处理类型上。

先来看一个简单示例

```rust
// Features: ["picker", "extras"]
@@@use mingling::macros::buffer;
@@@use mingling::macros::route;
@@@dispatcher!("greet", EntryGreet);
@@@#[derive(Grouped, Wrap)]
@@@pub struct ResultName(String);
@@@#[derive(Grouped, Wrap, Default)]
@@@pub struct ErrorNoName(());
 
#[chain]
fn handle_greet_entry(prev: EntryGreet) -> Next {
    // 使用 route! 宏展开 Result<Value, Route>
    let name = route!(
        prev.pick_or_route(&arg![name: String, 'n'], || {
            ErrorNoName::default().to_chain()
        })
        .to_result()
    );
    ResultName(name).into()
}
 
#[renderer(buffer)]
fn render_greet(result: ResultName) {
    r_println!("Hello, {}!", *result);
}
```
 
若使用 `pick_or_route`，`.to_result()` 不再直接返回参数，而是 `Result<Value, Route>`。

不过 **Mingling** 的 `extras` 特性提供了简化展开的宏 `route!`，它不复杂，只是省略了一部分样板代码：

```rust
// Features: ["picker", "extras"]
@@@ #[derive(Grouped, Wrap)]
@@@ pub struct ErrorFail(());
@@@ use mingling::macros::route;
@@@ use mingling::picker::IntoPicker;
@@@ fn func() -> mingling::ChainProcess<ThisProgram> {
@@@ let args: Vec<String> = vec![];
let name = route!(args.pick_or_route(&arg![String], || ErrorFail(()).to_chain()).to_result());
@@@ mingling::macros::empty_result!()
@@@ }
```
 
它展开为：

```rust
// Features: ["picker", "extras"]
@@@ #[derive(Grouped, Wrap)]
@@@ pub struct ErrorFail(());
@@@ use mingling::picker::IntoPicker;
@@@ fn func() -> mingling::ChainProcess<ThisProgram> {
@@@ let args: Vec<String> = vec![];
let name = match args.pick_or_route(&arg![String], || ErrorFail(()).to_chain()).to_result() {
    Ok(r) => r,
    Err(e) => return e,
};
@@@ mingling::macros::empty_result!()
@@@ }
```
 
## 提取值的后处理

在您使用 `pick` 提取了用户输入后，可以使用 `post` 立刻处理该参数

```rust
// Features: ["picker"]
@@@dispatcher!("greet", EntryGreet);
@@@#[derive(Grouped, Wrap)]
@@@pub struct ResultName(String);
 
#[chain]
fn handle_greet_entry(prev: EntryGreet) -> Next {
    let name = prev
        .pick_or(&arg![name: String, 'n'], || "World".to_string())
        // 在提取出 --name 后，立刻格式化
        .post(|name: String| {
            name.replace(['-', '_', '.'], " ")
                .to_lowercase()
                .trim()
                .to_string()
        })
        .unwrap();
 
    ResultName(name).into()
}
```
 
## 布尔值解析

`Picker` 将布尔值解析为**标志**：标志存在即为 `true`。

```rust
// Features: ["picker"]
@@@use mingling::picker::value::Flag;
@@@dispatcher!("test", EntryTest);
@@@#[derive(Grouped, Wrap, Default)]
@@@pub struct ResultDone(());
 
#[chain]
fn handle_entry(prev: EntryTest) -> Next {
    // `--confirm` / `-C` 存在 → true
    let _confirm: bool = *prev.pick(&arg![confirm: Flag, 'C']).unwrap();
    ResultDone::default().to_render()
}
```
 
> [!NOTE]
> 对于重要的确认行为，如果精确的布尔语义很关键，请将标志与显式的值检查配合使用。

## 自定义可解析类型

你可以使用 `SinglePickable` trait 使你的类型支持被 `Picker` 解析，这也是 `Picker` 拓展性的来源

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
@@@#[derive(Grouped, Wrap)]
@@@pub struct ResultConnected(Address);
 
#[chain]
fn handle_connect_entry(prev: EntryConnect) -> Next {
    let address: Address = prev.pick_or_default(&arg![Address]).unwrap();
    ResultConnected(address).into()
}
 
#[renderer(buffer)]
fn render_connected(addr: ResultConnected) {
    r_println!("Connected: IP: {} PORT: {}", addr.ip, addr.port);
}
```
 
执行效果如下：

```text
~# my-cli connect --addr 127.0.0.1:8080
Connected: IP: 127.0.0.1 PORT: 8080
```
 
## 为枚举实现 Pickable

要让枚举支持 `Picker` 解析，可以手写 `SinglePickable`，用 match 匹配输入：

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
@@@#[derive(Grouped, Wrap)]
@@@pub struct ResultFruit(Fruits);
 
#[chain]
fn handle_eat_entry(prev: EntryEat) -> Next {
    let fruit: Fruits = prev.pick_or_default(&arg![Fruits]).unwrap();
    ResultFruit(fruit).into()
}
 
#[renderer(buffer)]
fn render_fruit(prev: ResultFruit) {
    r_println!("Picked fruit: {:?}", *prev);
}
```
 
以上便是 `Picker` 的所有用法。

<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
