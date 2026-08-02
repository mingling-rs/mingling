Provide Mingling's build script module for build-time behavior of specific features in `build.rs`.

To use it, add a dependency on mingling under `[build-dependencies]` in `Cargo.toml`, and enable the relevant features:

## Build-Time Related Features

| Name             | Purpose                                                                                           |
| ---------------- | ------------------------------------------------------------------------------------------------- |
| `build`          | Master switch for build-time features                                                             |
| `build_advanced` | Master switch for build-time features, paired with the `advanced` feature                         |
| `build_full`     | Master switch for build-time features, paired with the `full` feature                             |
| `comp`           | Completion script builder; both sides must enable it, generates cross-platform completion scripts |
| `pathf`          | Type path analyzer; both sides must enable it, generates type mapping tables                      |
| `dispatch_tree`  | Compile-time dispatch tree; when `pathf` is a build-time dependency,                              |
|                  | and `dispatch_tree` (included in `advanced` or `full`) is enabled, both sides should enable it    |

```toml
# Cargo.toml
[dependencies.mingling]
features = [
    "advanced", # Enable `advanced` if using it
]

[build-dependencies.mingling]
features = [
    "build_advanced" # This side should enable `build_advanced`
]
```

## `build.rs` Templates

You can use the following template to write `build.rs` to quickly gain the build-time capabilities of `comp` and `pathf`:

```rust,ignore
// build.rs
fn main() {
    build_scripts();
    build_pathf_mapping();
}

/// Generate completion scripts
fn build_scripts() {
    // `env!("CARGO_PKG_NAME")` equals the crate name, which matches the binary name.
    // If your binary name differs from the crate name, specify it explicitly.
    mingling::build::build_comp_scripts(
        // Your binary name:
        env!("CARGO_PKG_NAME"),
    )
    .unwrap();
}

fn build_pathf_mapping() {
    // Build pathf type mapping to ensure that the enabled `pathf` feature
    // can correctly scan macros in the project
    mingling::build::analyze_and_build_type_mapping().unwrap();
}
```
