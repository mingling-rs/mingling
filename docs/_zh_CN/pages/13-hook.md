<h1 align="center">钩子系统</h1>
<p align="center">
    如何使用 ProgramHook 向程序内部插入行为
</p>

Hook 让你在管线的各个生命周期节点插入自定义逻辑 —— 在 dispatch 之前、chain 之后、render 前后、程序退出时 ……

你可以把横切关注点（日志、鉴权、指标收集）写在 hook 里，而不是散落在各处的业务代码中。

## 基本用法

`ProgramHook` 采用 builder 模式构造：

```rust
@@@use mingling::hook::ProgramHook;
fn main() {
    let mut program = ThisProgram::new();
    program.with_hook(
        ProgramHook::empty()
            .on_pre_chain(|info| {
                println!("before chain: {}", info.input);
            })
            .on_post_render(|info| {
                println!("after render: {}", info.result);
            }),
    );
    program.exec_and_exit();
}
```
 
> [!TIP]
> `ProgramHook::empty()` 创建一个空 hook，然后链式调用 `.on_*()` 方法注册你关心的生命周期节点。没有注册的节点不会执行。

## 生命周期节点

Hook 覆盖了管线的完整生命周期：

| 阶段         | Hook               | 触发时机      |
| ------------ | ------------------ | ------------- |
| **Dispatch** | `on_begin`         | 执行开始      |
|              | `on_pre_dispatch`  | Dispatch 之前 |
|              | `on_post_dispatch` | Dispatch 之后 |
| **Chain**    | `on_pre_chain`     | Chain 执行前  |
|              | `on_post_chain`    | Chain 执行后  |
| **Render**   | `on_pre_render`    | Render 执行前 |
|              | `on_post_render`   | Render 执行后 |
| **Finish**   | `on_finish`        | 程序退出前    |

每个 hook 回调接收对应的 `Hook*Info` 结构体，里面包含当前上下文的信息（输入类型、参数、渲染结果等）。

## 实际例子：记录操作日志

```rust
@@@use mingling::prelude::*;
@@@use mingling::hook::ProgramHook;
@@@
@@@dispatcher!("greet", CMDGreet => EntryGreet);
@@@pack!(ResultName = String);
@@@
@@@#[chain] fn handle_greet(args: EntryGreet) -> Next {
@@@    ResultName::new(args.inner.first().cloned().unwrap_or_default()).to_render()
@@@}
@@@#[renderer] fn render_name(r: ResultName) { r_println!("Hello, {}!", *r); }
fn main() {
    let mut program = ThisProgram::new();
 
    // 记录每次 chain 执行前后的信息
    program.with_hook(
        ProgramHook::empty()
            .on_pre_chain(|info| {
                eprintln!("[hook] executing chain for: {}", info.input);
            })
            .on_post_chain(|info| {
                eprintln!("[hook] chain output: {}", info.output.member_id);
            }),
    );
 
    program.with_dispatcher(CMDGreet);
    program.exec_and_exit();
}
```
 
运行效果：

```text
[hook] executing chain for: EntryGreet
[hook] chain output: ResultName
Hello, World!
```
 
## 通过 Hook 控制行为

Hook 不仅用来 "看"，还可以用 `ProgramControlUnit` 改变程序行为：

| 变体                       | 效果                                      |
| -------------------------- | ----------------------------------------- |
| `Continue`                 | 什么都不做，继续执行                      |
| `OverrideExitCode(i32)`    | 覆盖退出码                                |
| `RouteToChain(AnyOutput)`  | 将当前数据替换为新的，重新进入 Chain 循环 |
| `RouteToRender(AnyOutput)` | 跳过后续 Chain，直接渲染                  |

> [!NOTE]
> Hook 可以注册多个，按注册顺序执行。

<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
