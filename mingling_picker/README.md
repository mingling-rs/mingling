# Mingling Picker

A command-line argument parser for [Mingling](https://github.com/mingling-rs/mingling), enabled by the `mingling/picker` feature.

```toml
[dependencies.mingling]
version = "0.3.0"
features = [
    "picker"
]
```

## Chained Argument Parser

Provides a clean chained-call API for declaring arguments to parse:

```rust
let args: Vec<&str> = vec!["--name", "Bob", "--age", "24"];
let result = args
    .pick(&flag![name: String])
    .or(|| "Alice".to_string())
    .pick(&flag![age: i32])
    .or(|| 24)
    .post(|num| num.clamp(0, 120))
    .parse();
let (name, age): (String, i32) = a.unwrap();
```

## Parsing Function Library

Provides a pure function library `parselib` for analyzing the structure of command-line arguments.

```rust
use mingling::picker::parselib::*;
```
