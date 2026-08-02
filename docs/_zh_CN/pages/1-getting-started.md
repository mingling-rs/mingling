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
version = "0.3.0"
features = []
```
 
## 启用特性

**Mingling** 默认只启用 `core` 和 `macros`，其余部分需要按需启用

因为部分特性会 **直接影响整个生命周期的行为**，需要你按需启用，例如：

```toml
[dependencies.mingling]
version = "0.3.0"
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
 
> [!IMPORTANT]
> 文档中几乎所有 Rust 代码块都已在 CI 流程中编译通过，可以保证可用性。
>
> 但以 `// NOT VERIFIED` 开头的代码块 **未被验证**。
>
> 想确认哪些 `*.md` 文件被编译过？请看 [`verified-docs.toml`](https://github.com/mingling-rs/mingling/blob/main/verified-docs.toml)

## 编译验证

```plaintext
~# cargo check
```
 
---

一切无误后，开始写点什么吧！
