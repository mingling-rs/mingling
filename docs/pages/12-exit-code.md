<h1 align="center">Exit Code Control</h1>
<p align="center">
    Managing program exit codes via the resource system
</p>

Providing the shell with a correct exit code when a program terminates is a basic CLI convention. Mingling offers a ready-to-use `ExitCodeSetup` that, together with the `ResExitCode` resource, makes exit code control incredibly simple.

## Enabling ExitCodeSetup

```rust
@@@use mingling::prelude::*;
@@@use mingling::setup::ExitCodeSetup;
fn main() {
    let mut program = ThisProgram::new();
    program.with_setup(ExitCodeSetup::default());
@@@ program.exec_and_exit();
}
```
 
`ExitCodeSetup` does two things:

1. Registers the `ResExitCode` resource (default value `0`)
2. Registers a `finish` hook that reads the value of `ResExitCode` as the final exit code before the program exits

## Modifying the Exit Code

In a Chain or Renderer, inject `ResExitCode` to modify the exit code:

```rust
@@@use mingling::res::ResExitCode;
@@@use mingling::setup::ExitCodeSetup;
@@@pack!(EntryCheck = Vec<String>);
#[chain]
fn handle_check(_args: EntryCheck, ec: &mut ResExitCode) {
    // Modify exit code when check fails
    ec.exit_code = 1;
}
```
 
> [!TIP]
> `ResExitCode` is simply `struct ResExitCode { pub exit_code: i32 }`. Inject `&mut ResExitCode` and modify the field directly.

## Three Execution Modes of `Program`

`Program` provides three execution modes (excluding `exec_repl` under the `repl` feature):

| Mode                            | Behavior                                                                                  |
| ------------------------------- | ----------------------------------------------------------------------------------------- |
| `program.exec_and_exit()`       | Executes and terminates the process directly with the exit code                           |
| `program.exec()`                | Executes and returns an `i32` exit code, letting the caller decide handling               |
| `program.exec_without_render()` | Returns `Result<RenderResult, ProgramExecuteError>`, with internal `exit_code` accessible |

```rust
@@@use mingling::setup::ExitCodeSetup;
fn main() {
    let mut program = ThisProgram::new();
    program.with_setup(ExitCodeSetup::default());
 
    // Get exit code and handle it yourself
    let exit_code = program.exec();
    std::process::exit(exit_code);
}
@@@gen_program!();
```
 
<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
