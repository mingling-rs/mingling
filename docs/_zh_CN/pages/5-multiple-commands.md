<h1 align="center">多命令程序</h1>
<p align="center">
    在一个程序里添加多个命令
</p>

真实世界的 CLI 很少只有一个命令。这篇我们来扩展之前的 greet 程序，加上第二个命令，看看多命令的程序长什么样。

## 添加第二个命令

继续在同一个项目里操作：

```rust
// 声明两个命令
dispatcher!("greet", CMDGreet => EntryGreet);
dispatcher!("add",   CMDAdd   => EntryAdd);
 
pack!(ResultGreeting = String);
pack!(ResultSum = i32);
 
#[chain]
fn handle_greet(args: EntryGreet) -> Next {
    let name = args.inner.first().cloned().unwrap_or_else(|| "World".to_string());
    ResultGreeting::new(name)
}
 
#[chain]
fn handle_add(args: EntryAdd) -> Next {
    let sum: i32 = args.inner.iter().filter_map(|s| s.parse::<i32>().ok()).sum();
    ResultSum::new(sum)
}
 
#[renderer]
fn render_greet(result: ResultGreeting) {
    r_println!("Hello, {}!", *result);
}
 
#[renderer]
fn render_sum(result: ResultSum) {
    r_println!("Sum: {}", *result);
}
 
fn main() {
    let mut program = ThisProgram::new();
    program.with_dispatchers((CMDGreet, CMDAdd));
    program.exec_and_exit();
}
 
gen_program!();
```
 
两个命令共享同一个管线模型，但各走各的：

```text
> my-cli greet Alice
Hello, Alice!
> my-cli add 1 2 3
Sum: 6
```
 
## 注册多个分发器

注意到 `with_dispatchers` 了吗？当你需要注册多个分发器时，一次传一个元组就行：

```rust
@@@dispatcher!("greet", CMDGreet => EntryGreet);
@@@dispatcher!("add", CMDAdd => EntryAdd);
@@@pack!(ResultGreeting = String);
@@@pack!(ResultSum = i32);
@@@#[chain] fn handle_greet(_args: EntryGreet) -> Next { ResultGreeting::new("ok".into()) }
@@@#[renderer] fn render_greet(_greeting: ResultGreeting) { r_println!("hi"); }
@@@#[chain] fn handle_add(_args: EntryAdd) -> Next { ResultSum::new(0) }
@@@#[renderer] fn render_sum(_sum: ResultSum) { r_println!("sum"); }
fn main() {
    let mut program = ThisProgram::new();
    program.with_dispatchers((CMDGreet, CMDAdd));
    program.exec_and_exit();
}
```
 
等价于一个个注册，效果一样。

> [!TIP]
> 元组最多支持 7 个分发器。超过 7 个时链式调用 `with_dispatcher` 就行。

## 子命令

多层级的命令也是同理——每个点号分隔的层级都只是名字的一部分：

```rust
dispatcher!("remote.add", CMDRemoteAdd => EntryRemoteAdd);
dispatcher!("remote.rm",  CMDRemoteRm  => EntryRemoteRm);
```
 
每个子命令的 Entry、Chain、Renderer 完全独立，互不干扰。

## 数据类型的独立性

注意我们用了两个不同的 `pack!`：

- `pack!(ResultGreeting = String)`
- `pack!(ResultSum = i32)`

它们都是独立的类型，`gen_program!()` 会给它们分配不同的枚举变体。

调度器永远不会把 `ResultGreeting` 的数据送到 `render_sum` 去 —— **类型安全从命名那一刻就保证了**。

<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
