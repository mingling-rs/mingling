<h1 align="center">Structural Rendering</h1>
<p align="center">
    Use the <code>structural_renderer</code> feature to render output as serialized text
</p>

With `structural_renderer` enabled, your program can switch output to a structured format via `--json`, `--yaml`, etc., making it easy to integrate with other tools.

## Enabling the Feature

```toml
[dependencies.mingling]
features = ["structural_renderer"]
```
 
`structural_renderer` automatically enables `json_serde_fmt`.

For more formats, enable `structural_renderer_full` (includes JSON, YAML, TOML, RON).

> [!NOTE]
> To customize output types, see [Features](./pages/other/features)

## Basic Usage

After enabling `StructuralRendererSetup`, use `pack_structural!` instead of `pack!` to declare types that support structured output:

```rust
// Features: ["structural_renderer"]
// Dependencies:
// serde = "1"
@@@use mingling::setup::StructuralRendererSetup;
@@@dispatcher!("render", CMDRender => EntryRender);
 
// pack_structural! is equivalent to pack! + StructuralData
pack_structural!(ResultInfo = (String, i32));
 
#[chain]
fn handle_render(args: EntryRender) -> Next {
    let name = args.inner.first().cloned().unwrap_or_default();
    let age = args.inner.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    ResultInfo::new((name, age)).into()
}
 
#[renderer]
fn render_info(r: ResultInfo) -> RenderResult {
    let mut result = RenderResult::new();
    writeln!(result, "{:?}", *r).ok();
    result
}
```
 
Output:

```text
~# my-cli render Bob 22
("Bob", 22)
 
~# my-cli render Bob 22 --json
{"inner":["Bob",22]}
```
 
When the user passes `--json`, the framework automatically serializes the render result as JSON — no business logic changes needed.

## Customizing Output Structure

The default output from `pack_structural!` includes an `inner` field. For full control over the output structure, define the type manually with `#[derive(StructuralData, Serialize, Groupped)]`:

```rust
// Features: ["structural_renderer"]
// Dependencies:
// serde = "1"
@@@use mingling::prelude::*;
@@@use mingling::setup::StructuralRendererSetup;
@@@use mingling::StructuralData;
@@@use serde::Serialize;
@@@dispatcher!("render", CMDRender => EntryRender);
 
#[derive(Serialize, StructuralData, Groupped)]
struct Info {
    name: String,
    age: i32,
}
 
#[chain]
fn handle_render(args: EntryRender) -> Next {
    let name = args.inner.first().cloned().unwrap_or_default();
    let age = args.inner.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    Info { name, age }.to_render()
}
 
#[renderer]
fn render_info(info: Info) -> RenderResult {
    let mut r = RenderResult::new();
    writeln!(r, "{} is {} years old", info.name, info.age).ok();
    r
}
@@@
@@@fn main() {
@@@    let mut program = ThisProgram::new();
@@@    program.with_setup(StructuralRendererSetup);
@@@    program.with_dispatcher(CMDRender);
@@@    program.exec();
@@@}
@@@gen_program!();
```
 
Now `--json` outputs:

```json
{ "name": "Bob", "age": 22 }
```
 
## Notes

- Supported formats: JSON, YAML, TOML, RON (depends on enabled features)
- `StructuralRendererSetup` registers global params like `--json`, `--yaml`, `--toml`, `--ron`

> [!NOTE]
> Each type still needs an **empty Renderer**, otherwise that type **is not considered renderable**

See [example-structural-renderer](https://mingling-rs.github.io/mingling/docs/example-viewer.html?name=example-structural-renderer).

<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
