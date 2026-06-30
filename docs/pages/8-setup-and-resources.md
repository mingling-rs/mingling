<h1 align="center">Program Setup</h1>
<p align="center">
    Initialize your program with Setup
</p>

When a program needs to do some init work at startup—like parsing global args or registering resources—you can organize that logic with `#[program_setup]`.

## Initialize with Setup

```rust
// Features: ["extra_macros"]
@@@use mingling::macros::program_setup;
@@@use mingling::Program;
#[program_setup]
fn my_setup(program: &mut Program<ThisProgram>) {
    // Extract global flag from args
    program.global_flag(["-v", "--verbose"], |program| {
        program.stdout_setting.verbose = true;
    });
}
@@@
@@@fn main() {
@@@    let mut program = ThisProgram::new();
@@@    program.with_setup(MySetup);
@@@    program.exec_and_exit();
@@@}
@@@gen_program!();
```
 
A function annotated with `#[program_setup]` receives `&mut Program<ThisProgram>`, where you can do any init work.

Register it in `main` via `program.with_setup(...)` to use it.

> [!NOTE]
> `#[program_setup]` requires the `extra_macros` feature. Without it, you can manually implement the `ProgramSetup` trait.

## Extract Global Args

The most common use of Setup is extracting global args. Mingling provides a few helper methods:

```rust
// Features: ["extra_macros"]
@@@use mingling::macros::program_setup;
@@@use mingling::Program;
#[program_setup]
fn my_setup(program: &mut Program<ThisProgram>) {
    // Boolean flag
    program.global_flag(["-v", "--verbose"], |program| {
        program.stdout_setting.verbose = true;
    });
 
    // Flag with a value
    program.global_argument("--name", |_program, value| {
        // value is "Alice"
        let _ = value;
    });
}
```
 
> [!TIP]
> `global_flag` and `global_argument` automatically remove matched args from `program.args`, so they won't enter the pipeline.

## Built-in Setup

Mingling provides several ready-to-use Setups covering the most common CLI program needs:

| Setup                       | Functionality                                                                              |
| --------------------------- | ------------------------------------------------------------------------------------------ |
| `BasicProgramSetup`         | Parses `--help`/`-h`, `--quiet`/`-q`, `--confirm`/`-C`                                     |
| `DirectoryEnvironmentSetup` | Registers directory resources: current dir, executable dir, home dir, temp dir             |
| `ExitCodeSetup`             | Controls program exit code via `ResExitCode`                                               |
| `StructuralRendererSetup`   | Enables `--json`, `--yaml` etc. structured output (requires `structural_renderer` feature) |

Usage is just one line in `main`:

```rust
@@@use mingling::setup::BasicProgramSetup;
fn main() {
    let mut program = ThisProgram::new();
    program.with_setup(BasicProgramSetup);
    program.exec_and_exit();
}
```
 
`BasicProgramSetup` handles common params that most CLI programs need, saving you the trouble of manual parsing.

<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
