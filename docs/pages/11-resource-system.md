<h1 align="center">Using the Resource System</h1>
<p align="center">
    A hands-on guide to resources
</p>

Resources are Mingling's mechanism for managing global state. Any type that implements `Default + Clone` can be a resource.

## Define a Resource

```rust
// Any type implementing Default + Clone can be used as a resource
#[derive(Default, Clone)]
struct ResCurrentDir(String);
 
// Register with the Program
fn main() {
    let mut program = ThisProgram::new();
    program.with_resource(ResCurrentDir(".".into()));
    program.exec_and_exit();
}
```
 
Since `ResCurrentDir` implements both `Default` and `Clone`, the framework automatically implements `ResourceMarker` for it — no manual impl needed.

## Inject & Use

In a Chain or Renderer, simply declare the resource in the parameter list:

```rust
@@@#[derive(Default, Clone)]
@@@struct ResCurrentDir(String);
@@@dispatcher!("pwd", CMDPrintWorkingDir => EntryPrintWorkingDir);
@@@pack!(ResultPath = String);
// Inject read-only resource via &T
#[chain]
fn handle_pwd(_args: EntryPrintWorkingDir, cwd: &ResCurrentDir) -> Next {
    ResultPath::new(cwd.0.clone()).to_render()
}
 
#[renderer]
fn render_path(result: ResultPath) -> RenderResult {
    let mut r = RenderResult::new();
    writeln!(r, "{}", *result).ok();
    r
}
```
 
## Modify a Resource

Use `&mut T` to inject a mutable resource:

```rust
@@@#[derive(Default, Clone)]
@@@struct ResVisitCount(u32);
@@@dispatcher!("visit", CMDVisit => EntryVisit);
@@@pack!(ResultDone = ());
#[chain]
fn handle_visit(_args: EntryVisit, counter: &mut ResVisitCount) -> Next {
    counter.0 += 1;
    ResultDone::default().into()
}
 
#[renderer]
fn render_done(_done: ResultDone, counter: &ResVisitCount) -> RenderResult {
    let mut r = RenderResult::new();
    writeln!(r, "visit count is : {}", counter.0).ok();
    r
}
```
 
## Use Multiple Resources

A Chain can inject any number of resources at once — the framework matches them by type automatically:

```rust
@@@#[derive(Default, Clone)] struct ResConfig(String);
@@@#[derive(Default, Clone)] struct ResCounter(u32);
@@@dispatcher!("test", CMDTest => EntryTest);
@@@pack!(ResultDone = ());
// Inject both read-only and mutable resources
#[chain]
fn handle_test(_args: EntryTest, config: &ResConfig, counter: &mut ResCounter) -> Next {
    println!("config: {}", config.0);
    counter.0 += 1;
    ResultDone::default().to_render()
}
```
 
For deeper topics like `ResourceMarker`, lazy loading with `LazyRes`, etc., see [Core Concepts: Resource System](pages/concepts/2-resource).

<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
