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
version = "0.2"
features = []
```

## Enable Features

**Mingling** has all features disabled by default and does not provide an all-in-one feature like `full`.

Some features will **directly affect the behavior of the entire lifecycle**, so you need to enable them as needed, for example:

```toml
[dependencies.mingling]
version = "0.2"
features = [
    "parser",
    "comp",
]
```

> [!NOTE]
> Visit [docs.rs](https://docs.rs/mingling/latest/mingling/feature/index.html) or [Features](pages/other/features) to learn about all available features.

## Write the Basic Entry Point

Edit `src/main.rs` with the following code:

```rust
use mingling::prelude::*;

fn main() {
    let mut program = ThisProgram::new();

    program.exec_and_exit();
}

gen_program!();
```

## Verify Compilation

```plaintext
~# cargo check
```

---

Once everything is good, start building!
