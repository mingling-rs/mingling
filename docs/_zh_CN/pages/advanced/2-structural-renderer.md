<h1 align="center">结构化渲染</h1>
<p align="center">
    使用 structural_renderer 特性将结果渲染为序列化文本
</p>

启用 `structural_renderer` 后，你的程序可以通过 `--json`、`--yaml` 等参数将输出切换为结构化格式，方便与其他工具集成。

## 开启特性

```toml
[dependencies.mingling]
features = ["structural_renderer"]
```
 
`structural_renderer` 会自动启用 `json_serde_fmt`。

如果需要更多格式，可以启用 `structural_renderer_full`（包含 JSON、YAML、TOML、RON）。

> [!NOTE]
> 若需要定制输出类型，可以查看 [特性](./pages/other/features)

## 基本用法

启用 `StructuralRendererSetup` 后，用 `#[derive(StructuralData)]`（搭配 `serde::Serialize`、`Grouped` 和 `Wrap`）来声明支持结构化输出的类型：

```rust
// Features: ["structural_renderer"]
// Dependencies:
// serde = "1"
@@@use mingling::macros::buffer;
@@@use mingling::setup::StructuralRendererSetup;
@@@use mingling::StructuralData;
@@@dispatcher!("render", EntryRender);
 
// #[derive(Grouped, Wrap)] + StructuralData + serde::Serialize 等价于旧版的 pack_structural!
#[derive(serde::Serialize, StructuralData, Grouped, Wrap)]
pub struct ResultInfo((String, i32));
 
#[chain]
fn handle_render(args: EntryRender) -> Next {
    let name = args.0.first().cloned().unwrap_or_default();
    let age = args.0.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    ResultInfo((name, age)).into()
}
 
#[renderer(buffer)]
fn render_info(r: ResultInfo) {
    r_println!("{:?}", *r);
}
```
 
运行效果：

```text
~# my-cli render Bob 22
("Bob", 22)
 
~# my-cli render Bob 22 --json
{"inner":["Bob",22]}
```
 
用户传入 `--json` 时，框架自动将渲染结果序列化为 JSON，无需修改业务代码。

## 自定义输出结构

用 `#[derive(Grouped, Wrap)]` 包装的元组结构体默认输出包含 `inner` 字段。要完全控制输出结构，可以用 `#[derive(StructuralData, Serialize, Grouped)]` 手动定义类型：

```rust
// Features: ["structural_renderer"]
// Dependencies:
// serde = "1"
@@@use mingling::macros::buffer;
@@@use mingling::prelude::*;
@@@use mingling::setup::StructuralRendererSetup;
@@@use mingling::StructuralData;
@@@use serde::Serialize;
@@@dispatcher!("render", EntryRender);
 
#[derive(Serialize, StructuralData, Grouped)]
struct Info {
    name: String,
    age: i32,
}
 
#[chain]
fn handle_render(args: EntryRender) -> Next {
    let name = args.0.first().cloned().unwrap_or_default();
    let age = args.0.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    Info { name, age }.to_render()
}
 
#[renderer(buffer)]
fn render_info(info: Info) {
    r_println!("{} is {} years old", info.name, info.age);
}
@@@
@@@fn main() {
@@@    let mut program = ThisProgram::new();
@@@    program.with_setup(StructuralRendererSetup);
@@@    program.exec();
@@@}
@@@gen_program!();
```
 
这时 `--json` 输出：

```json
{ "name": "Bob", "age": 22 }
```
 
## 注意事项

- 支持格式：JSON、YAML、TOML、RON（取决于启用的特性）
- `StructuralRendererSetup` 注册 `--json`、`--yaml`、`--toml`、`--ron` 等全局参数

> [!NOTE]
> 每个类型仍需一个**空的 Renderer**，否则该类型 **不被视为可渲染**

详见 [example-structural-renderer](https://mingling-rs.github.io/mingling/docs/example-viewer.html?name=example-structural-renderer)。

<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
