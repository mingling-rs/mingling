<h1 align="center">创建子命令 greet</h1>
<p align="center">
    通过 <code>#[command]</code> 定义子命令：<code>greet</code>
</p>

回顾上页中的示例：

```rust
use mingling::prelude::*;
 
fn main() {
    let mut program = ThisProgram::new();
    program.exec_and_exit();
}
 
gen_program!();
```
 
它执行后是这样的：

```bash,simulation
~# cargo run
~#
```
 
可以看见，它什么都没输出，这是因为我们还没有为该程序定义任何子命令和行为，mingling 在这种情况下是 **完全静默** 的，而本页我们来创建第一条子命令。

## 如何创建子命令？

在 Mingling 中，创建一条子命令是非常快速的：
你只需要在函数上增加一条属性宏 `#[command]`，它就会被识别为一条命令的入口点，并被自动注册到 `crate::ThisProgram` 中

```rust
use mingling::macros::command;
 
#[command] // 标记该函数是一条命令
pub fn command() { // 函数名即命令名
    // ... command 命令的行为
}
```
 
你的函数名将会被映射为命令名，规则如下：

| 函数名        | 节点名         | 实际执行时       |
| ------------- | -------------- | ---------------- |
| fn mycommand  | `"mycommand"`  | `cli mycommand`  |
| fn my_command | `"my.command"` | `cli my command` |

如果你需要命令名称里包含符号（例如 `-`），便需要使用 `#[command(node = "node.name")]` 来精确指定节点名称，例如：

```rust
use mingling::macros::command;
 
#[command(node = "my-command")]
pub fn my_command() {
    // ... my-command 命令的行为
    println!("My Command!");
}
```
 
他执行后就是这样的：

```bash,simulation
~# cargo run -- my-command
My Command!
```
 
---

好了，让我们开始编写第一条子命令吧！

---

## 1. 设计命令行为

在编写命令前，我们先设想一下该命令执行后的样子：

首先，我希望无参数的情况下，它能够输出 `"Hello, Mingling!"`，有参数的时候则会输出我指定的内容，就像这样：

```bash,simulation
~# cargo run -- greet
Hello, Mingling!
~# cargo run -- greet Alice
Hello, Alice!
```
 
## 2. 编写命令行为

那么，我们首先添加子命令 `greet`：

```rust,diff
use mingling::prelude::*;
 
fn main() {
    let mut program = ThisProgram::new();
    program.exec_and_exit();
}
 
+ #[command]
+ pub fn greet() {
+
+ }
 
gen_program!();
```
 
然后，编写具体的解析行为

```rust,diff
#[command]
- pub fn greet() {
+ pub fn greet(args: Vec<String>) { // 增加一个参数输入 `Vec<String>`
+     let name = args
+         .first() // 获取命令名之后的第一个参数
+         .map(|s| s.as_str()) // 将该参数转换为字符串切片
+         .unwrap_or("Mingling"); // 如果没有参数，则使用默认值 `"Mingling"`
+     println!("Hello, {}!", name); // 输出问候语
}
```
 
完整代码：

```rust
use mingling::prelude::*;
 
fn main() {
    let mut program = ThisProgram::new();
    program.exec_and_exit();
}
 
#[command]
pub fn greet(args: Vec<String>) {
    let name = args
        .first() // 获取命令名之后的第一个参数
        .map(|s| s.as_str()) // 将该参数转换为字符串切片
        .unwrap_or("Mingling"); // 如果没有参数，则使用默认值 `"Mingling"`
    println!("Hello, {}!", name); // 输出问候语
}
 
gen_program!();
```
 
使用 `cargo run -- greet` 运行 `greet` 命令：

```bash,simulation
~# cargo run -- greet
Hello, Mingling!
~# cargo run -- greet Alice
Hello, Alice!
```
 
可以看到，执行的效果完全符合我们的预期，不过 ...

**它不符合 Mingling 的哲学 —— 通过架构分离副作用。**

## “通过架构分离副作用”

上述代码虽能完美满足我们设想的样子，但它并不是 Mingling 的最佳实践，下面请将注意力集中在 `#[command]` 绑定函数的实现上：

```rust
#[command]
pub fn greet(args: Vec<String>) {
    // ...
@@@let name = args.first().map(|s| s.as_str()).unwrap_or("Mingling");
    // 在此处，greet 函数直接调用了 `println!`
    println!("Hello, {}!", name);
}
```
 
`println!` 在 `fn greet` 函数内通过执行标准输出产生了副作用，
这意味着：当我们需要测试该函数时，你必须捕获其中的 stdout 才能断言它的结果，这是 **难以测试** 的。

Mingling 提供了更好的方法来让其 **更适合测试** —— 增加结果类型和渲染器：通过定义结果类型，`fn greet` 可以轻松地将结果包装成类型化的信息输出，
渲染器也能根据类型化的信息决定如何呈现，如此一来：**函数只负责输出，渲染器只负责呈现**。

我们增加如下代码：

```rust
#[derive(Grouped)] // 通过 `Grouped` 将结构体添加到当前程序中
pub struct ResultGreet { // 创建结构体 `ResultGreet`
    name: String // 定义内部值 `name`
}
 
#[renderer] // 定义渲染器
pub fn render_greet(r: ResultGreet) -> String { // 处理 `ResultGreet`，并渲染为 `String`
    format!("Hello, {}\n", r.name).into() // 将 `r.name` 填入模板中
}
```
 
修改原来的 `fn greet` 函数：

```rust,diff
@@@#[derive(Grouped)] pub struct ResultGreet { name: String }
#[command]
- pub fn greet(args: Vec<String>) {
+ pub fn greet(args: Vec<String>) -> ResultGreet { // 返回数据类型 `ResultGreet`
    let name = args
        .first()
        .map(|s| s.as_str())
        .unwrap_or("Mingling");
-     println!("Hello, {}!", name); // 删除原来的直接 stdout
+     ResultGreet { name: name.to_string() } // 包装为数据类型 `ResultGreet`
}
```
 
完整代码：

```rust
use mingling::prelude::*;
 
fn main() {
    let mut program = ThisProgram::new();
    program.exec_and_exit();
}
 
#[derive(Grouped)]
pub struct ResultGreet {
    name: String
}
 
#[command]
pub fn greet(args: Vec<String>) -> ResultGreet {
    let name = args
        .first()
        .map(|s| s.as_str())
        .unwrap_or("Mingling")
        .to_string();
    ResultGreet { name }
}
 
#[renderer]
pub fn render_greet(r: ResultGreet) -> String {
    format!("Hello, {}\n", r.name).into()
}
 
gen_program!();
```
 
执行命令：

```bash,simulation
~# cargo run -- greet
Hello, Mingling!
~# cargo run -- greet Alice
Hello, Alice!
```
 
---

🎉 至此，你已经了解了如何使用 `#[command]` 和 `#[renderer]` 制作符合 Mingling 哲学的 `greet` 命令。

---

## 你可能还疑惑的点：

### 分离出 Renderer 后，代码更复杂了？

是的，在将结果从 `println!` 分离为 `struct` + `Renderer` 后，代码多了很多样板代码，但你换来了：

1. **可控渲染开关**：Renderer 的输出会将内容存储在一个缓冲区内而不是立刻输出，这为框架提供了可以通过 `--quiet` 等标识来统一控制输出的能力。
2. **结构化信息天然可序列化**：为类型添加 `serde::Serialize` trait，可以让他们具备序列化的能力，这让 `--json`、`--message-format=json` 等标识可以轻松实现。
3. **极致的可测试性**：在例子当中，你的 `fn greet` 只是个 **纯函数**，你天然可以直接构造 `Vec<String>`、断言 `ResultGreet` 来验证程序正确性。

### 什么是 `Grouped`？

你肯定在上述代码中见到过如下片段：

```rust
#[derive(Grouped)]
pub struct ResultGreet {
    // ...
@@@ name: String
}
```
 
这里的 `Grouped` 派生宏做了什么？

简单来说，`Grouped` 为程序实现了 `Grouped<crate::ThisProgram>` trait，
并同时为该类型注册了全局唯一的成员ID，它是 Mingling 程序运行时判别输入的类型的最快手段。

展开后大致如下：

```rust
use mingling::Grouped;
use mingling::macros::register_type;
 
pub struct ResultGreet {
    name: String
}
 
unsafe impl Grouped<ThisProgram> for ResultGreet { // 因为它的类型由宏手动保证，所以不安全
    fn member_id() -> ThisProgram {
        // 将生成的变体绑定到 trait 上
        ThisProgram::ResultGreet
    }
}
 
// 将成员 ID: ResultGreet 注册到程序
register_type!(ResultGreet); // 为 `ThisProgram` 生成 `ResultGreet` 枚举变体
```
