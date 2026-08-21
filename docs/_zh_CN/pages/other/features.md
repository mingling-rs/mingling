<h1 align="center">特性</h1>
<p align="center">
    <b>Mingling</b> 的所有特性一览
</p>

# 预设特性组

Mingling 提供了一系列**预设特性组**，方便用户按需组合启用特性。

## `mini`

**启用特性：** `picker`、`comp`

**定位：** 精简模式，适合小型 CLI 工具或需要快速起步的项目。仅包含参数解析和代码补全能力。

## `advanced`

**启用特性：** `picker`、`repl`、`comp`、`structural_renderer`、`pathf`

**定位：** 进阶模式，在 `mini` 的基础上加入了交互式 REPL 环境、基础的结构化输出能力以及实验性的路径分析器，适合功能较完整的中型命令行应用。

## `full`

**启用特性：** `picker`、`repl`、`clap`、`comp`、`structural_renderer_full`

**定位：** 完整模式，启用 Mingling 的核心功能。在 `advanced` 的基础上替换为完整的结构化渲染器（含所有序列化格式），并额外包含 clap 集成，适合大型、功能全面的命令行应用。

# 特性详解

## 特性 `all_serde_fmt`

**介绍:**

为 `structural_renderer` 启用所有序列化格式（JSON、RON、TOML、YAML）的 serde 格式化支持。

开启此特性将自动启用 `json_serde_fmt`、`ron_serde_fmt`、`toml_serde_fmt`、`yaml_serde_fmt` 四个子特性。

## 特性 `async`

**介绍:**

启用异步运行时支持，允许 `#[chain]` 绑定 `async` 函数，例如：

```rust
// Features: ["async"]
 
#[derive(Grouped, Wrap)]
pub struct StateFoo(());
 
#[chain]
async fn handle_state_foo(foo: StateFoo) -> Next {
    StateFoo(()).into()
}
```
 
详见 [示例](https://mingling-rs.github.io/mingling/docs/example-viewer.html?name=example-async-support)

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
```
 
开启 `pathf` 特性后，`gen_program!()` 会在编译期自动调用 `build_pathf!()` 执行类型映射分析。

详见 [示例](https://mingling-rs.github.io/mingling/docs/example-viewer.html?name=example-pathfinder)

## 特性 `picker`

**介绍：**

引入依赖 `arg-picker`，为 Mingling 提供更高级的参数解析能力。

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
