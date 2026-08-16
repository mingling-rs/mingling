<h1 align="center">使用资源系统</h1>
<p align="center">
    手把手带你使用资源
</p>

资源是 Mingling 中管理全局状态的机制。任何实现了 `Default + Clone` 的类型都可以成为资源。

## 定义一个资源

```rust
// 只要实现 Default + Clone，就可以作为资源使用
#[derive(Default, Clone)]
struct ResCurrentDir(String);
 
// 注册到 Program
fn main() {
    let mut program = ThisProgram::new();
    program.with_resource(ResCurrentDir(".".into()));
    program.exec_and_exit();
}
```
 
因为 `ResCurrentDir` 同时实现了 `Default` 和 `Clone`，框架会自动为它实现 `ResourceMarker` trait，无需手动 impl。

## 注入并使用

在 Chain 或 Renderer 中，只需在参数列表里声明你要的资源：

```rust
@@@use mingling::macros::buffer;
@@@#[derive(Default, Clone)]
@@@struct ResCurrentDir(String);
@@@dispatcher!("pwd", EntryPrintWorkingDir);
@@@#[derive(Grouped, Wrap)]
@@@pub struct ResultPath(String);
// 通过 &T 注入只读资源
#[chain]
fn handle_pwd(_args: EntryPrintWorkingDir, cwd: &ResCurrentDir) -> Next {
    ResultPath(cwd.0.clone()).to_render()
}
 
#[renderer(buffer)]
fn render_path(result: ResultPath) {
    r_println!("{}", *result);
}
```
 
## 修改资源

用 `&mut T` 注入可修改资源：

```rust
@@@use mingling::macros::buffer;
@@@#[derive(Default, Clone)]
@@@struct ResVisitCount(u32);
@@@dispatcher!("visit", EntryVisit);
@@@#[derive(Grouped, Wrap, Default)]
@@@pub struct ResultDone(());
#[chain]
fn handle_visit(_args: EntryVisit, counter: &mut ResVisitCount) -> Next {
    counter.0 += 1;
    ResultDone::default().into()
}
 
#[renderer(buffer)]
fn render_done(_done: ResultDone, counter: &ResVisitCount) {
    r_println!("visit count is : {}", counter.0);
}
```
 
## 多个资源同用

Chain 可以同时注入任意多个资源，框架按类型自动匹配：

```rust
@@@#[derive(Default, Clone)] struct ResConfig(String);
@@@#[derive(Default, Clone)] struct ResCounter(u32);
@@@dispatcher!("test", EntryTest);
@@@#[derive(Grouped, Wrap, Default)]
@@@pub struct ResultDone(());
// 同时注入只读 + 可修改
#[chain]
fn handle_test(_args: EntryTest, config: &ResConfig, counter: &mut ResCounter) -> Next {
    println!("config: {}", config.0);
    counter.0 += 1;
    ResultDone::default().to_render()
}
```
 
如果要深入了解 `ResourceMarker`、`LazyRes` 惰性加载等进阶内容，可以查看 [核心概念：资源系统](pages/concepts/2-resource)。

<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
