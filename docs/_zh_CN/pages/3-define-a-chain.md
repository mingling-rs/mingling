<h1 align="center">声明一个链</h1>
<p align="center">
    使用 chain 宏声明链，并承接 Entry 输入
</p>

上一节我们声明了 `dispatcher!("greet", CMDGreet => EntryGreet)`

现在用户输入 `greet` 时会被匹配并包装成 `EntryGreet`。

但拿到 Entry 之后呢？

我们需要一个 Chain 来处理它。

## `#[chain]` 宏

`#[chain]` 用来标记一个处理函数，格式非常直接：

```rust
@@@dispatcher!("greet", CMDGreet => EntryGreet);
pack!(ResultName = String);
 
#[chain]
fn handle_greet(args: EntryGreet) -> Next {
    // args 就是用户输入经过匹配后剩下的参数
    let name = args.inner.first().cloned().unwrap_or_else(|| "World".to_string());
    // 把结果包装成 Next，告诉调度器下一步去哪
    ResultName::new(name).into()
}
```
 
注意到了吗？

Chain 函数签名里写着它需要什么——`args: EntryGreet`

然后用 `ResultName::new(name)` 返回一个新类型。

这个返回的 `Next` 会展开成 `impl Into<ChainProcess<ThisProgram>>`。

> [!TIP]
> 想知道 `Into<ChainProcess<G>>` 是怎么工作的？
>
> 可以去 [任意输出机制](pages/concepts/3-any-output) 章节了解 `ChainProcess`。

## `pack!` 宏

你大概猜到了，`pack!(ResultName = String)` 定义了一个管线中传递的类型：

```rust
// pack!(ResultName = String) 大概生成了这样的代码
 
#[derive(Grouped)]
pub struct ResultName {
    pub inner: String,
}
```
 
你可以把它理解为一个 打了标签的 `String`。

调度器通过这个标签来精确路由，确保数据不会混淆 —— 比如发给 `RenderGreet` 的数据不会被误传给 `RenderError`。

> [!NOTE]
> 与简单的类型别名 (`type`) 不同，`pack!` 会生成一个全新的类型，拥有独立的 `TypeId`。

命名上推荐这样的习惯：

| 角色     | 命名模式         | 示例                 |
| -------- | ---------------- | -------------------- |
| 入口     | `Entry` + 命令名 | `EntryGreet`         |
| 中间状态 | `State` + 描述   | `StateParsedArgs`    |
| 最终结果 | `Result` + 描述  | `ResultGreetSomeone` |
| 错误     | `Error` + 描述   | `ErrorUserNotFound`  |

详见 [命名规范](pages/other/naming_rule)，不过现在你只需要记住：**用 `pack!` 给你的数据取一个有意义的名字**。

## 从 Entry 中提取参数

`EntryGreet` 的 `inner` 是一个 `Vec<String>`，你可以在 Chain 里自由地处理它：

```rust
@@@dispatcher!("greet", CMDGreet => EntryGreet);
@@@pack!(ResultName = String);
#[chain]
fn handle_greet(args: EntryGreet) -> Next {
    // 取第一个参数，没有就用默认值
    let name = args
        .inner
        .first()
        .cloned()
        .unwrap_or_else(|| "World".to_string());
 
    ResultName::new(name).into()
}
```
 
如果你启用了 `parser` 特性，还可以用 `Picker` 做更灵活的参数提取，不过那是后话了。

## 组合起来

现在把 Dispatcher 和 Chain 连在一起：

```rust
// 1. 声明命令
dispatcher!("greet", CMDGreet => EntryGreet);
 
// 2. 声明管线中的数据类型
pack!(ResultName = String);
 
// 3. 处理逻辑
#[chain]
fn handle_greet(args: EntryGreet) -> Next {
    let name = args.inner
        .first()
        .cloned()
        .unwrap_or_else(|| "World".to_string());
    ResultName::new(name).into()
}
 
fn main() {
    let mut program = ThisProgram::new();
    program.with_dispatcher(CMDGreet);
    program.exec_and_exit();
}
 
gen_program!();
```
 
不过这段代码还没写完 —— 我们只有 Dispatcher 和 Chain，还差最后一步：**把结果渲染出来**。这就是下一篇要讲的 Renderer。

<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
