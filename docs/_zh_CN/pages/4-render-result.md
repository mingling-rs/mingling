<h1 align="center">将结果渲染</h1>
<p align="center">
    使用 renderer 宏声明渲染器，将结果输出
</p>

现在，我们创建了 Dispatcher 和 Chain，也通过 `#[derive(Grouped, Wrap)]` 产出了一个 Result 类型。最后一步：**把结果展示给用户**。

## `#[renderer]` 宏

跟 `#[chain]` 类似，`#[renderer]` 用于标记一个输出函数：

```rust
@@@use mingling::macros::buffer;
@@@#[derive(Grouped, Wrap)]
@@@pub struct ResultName(String);
#[renderer(buffer)]
fn render_name(name: ResultName) {
    r_println!("Hello, {}!", *name);
}
```
 
Renderer 接收 Chain 产出的结果，然后返回一个 `RenderResult`。在函数内部，创建 `RenderResult`，用 `r_print!` / `r_println!` 写入内容，最后返回它。

## `buffer` 拓展

若您觉得显式创建并返回 `RenderResult` 过于繁琐，可以使用 `#[renderer(buffer)]` 在原始函数内植入一个缓冲区。

```rust
use mingling::macros::buffer;
 
@@@#[derive(Grouped, Wrap)]
@@@pub struct ResultName(String);
#[renderer(buffer)]
fn render_name(name: ResultName) {
    r_println!("Hello, {}!", *name);
}
```
 
这样，你的 renderer 函数就获得了更简洁的语法，但也引入了一个隐含机制：它会向函数内注入一个名为 `__render_result_buffer` 的可变 `RenderResult`。当 `r_print!` 宏没有显式指定 `RenderResult` 时，它便会按照约定向该缓冲区追加输出。

## `RenderResult` 类型

`RenderResult` 是一个缓冲区类型，持有渲染后的文本和退出码。它不是直接输出到终端，而是把内容写入缓冲区。这样做的好处是：

1. **持有退出码**——你可以设置程序以特定退出码结束
2. **方便测试**——可以捕获渲染结果做断言
3. **便于后处理**——你可以将结果捕获，统一进行文本后处理

## 完整的可运行程序

把三篇教程的内容合在一起，你的第一个 Mingling 程序就完整了：

```rust
use mingling::macros::buffer;
 
// 1. 用 Dispatcher 声明命令
dispatcher!("greet", EntryGreet);
 
// 2. 用 #[derive(Grouped, Wrap)] 声明结果数据
#[derive(Grouped, Wrap)]
pub struct ResultName(String);
 
// 3. 用 Chain 处理逻辑
#[chain]
fn handle_greet(args: EntryGreet) -> Next {
    let name = args.0
        .first()
        .cloned()
        .unwrap_or_else(|| "World".to_string());
    ResultName(name).into()
}
 
// 4. 用 Renderer 输出结果
#[renderer(buffer)]
fn render_name(name: ResultName) {
    r_println!("Hello, {}!", *name);
}
 
// 5. 在 main 函数内装配程序并运行
fn main() {
    let mut program = ThisProgram::new();
    program.exec_and_exit();
}
 
// 6. 使用 gen_program! 生成完整程序
gen_program!();
```
 
## 跑起来试试

```bash
~# cargo run -- greet Alice
```
 
```text
Hello, Alice!
```
 
试试不给参数：

```bash
~# cargo run -- greet
```
 
```text
Hello, World!
```
 
试试不存在的命令：

```bash
cargo run -- great
```
 
```text
# 什么也没输出！
```
 
## 补上 Fallback

`gen_program!()` 自动生成了一个 `EntryFallback` 类型，包裹 `Vec<String>`——它存的是用户输入的那些没匹配到的命令。你只需要给它写一个 Renderer：

```rust
use mingling::macros::buffer;
 
#[renderer(buffer)]
fn render_entry_fallback(err: EntryFallback) {
    if err.0.is_empty() {
        r_println!("Unknown command");
    } else {
        r_println!("Command not found: \"{}\"", err.0.join(" "));
    }
}
```
 
加上之后，再试试不存在的命令：

```bash
cargo run -- great
```
 
```text
Command not found: "great"
```
 
## 恭喜

你完成了第一个完整的 Mingling 程序！来回顾一下学到的东西：

| 概念     | 对应宏/函数                | 一句话                     |
| -------- | -------------------------- | -------------------------- |
| 声明命令 | `dispatcher!`              | 告诉程序用户能输入什么     |
| 处理逻辑 | `#[chain]`                 | 收到参数后做什么           |
| 输出结果 | `#[renderer]`              | 怎么把结果告诉用户         |
| 类型包装 | `#[derive(Grouped, Wrap)]` | 给你的数据取个有意义的名字 |
| 程序入口 | `gen_program!()`           | 自动生成管线的接线图       |

在真实项目中你还会用到资源注入、hook、补全、REPL 等高级功能，不过核心骨架永远不变：**Dispatcher → Chain → Renderer**。

<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
