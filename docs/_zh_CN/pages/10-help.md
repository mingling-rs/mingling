<h1 align="center">帮助信息</h1>
<p align="center">
    为命令添加 --help 支持
</p>

没有帮助信息的 CLI 不是好 CLI。

Mingling 里用 `#[help]` 宏给命令添加帮助文本。

## 最简单的帮助

直接给 Entry 写一个帮助函数：

```rust
@@@use mingling::macros::help;
@@@use mingling::macros::buffer;
@@@dispatcher!("greet", EntryGreet);
#[help(buffer)]
fn help_greet(_entry: EntryGreet) {
    r_println!("Usage: greet [name]");
    r_println!("Say hello to someone.");
}
```
 
> [!NOTE]
> 帮助函数同样通过 `r_println!` 向 `RenderResult` 写入内容，因为 `#[help]` 遵循渲染管线 —— 它是由 `--help` 标志提前触发的短路渲染，而不是管线之外的逻辑。

## 全局帮助

你也可以为 `EntryFallback` 写帮助，作为"根帮助"：

```rust
@@@use mingling::macros::help;
@@@use mingling::macros::buffer;
// 用户直接输入 --help 时触发
#[help(buffer)]
fn help_root(entry: EntryFallback) {
    r_println!("Usage: my-cli <command>");
    r_println!("Commands:");
    r_println!("  greet    Say hello");
}
```
 
> [!TIP]
> `EntryFallback` 是 `gen_program!()` 自动生成的类型，代表"没有匹配到任何命令"的情况。为它写 `#[help]` 就是给程序的根命令加帮助。

## 需要 Setup 配合

要让 `--help` 正常工作，需要在 `main` 里加上 `BasicProgramSetup`：

```rust
@@@use mingling::macros::help;
@@@use mingling::setup::BasicProgramSetup;
@@@dispatcher!("greet", EntryGreet);
fn main() {
    let mut program = ThisProgram::new();
    program.with_setup(BasicProgramSetup);
    program.exec_and_exit();
}
```
 
`BasicProgramSetup` 内置了 `HelpFlagSetup`，它的作用仅仅是把 `program.user_context.help` 设为 `true`。

真正把请求路由到 `#[help]` 函数的是 `gen_program!()` 生成的代码 —— 它在调度时检查这个标记，如果为 `true` 就走帮助渲染路径，不经过 Chain。

不加 `BasicProgramSetup` 的话，`--help` 只是一个普通参数，会被当成 Entry 的输入传给 Chain。

<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
