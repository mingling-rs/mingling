<h1 align="center">Getting Started</h1>

## Create a New Project

```bash
cargo new my-cli
cd my-cli
```
 
## Add Dependency

Add the following to `Cargo.toml`:

```toml
[dependencies.mingling]
version = "0.2.0"
features = []
```
 
## Enable Features

**Mingling** has all features disabled by default and does **not** provide an all-in-one feature like `full`.

Some features **directly affect the entire lifecycle behavior**, so you need to enable them as needed, e.g.:

```toml
[dependencies.mingling]
version = "0.2.0"
features = [
    "parser",
    "comp",
]
```
 
> [!NOTE]
> Visit [docs.rs](https://docs.rs/mingling/latest/mingling/feature/index.html) or [Features](pages/other/features) to learn about all features.

## Write the Basic Entry Point

Write the following code in `src/main.rs`:

```rust
use mingling::prelude::*;
 
fn main() {
    let mut program = ThisProgram::new();
 
    program.exec_and_exit();
}
 
gen_program!();
```
 
> [!IMPORTANT]
> Almost all Rust code blocks in the docs have been compiled in CI and are guaranteed to work.
>
> However, code blocks starting with `// NOT VERIFIED` are **not verified**.
>
> Want to know which `*.md` files are compiled? See [`verified-docs.toml`](https://github.com/mingling-rs/mingling/blob/main/verified-docs.toml).

## Verify with Compilation

```plaintext
~# cargo check
```
 
---

Once everything is good, start writing something!
