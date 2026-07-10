<h1 align="center">About Nightly Features</h1>
<p align="center">
    Using nightly Rust features in Mingling
</p>

**Mingling** uses some features that are only available in the `nightly` toolchain. This requires you to enable the `nightly` feature:

```toml
[dependencies]
mingling = { version = "...", features = ["nightly"] }
```
 
## Features

> [!WARNING]
> The following features can only be used with the nightly toolchain, and are only guaranteed to compile, not to be stable or production-ready.
>
> If you need a stable development experience, please **do not use** the `nightly` feature!
