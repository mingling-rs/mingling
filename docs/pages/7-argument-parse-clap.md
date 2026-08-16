<h1 align="center">Parsing Arguments with Clap</h1>
<p align="center">
    Use clap for more complex argument parsing
</p>

Picker is suitable for lightweight arg extraction, but when there are many args, complex validation rules, or you need auto-generated `--help`, you can use [clap](https://crates.io/crates/clap).

## Enable clap feature

```toml
[dependencies.mingling]
features = ["clap"]
 
[dependencies.clap]
version = "4"
features = ["derive", "color"]
```
 
## dispatcher_clap

Add `#[dispatcher_clap]` on a `clap::Parser` struct to auto-generate a Dispatcher:

```rust
// Features: ["clap"]
// Dependencies:
// clap = "4"
@@@ use mingling::macros::dispatcher_clap;
@@@ use mingling::macros::buffer;
#[derive(Default, clap::Parser, Grouped)]
#[dispatcher_clap("greet", CMDGreet, help = true, error = ErrorGreetParsed)]
pub struct EntryGreet {
    #[clap(default_value = "World")]
    name: String,
    #[arg(short, long, default_value_t = 1)]
    repeat: i32,
}
 
#[renderer(buffer)]
fn render_greet(greet: EntryGreet) {
    let count = greet.repeat.max(0) as usize;
    r_print!("Hello, ");
    for _ in 0..count {
        r_print!("{} ", greet.name);
    }
    r_println!("!");
}
 
#[renderer(buffer)]
fn render_greet_parse_failed(err: ErrorGreetParsed) {
    r_println!("{}", *err);
}
```
 
## Working with BasicProgramSetup

If you need `--help` support, register `BasicProgramSetup` in main and set the clap help output mode:

```rust
// Features: ["clap"]
// Dependencies:
// clap = "4"
@@@use mingling::macros::buffer;
@@@use mingling::setup::BasicProgramSetup;
@@@use mingling::macros::dispatcher_clap;
@@@#[derive(Default, clap::Parser, Grouped)]
@@@#[dispatcher_clap("greet", CMDGreet)]
@@@pub struct EntryGreet {
@@@    name: String,
@@@}
@@@#[renderer(buffer)]
@@@fn render_greet(greet: EntryGreet) {
@@@    r_println!("Hello, {}!", greet.name);
@@@}
fn main() {
    let mut program = ThisProgram::new();
    program.with_setup(BasicProgramSetup);
    program.stdout_setting.clap_help_print_behaviour =
        mingling::config::ClapHelpPrintBehaviour::WriteToRenderResult;
    program.exec_and_exit();
}
```
 
See [example-clap-binding](https://mingling-rs.github.io/mingling/docs/example-viewer.html?name=example-clap-binding) for more details.

<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
