# Argument Picker

A command-line argument parser for [Mingling](https://github.com/mingling-rs/mingling), enabled by the `mingling/picker` feature.

```toml
[dependencies.mingling]
version = "0.3.0"
features = [
    "picker"
]
```

Of course, you can also use it as a standalone crate by replacing `mingling::picker` with `arg_picker`:

```toml
[dependencies]
arg-picker = "0.1.1"
```

## Chained Argument Parser

Provides a clean chained-call API for declaring arguments to parse:

```rust
use arg_picker::prelude::*;

let args: Vec<&str> = vec!["--name", "Bob", "--age", "24"];

let (name, age) = args
    .pick(&arg![name: String])
    .or(|| "Alice".to_string())
    .pick(&arg![age: i32])
    .or(|| 24)
    .post(|num| num.clamp(0, 120))
    .unwrap();

assert_eq!(name, "Bob".to_string());
assert_eq!(age, 24);
```

## Parsing Function Library

Provides a pure function library `parselib` for analyzing the structure of command-line arguments.

```rust
use arg_picker::parselib::*;
```
