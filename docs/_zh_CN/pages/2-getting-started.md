<h1 align="center">起步</h1>

## 创建一个新项目

```bash
cargo new my-cli
cd my-cli
```

## 添加依赖

在 `Cargo.toml` 写入如下内容

```toml
[dependencies.mingling]
version = "0.2"
features = []
```

## 启用特性

**Mingling** 默认所有特性关闭，且不提供类似 `full` 的全开特性。

因为部分特性会 **直接影响整个生命周期的行为**，需要你按需启用，例如：

```toml
[dependencies.mingling]
version = "0.2"
features = [
    "parser",
    "comp",
]
```

> [!NOTE]
> 请前往 [docs.rs](https://docs.rs/mingling/latest/mingling/feature/index.html) 或 [特性](pages/other/features) 以了解所有特性

## 编写基本入口

编写 `src/main.rs`，写入以下代码：

```rust
use mingling::prelude::*;

fn main() {
    let mut program = ThisProgram::new();
    
    program.exec_and_exit();
}

gen_program!();
```

## 编译验证

```plaintext
~# cargo check
```

---

一切无误后，开始写点什么吧！
