<h1 align="center">错误处理</h1>
<p align="center">
    将错误优雅地展示给用户
</p>

管线里不只有成功路径。当输入有误、资源找不到、操作失败时，你需要一个地方来处理这些"意外"，而不是让程序 panic。

## 两个路径：成功 vs 错误

回顾管线模型，Chain 的返回值是 `Next`，它有两个去向：

| 路由           | 含义                         |
| -------------- | ---------------------------- |
| `.to_render()` | 出结果了，交给 Renderer 展示 |
| `.to_chain()`  | 还没处理完，交给下一个 Chain |

错误类型的值也可以走任意一条路——你可以选择直接渲染错误信息，也可以交给下一个 Chain 尝试恢复。

## 用独立类型区分错误

```rust
@@@dispatcher!("greet", EntryGreet);
#[derive(Grouped, Wrap)]
pub struct ResultGreeting(String);
#[derive(Grouped, Wrap)]
pub struct ErrorNameEmpty(String);
 
#[chain]
fn handle_greet(args: EntryGreet) -> Next {
    let name = args.0.first().cloned().unwrap_or_default();
 
    if name.is_empty() {
        ErrorNameEmpty("name is required".to_string()).to_render()
    } else {
        ResultGreeting(name).to_render()
    }
}
```
 
然后各自写 Renderer：

```rust
@@@use mingling::macros::buffer;
@@@dispatcher!("greet", EntryGreet);
@@@#[derive(Grouped, Wrap)]
@@@pub struct ResultGreeting(String);
@@@#[derive(Grouped, Wrap)]
@@@pub struct ErrorNameEmpty(String);
@@@#[chain] fn handle_greet(args: EntryGreet) -> Next { ResultGreeting(args.0.first().cloned().unwrap_or_default()).to_render() }
 
#[renderer(buffer)]
fn render_greet(result: ResultGreeting) {
    r_println!("Hello, {}!", *result);
}
 
#[renderer(buffer)]
fn render_error_name_empty(err: ErrorNameEmpty) {
    r_println!("Error: {}", *err);
}
```
 
两个 Renderer 各司其职，用户看到什么取决于 Chain 返回了什么。

## 完整的例子

```rust
@@@use mingling::macros::buffer;
dispatcher!("greet", EntryGreet);
 
#[derive(Grouped, Wrap)]
pub struct ResultGreeting(String);
#[derive(Grouped, Wrap)]
pub struct ErrorNameEmpty(String);
 
#[chain]
fn handle_greet(args: EntryGreet) -> Next {
    let name = args.0.first().cloned().unwrap_or_default();
    if name.is_empty() {
        ErrorNameEmpty("name is required".to_string()).to_render()
    } else {
        ResultGreeting(name).to_render()
    }
}
 
#[renderer(buffer)]
fn render_greet(result: ResultGreeting) {
    r_println!("Hello, {}!", *result);
}
 
#[renderer(buffer)]
fn render_error_name_empty(err: ErrorNameEmpty) {
    r_println!("Error: {}", *err);
}
 
fn main() {
    let mut program = ThisProgram::new();
    program.exec_and_exit();
}
 
gen_program!();
```
 
运行效果：

```text
~# my-cli greet Alice
Hello, Alice!
 
~# my-cli greet
Error: name is required
```
 
## 关于错误类型

不需要额外上下文、只起“标记”作用的错误类型，可以直接用 `#[derive(Grouped, Default)]` 声明：

```rust
// Features: ["extras"]
#[derive(Grouped, Default)]
pub struct ErrorNotFound;
```
 
详见 [特性列表](pages/other/features)。

<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
