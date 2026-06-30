<h1 align="center">Hook System</h1>
<p align="center">
  How to insert custom behavior into a program using ProgramHook
</p>

Hooks let you insert custom logic at various lifecycle points in the pipeline — before dispatch, after chain, before/after render, on program exit …

You can write cross-cutting concerns (logging, auth, metrics collection) in hooks instead of scattering them across business code.

## Basic Usage

`ProgramHook` uses a builder pattern:

```rust
@@@use mingling::hook::ProgramHook;
fn main() {
    let mut program = ThisProgram::new();
    program.with_hook(
        ProgramHook::empty()
            .on_pre_chain(|info| {
                println!("before chain: {}", info.input);
            })
            .on_post_render(|info| {
                println!("after render: {}", info.result);
            }),
    );
    program.exec_and_exit();
}
```
 
> [!TIP]
> `ProgramHook::empty()` creates an empty hook, then chain-calls `.on_*()` methods to register the lifecycle nodes you care about. Unregistered nodes won't execute.

## Lifecycle Nodes

Hooks cover the full pipeline lifecycle:

| Stage        | Hook               | Trigger Point       |
| ------------ | ------------------ | ------------------- |
| **Dispatch** | `on_begin`         | Execution start     |
|              | `on_pre_dispatch`  | Before dispatch     |
|              | `on_post_dispatch` | After dispatch      |
| **Chain**    | `on_pre_chain`     | Before chain exec   |
|              | `on_post_chain`    | After chain exec    |
| **Render**   | `on_pre_render`    | Before render exec  |
|              | `on_post_render`   | After render exec   |
| **Finish**   | `on_finish`        | Before program exit |

Each hook callback receives a corresponding `Hook*Info` struct containing context info (input type, params, render results, etc.).

## Real Example: Logging Operations

```rust
@@@use mingling::prelude::*;
@@@use mingling::hook::ProgramHook;
@@@
@@@dispatcher!("greet", CMDGreet => EntryGreet);
@@@pack!(ResultName = String);
@@@
@@@#[chain] fn handle_greet(args: EntryGreet) -> Next {
@@@    ResultName::new(args.inner.first().cloned().unwrap_or_default()).to_render()
@@@}
@@@#[renderer] fn render_name(r: ResultName) { r_println!("Hello, {}!", *r); }
fn main() {
    let mut program = ThisProgram::new();
 
    // Log info before and after each chain execution
    program.with_hook(
        ProgramHook::empty()
            .on_pre_chain(|info| {
                eprintln!("[hook] executing chain for: {}", info.input);
            })
            .on_post_chain(|info| {
                eprintln!("[hook] chain output: {}", info.output.member_id);
            }),
    );
 
    program.with_dispatcher(CMDGreet);
    program.exec_and_exit();
}
```
 
Run output:

```text
[hook] executing chain for: EntryGreet
[hook] chain output: ResultName
Hello, World!
```
 
## Controlling Behavior via Hooks

Hooks aren't just for observation — you can use `ProgramControlUnit` to alter program behavior:

| Variant                    | Effect                                    |
| -------------------------- | ----------------------------------------- |
| `Continue`                 | Do nothing, continue execution            |
| `OverrideExitCode(i32)`    | Override the exit code                    |
| `RouteToChain(AnyOutput)`  | Replace current data, re-enter Chain loop |
| `RouteToRender(AnyOutput)` | Skip subsequent Chain, render directly    |

> [!NOTE]
> Multiple hooks can be registered and execute in registration order.

<p align="center" style="font-size: 0.85em; color: gray;">
  Written by @Weicao-CatilGrass
</p>
