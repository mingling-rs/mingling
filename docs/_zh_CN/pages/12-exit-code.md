<h1 align="center">退出码控制</h1>
<p align="center">
    如何使用资源系统管理程序退出码
</p>

程序退出时给 shell 一个正确的退出码是 CLI 的基本素养

。Mingling 提供了开箱即用的 `ExitCodeSetup`，配合 `ResExitCode` 资源，让退出码控制变得极其简单。

## 启用 ExitCodeSetup

```rust
@@@use mingling::prelude::*;
@@@use mingling::setup::ExitCodeSetup;
fn main() {
    let mut program = ThisProgram::new();
    program.with_setup(ExitCodeSetup::default());
@@@ program.exec_and_exit();
}
```
 
`ExitCodeSetup` 做了两件事：

1. 注册 `ResExitCode` 资源（默认值为 `0`）
2. 注册一个 `finish` hook，在程序退出前读取 `ResExitCode` 的值作为最终退出码

## 修改退出码

在 Chain 或 Renderer 中通过 `ResExitCode` 注入来修改退出码：

```rust
@@@use mingling::res::ResExitCode;
@@@use mingling::setup::ExitCodeSetup;
@@@pack!(EntryCheck = Vec<String>);
#[chain]
fn handle_check(_args: EntryCheck, ec: &mut ResExitCode) {
    // 检查失败的时候修改退出码资源
    ec.exit_code = 1;
}
```
 
> [!TIP]
> `ResExitCode` 就是一个 `struct ResExitCode { pub exit_code: i32 }`。`&mut ResExitCode` 注入后直接改字段即可。

## `Program` 的三种执行方式

`Program` 提供了三种执行方式（不包括 `repl` 特性下的 `exec_repl`）：

| 方式                            | 行为                                                                       |
| ------------------------------- | -------------------------------------------------------------------------- |
| `program.exec_and_exit()`       | 执行并直接以退出码终止进程                                                 |
| `program.exec()`                | 执行后返回 `i32` 退出码，由调用方决定怎么处理                              |
| `program.exec_without_render()` | 返回 `Result<RenderResult, ProgramExecuteError>`，可读取内部的 `exit_code` |

```rust
@@@use mingling::setup::ExitCodeSetup;
fn main() {
    let mut program = ThisProgram::new();
    program.with_setup(ExitCodeSetup::default());
 
    // 获取退出码自行处理
    let exit_code = program.exec();
    std::process::exit(exit_code);
}
@@@gen_program!();
```
 
<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
