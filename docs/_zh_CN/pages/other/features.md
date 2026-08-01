<h1 align="center">特性</h1>
<p align="center">
    <b>Mingling</b> 的所有特性一览
</p>

## 特性 `all_serde_fmt`

**介绍:**

为 `structural_renderer` 启用所有序列化格式（JSON、RON、TOML、YAML）的 serde 格式化支持。

开启此特性将自动启用 `json_serde_fmt`、`ron_serde_fmt`、`toml_serde_fmt`、`yaml_serde_fmt` 四个子特性。

## 特性 `async`

**介绍:**

启用异步运行时支持，允许 `#[chain]` 绑定 `async` 函数，例如：

```rust
// Features: ["async"]
 
pack!(StateFoo = ());
 
#[chain]
async fn handle_state_foo(foo: StateFoo) -> Next {
    StateFoo::new(()).into()
}
```
 
详见 [示例](https://mingling-rs.github.io/mingling/docs/example-viewer.html?name=example-async-support)

## 特性 `builds`

**介绍:**

启用部分需要在 `build.rs` 使用的脚本，目前包含：

1. `comp` 特性下的补全脚本生成：

```rust
// BUILD TIME
// Features: ["builds", "comp"]
use mingling::build::build_comp_scripts;
 
// 为 `myprogram` 生成补全脚本
build_comp_scripts("myprogram").unwrap();
```
 
## 特性 `clap`

**介绍:**

启用与 [clap](https://crates.io/crates/clap) 命令行参数解析库的集成，方便构建 CLI 应用程序。

开启此特性后，可以使用 `#[dispatcher_clap]` 属性宏从 `clap::Parser` 结构体生成 dispatcher。

详见 [示例](https://mingling-rs.github.io/mingling/docs/example-viewer.html?name=example-clap-binding)

## 特性 `comp`

**介绍:**

启用代码补全功能，为交互式环境提供自动补全支持。

开启后可以使用 `#[completion]` 属性宏定义动态补全逻辑，并支持为 bash、zsh、fish、pwsh 等 shell 生成补全脚本。

详见 [示例](https://mingling-rs.github.io/mingling/docs/example-viewer.html?name=example-completion)

## 特性 `debug`

**介绍:**

启用调试相关功能，提供更详细的错误信息和诊断输出。

## 特性 `dispatch_tree`

**介绍:**

启用调度树机制，支持基于条件的分发和路由功能。

开启后，Mingling 在**编译时**将子命令结构硬编码为前缀树（Trie），实现极速的子命令查找。查找复杂度为 **O(n)**，其中 _n_ 是输入长度，而非命令数量。

详见 [示例](https://mingling-rs.github.io/mingling/docs/example-viewer.html?name=example-dispatch-tree)

## 特性 `extras`

**介绍:**

启用额外的宏集合，提供更多便捷的语法糖和元编程能力。

例如，允许 `dispatcher!("greet")` 的缩写形式，自动生成 `CMDGreet` / `EntryGreet`。

| 宏                                                      | 说明                                          |
| ------------------------------------------------------- | --------------------------------------------- |
| `empty_result!()`                                       | 链中提前返回空结果的简写                      |
| `entry!(Type, ["a", "b"])`                              | 构造入口类型的测试数据                        |
| `group!(Type)`                                          | 将外部类型注册为组成员，无需修改其定义        |
| `pack_err!(ErrorType)` / `pack_err!(ErrorType = Inner)` | 创建带自动 `name` 字段的错误类型              |
| `#[program_setup]`                                      | 声明程序初始化函数                            |
| `dispatcher!("cmd.path")` **缩写形式**                  | 省略 `CMDStruct => EntryStruct`，名字自动推导 |

<details>
<summary> Details </summary>

### `empty_result!()`

```rust
// Features: ["extras"]
 
pack!(StatePrev1 = ());
pack!(StatePrev2 = ());
 
pack!(StateNext = ());
 
#[chain]
fn handle_state_prev2(_p: StatePrev2) {
    // 无 Next 的 #[chain] 可以直接不返回值
}
 
#[chain]
fn handle_state_prev1(_p: StatePrev1) -> Next {
    let foo = 1;
    let bar = 2;
    if foo != bar {
        // 当需要 Next 且不需要返回值，便可以使用它
        empty_result!()
    } else {
        StateNext::new(()).into()
    }
}
```
 
### `#[program_setup]`

```rust
// Features: ["extras"]
use mingling::{macros::program_setup, Program};
 
fn main() {
    let mut program = ThisProgram::new();
    program.with_setup(NoErrorSetup);
    program.exec_and_exit();
}
 
#[program_setup]
fn no_error_setup(program: &mut Program<ThisProgram>) {
    program.global_flag(["--no-error"], |program| {
        program.stdout_setting.error_output = false;
    });
}
```
 
### `entry!`

```rust
// Features: ["extras"]
use mingling::macros::entry;
 
pack!(EntryHello = Vec<String>);
 
fn main() {
    let result: Next = handle_hello(entry!("--name", "Bob")).into();
    // ... 此处为断言逻辑
}
 
#[chain]
fn handle_hello(args: EntryHello) {}
```
 
### `group!`

将外部类型注册为程序组成员，无需修改原始类型的定义。
类型名会直接作为枚举变体，与 `pack!` 或 `#[derive(Grouped)]` 一致。

```rust
// Features: ["extras"]
use mingling::macros::group;
use std::num::ParseIntError;
 
// 将 std::num::ParseIntError 注册为组成员。
// 注册后，ParseIntError 可用于 #[chain] 和 #[renderer] 函数中。
group!(std::num::ParseIntError);
```
 
### `pack_err!`

创建带自动 `name: String` 字段的错误结构体，字段值自动设为结构体名的蛇形命名。
可选择包裹一个内部类型以携带额外上下文。

```rust
// Features: ["extras"]
use std::path::PathBuf;
 
// 简单形式——仅包含 name 字段：
pack_err!(ErrorNotFound);
// 生成：
//   struct ErrorNotFound { pub name: String }
//   impl Default for ErrorNotFound { ... }
 
// 带类型的形式——包含额外的 info 字段：
pack_err!(ErrorNotDir = PathBuf);
// 生成：
//   struct ErrorNotDir { pub name: String, pub info: PathBuf }
//   impl ErrorNotDir { pub fn new(info: PathBuf) -> Self { ... } }
```
 
</details>

## 特性 `structural_renderer`

**介绍:**

启用通用渲染器，提供基础的内容渲染能力。开启此特性将自动启用 `json_serde_fmt`。

开启后，用户可以通过 `--json` 或 `--yaml` 等标志获取结构化输出。

详见 [示例](https://mingling-rs.github.io/mingling/docs/example-viewer.html?name=example-structural-renderer)

## 特性 `structural_renderer_empty`

**介绍:**

启用通用渲染器的空实现版本，适用于不需要实际渲染功能的场景。此特性不启用任何 serde 格式化后端。

## 特性 `structural_renderer_full`

**介绍:**

启用通用渲染器的完整实现，包含所有渲染功能和序列化格式支持。开启此特性将自动启用 `all_serde_fmt`。

## 特性 `json_serde_fmt`

**介绍:**

启用 JSON 格式的 serde 序列化/反序列化格式化支持。

## 特性 `nightly`

**介绍:**

启用仅在不稳定（Nightly）Rust 编译器中可用的实验性功能。

## 特性 `pathf`

> [!IMPORTANT]
>
> 此特性为 **实验性功能**，API 可能在后续版本中发生变化。

**介绍:**

启用模块路径分析器（Module Pathfinder），在构建期自动解析所有 Mingling 类型的完整模块路径，生成 `use` 语句映射文件供 `gen_program!()` 消费。

开启后，类型可以定义在任意子模块中，`gen_program!()` 无需手动 `use` 即可自动识别并生成正确的完整路径引用。

```toml
# Cargo.toml
[dependencies.mingling]
features = ["pathf"]
 
[build-dependencies.mingling]
features = ["builds", "pathf"]
```
 
```rust
// BUILD TIME
// Features: ["pathf"]
analyze_and_build_type_mapping().unwrap();
```
 
详见 [示例](https://mingling-rs.github.io/mingling/docs/example-viewer.html?name=example-pathfinder)

## 特性 `parser`

**介绍:**

启用参数解析器模块，提供参数解析功能。

开启后可以使用 `Picker` 进行简易的参数提取，支持 `pick()` 和 `pick_or()` 等方法。

详见 [示例](https://mingling-rs.github.io/mingling/docs/example-viewer.html?name=example-argument-parse)

## 特性 `picker`

**介绍：**

引入依赖 `arg-picker`，为 Mingling 提供更高级的参数解析能力。

它可以与 `parser`、`clap` 特性共存，但建议不要和 `parser` 特性同时启用，因为两者的 API 极为相似。

`picker` 是独立于 Mingling 的参数解析器，不依赖 `mingling_core` 的内置参数提取 API。

详见 [示例](https://mingling-rs.github.io/mingling/docs/example-viewer.html?name=example-argument-picker)

## 特性 `repl`

**介绍:**

启用交互式 REPL（Read-Eval-Print Loop）环境支持。

开启后，可以通过 `program.exec_repl()` 将 CLI 转变为交互式 shell。

详见 [示例](https://mingling-rs.github.io/mingling/docs/example-viewer.html?name=example-repl-basic)

## 特性 `ron_serde_fmt`

**介绍:**

启用 RON（Rusty Object Notation）格式的 serde 序列化/反序列化格式化支持。

## 特性 `toml_serde_fmt`

**介绍:**

启用 TOML 格式的 serde 序列化/反序列化格式化支持。

## 特性 `yaml_serde_fmt`

**介绍:**

启用 YAML 格式的 serde 序列化/反序列化格式化支持。

<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
