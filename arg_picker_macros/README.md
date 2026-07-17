# Mingling Picker Macros

Procedural macros for [Mingling Picker](https://github.com/mingling-rs/mingling/tree/main/mingling_picker), enabled by the `mingling/picker` feature.

```toml
[dependencies.mingling]
version = "0.3.0"
features = [
    "picker"
]
```

## Provided Macros

### Macro `arg!`

Declares a parameter definition for use with `Picker`'s `.pick()` method:

```rust,ignore
use mingling_picker_macros::arg;

// Named flag with a value
let flag = arg![name: String];

// Named flag with short form
let flag = arg![name: String, 'n'];

// Named flag with alias
let flag = arg![name: String, 'n', "nickname"];

// Positional parameter
let flag = arg![String];

// Flag-only parameter (boolean)
let flag = arg![verbose: Flag];
```

### Macro `internal_repeat!` (Internal)

Internal macro used by Picker to generate `PickerPattern1..=32` and their parsing logic. Not intended for direct use.
