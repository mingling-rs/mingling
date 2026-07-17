<h1 align="center">将结果渲染</h1>
<p align="center">
    使用 renderer 宏声明渲染器，将结果输出
</p>

现在，我们创建了 Dispatcher 和 Chain，也通过 `pack!` 产出了一个 Result 类型。最后一步：**把结果展示给用户**。

## `#[renderer]` 宏

跟 `#[chain]` 类似，`#[renderer]` 用于标记一个输出函数：

```rust
use std::io::Write;
 
@@@pack!(ResultName = String);
#[renderer]
fn render_name(name: ResultName) -> RenderResult {
    let mut result = RenderResult::new();
    writeln!(result, "Hello, {}!", *name).ok();
    result
}
```
 
Renderer 接收 Chain 产出的结果，然后返回一个 `RenderResult`。在函数内部，创建 `RenderResult`，用 `write!` / `writeln!`（来自 [`std::io::Write`](https://doc.rust-lang.org/std/io/trait.Write.html)）写入内容，最后返回它。

## `RenderResult` 类型

`RenderResult` 是一个缓冲区类型，持有渲染后的文本和退出码。它不是直接输出到终端，而是把内容写入缓冲区。这样做的好处是：

1. **持有退出码**——你可以设置程序以特定退出码结束
2. **方便测试**——可以捕获渲染结果做断言
3. **便于后处理**——你可以将结果捕获，统一进行文本后处理

## 完整的可运行程序

把三篇教程的内容合在一起，你的第一个 Mingling 程序就完整了：

```rust
use std::io::Write;
 
// 1. 用 Dispatcher 声明命令
dispatcher!("greet", CMDGreet => EntryGreet);
 
// 2. 用 pack! 声明结果数据
pack!(ResultName = String);
 
// 3. 用 Chain 处理逻辑
#[chain]
fn handle_greet(args: EntryGreet) -> Next {
    let name = args.inner
        .first()
        .cloned()
        .unwrap_or_else(|| "World".to_string());
    ResultName::new(name).into()
}
 
// 4. 用 Renderer 输出结果
#[renderer]
fn render_name(name: ResultName) -> RenderResult {
    let mut result = RenderResult::new();
    writeln!(result, "Hello, {}!", *name).ok();
    result
}
 
// 5. 在 main 函数内装配程序并运行
fn main() {
    let mut program = ThisProgram::new();
    program.with_dispatcher(CMDGreet);
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

`gen_program!()` 自动生成了一个 `ErrorDispatcherNotFound` 类型，包裹 `Vec<String>`——它存的是用户输入的那些没匹配到的命令。你只需要给它写一个 Renderer：

```rust
use std::io::Write;
 
#[renderer]
fn render_dispatcher_not_found(err: ErrorDispatcherNotFound) -> RenderResult {
    let mut result = RenderResult::new();
    if err.inner.is_empty() {
        writeln!(result, "Unknown command").ok();
    } else {
        writeln!(result, "Command not found: \"{}\"", err.inner.join(" ")).ok();
    }
    result
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

| 概念     | 对应宏/函数      | 一句话                     |
| -------- | ---------------- | -------------------------- |
| 声明命令 | `dispatcher!`    | 告诉程序用户能输入什么     |
| 处理逻辑 | `#[chain]`       | 收到参数后做什么           |
| 输出结果 | `#[renderer]`    | 怎么把结果告诉用户         |
| 类型包装 | `pack!`          | 给你的数据取个有意义的名字 |
| 程序入口 | `gen_program!()` | 自动生成管线的接线图       |

在真实项目中你还会用到资源注入、hook、补全、REPL 等高级功能，不过核心骨架永远不变：**Dispatcher → Chain → Renderer**。

<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
