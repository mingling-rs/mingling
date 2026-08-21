<h1 align="center">程序装配</h1>
<p align="center">
    用 Setup 初始化你的程序
</p>

当程序启动时需要做一些初始化工作——比如解析全局参数、注册资源——你可以用 `#[program_setup]` 来组织这些逻辑。

## 用 Setup 做初始化

```rust
@@@use mingling::macros::program_setup;
@@@use mingling::Program;
@@@use mingling::config::Verbosity;
#[program_setup]
fn my_setup(program: &mut Program<ThisProgram>) {
    // 从参数中提取全局标志
    program.global_flag(["-v", "--verbose"], |program| {
        program.stdout_setting.verbosity = Verbosity::Verbose;
    });
}
@@@
@@@fn main() {
@@@    let mut program = ThisProgram::new();
@@@    program.with_setup(MySetup);
@@@    program.exec_and_exit();
@@@}
@@@gen_program!();
```
 
`#[program_setup]` 标记的函数接收 `&mut Program<ThisProgram>`，你可以在里面做任何初始化操作。

在 `main` 里通过 `program.with_setup(...)` 注册即可使用。

> [!NOTE]
> `#[program_setup]` 无需任何特性即可使用。也可以手动实现 `ProgramSetup` trait。

## 提取全局参数

Setup 里最常用的操作就是提取全局参数。Mingling 提供了几个辅助方法：

```rust
@@@use mingling::macros::program_setup;
@@@use mingling::Program;
@@@use mingling::config::Verbosity;
#[program_setup]
fn my_setup(program: &mut Program<ThisProgram>) {
    // 布尔标志
    program.global_flag(["-v", "--verbose"], |program| {
        program.stdout_setting.verbosity = Verbosity::Verbose;
    });
 
    // 带值的参数
    program.global_argument("--name", |_program, value| {
        // value 就是 "Alice"
        let _ = value;
    });
}
```
 
> [!TIP]
> `global_flag` 和 `global_argument` 会自动从 `program.args` 中移除已匹配的参数，这些参数不会进入管线。

## 内置 Setup

Mingling 提供了一些开箱即用的 Setup，覆盖了 CLI 程序最常见的需求：

| Setup                       | 功能                                                                  |
| --------------------------- | --------------------------------------------------------------------- |
| `BasicProgramSetup`         | 解析 `--help`/`-h`、`--quiet`/`-q`、`--confirm`/`-C`                  |
| `DirectoryEnvironmentSetup` | 注册目录资源：当前目录、可执行目录、Home 目录、临时目录               |
| `ExitCodeSetup`             | 通过 `ResExitCode` 控制程序退出码                                     |
| `StructuralRendererSetup`   | 启用 `--json`、`--yaml` 等结构化输出（需 `structural_renderer` 特性） |

用法就是在 `main` 里加一行：

```rust
@@@use mingling::setup::BasicProgramSetup;
fn main() {
    let mut program = ThisProgram::new();
    program.with_setup(BasicProgramSetup);
    program.exec_and_exit();
}
```
 
`BasicProgramSetup` 帮你处理了绝大多数 CLI 程序都需要的通用参数，省去自己手动解析的麻烦。

<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
