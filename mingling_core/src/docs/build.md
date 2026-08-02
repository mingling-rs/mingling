Provides Mingling's build script module, used in `build.rs` to provide build-time behavior for certain features.

To use it, add the following to your `Cargo.toml` under `[build-dependencies]`, and enable the features
that require build-time behavior from the crate:

```toml
# Cargo.toml
[build-dependencies.mingling]
version = "0.3.0"
features = [
    "build", # Enable it
    "comp",  # If you need completion-related build-time behavior, enable this as well
]
```
