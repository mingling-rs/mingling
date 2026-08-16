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

After enabling `StructuralRendererSetup`, use `#[derive(StructuralData, Grouped, Wrap)]` to declare types that support structured output:

```rust
// Features: ["structural_renderer"]
// Dependencies:
// serde = "1"
@@@use mingling::macros::buffer;
@@@use mingling::setup::StructuralRendererSetup;
@@@use mingling::StructuralData;
@@@dispatcher!("render", EntryRender);
 
// StructuralData + Grouped + Wrap gives the type structured output support
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
 
Output:

```text
~# my-cli render Bob 22
("Bob", 22)
 
~# my-cli render Bob 22 --json
{"inner":["Bob",22]}
```
 
When the user passes `--json`, the framework automatically serializes the render result as JSON — no business logic changes needed.

## Customizing Output Structure

The default output from a tuple newtype (e.g. `#[derive(StructuralData, Grouped, Wrap)]`) wraps the value under an `inner` key. For full control over the output structure, define the type manually with `#[derive(StructuralData, Serialize, Grouped)]`:

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
