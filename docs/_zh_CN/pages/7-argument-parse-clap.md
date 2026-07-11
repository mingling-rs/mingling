<h1 align="center">使用 Clap 完成参数解析</h1>
<p align="center">
    用 clap 做更复杂的参数解析
</p>

Picker 适合轻量级的参数提取，但当参数数量多、有复杂的校验规则、或者需要自动生成 `--help` 时，可以接入 [clap](https://crates.io/crates/clap)。

## 开启 clap 特性

```toml
[dependencies.mingling]
features = ["clap"]
 
[dependencies.clap]
version = "4"
features = ["derive", "color"]
```
 
## dispatcher_clap

`#[dispatcher_clap]` 加在 `clap::Parser` 结构体上，自动生成 Dispatcher：

```rust
// Features: ["clap"]
// Dependencies:
// clap = "4"
@@@ use mingling::macros::dispatcher_clap;
#[derive(Default, clap::Parser, Groupped)]
#[dispatcher_clap("greet", CMDGreet, help = true, error = ErrorGreetParsed)]
pub struct EntryGreet {
    #[clap(default_value = "World")]
    name: String,
    #[arg(short, long, default_value_t = 1)]
    repeat: i32,
}
 
#[renderer]
fn render_greet(greet: EntryGreet) -> RenderResult {
    let mut r = RenderResult::new();
    let count = greet.repeat.max(0) as usize;
    write!(r, "Hello, ").ok();
    for _ in 0..count {
        write!(r, "{} ", greet.name).ok();
    }
    writeln!(r, "!").ok();
    r
}
 
#[renderer]
fn render_greet_parse_failed(err: ErrorGreetParsed) -> RenderResult {
    let mut r = RenderResult::new();
    writeln!(r, "{}", *err).ok();
    r
}
```
 
## 与 BasicProgramSetup 配合

如果需要 `--help` 支持，在 main 中注册 `BasicProgramSetup` 并设置 clap 帮助的输出模式：

```rust
// Features: ["clap"]
// Dependencies:
// clap = "4"
@@@use mingling::setup::BasicProgramSetup;
@@@use mingling::macros::dispatcher_clap;
@@@#[derive(Default, clap::Parser, Groupped)]
@@@#[dispatcher_clap("greet", CMDGreet)]
@@@pub struct EntryGreet {
@@@    name: String,
@@@}
@@@#[renderer]
@@@fn render_greet(greet: EntryGreet) -> RenderResult {
@@@    let mut r = RenderResult::new();
@@@    write!(r, "Hello, {}!", greet.name).ok();
@@@    r
@@@}
fn main() {
    let mut program = ThisProgram::new();
    program.with_setup(BasicProgramSetup);
    program.stdout_setting.clap_help_print_behaviour =
        mingling::ClapHelpPrintBehaviour::WriteToRenderResult;
    program.with_dispatcher(CMDGreet);
    program.exec_and_exit();
}
```
 
详见 [example-clap-binding](https://mingling-rs.github.io/mingling/docs/example-viewer.html?name=example-clap-binding)。

<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
