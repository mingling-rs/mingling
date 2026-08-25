<h1 align="center">配置项目</h1>
<p align="center">
    通过 <code>Cargo.toml</code> 配置 Mingling 项目
</p>

欢迎使用 Mingling！

本页面将带领你快速了解并构建你的第一款 Mingling CLI 程序，
你可以跟着文章内容一步一步操作，逐步搭建你的 Mingling 程序。

## 1. 检查 `rustc` 版本

`Mingling` 基于 Rust 1.95.0 开发，请确认你的 Rust 工具链版本是否满足要求：

```bash
rustc --version # 检查 rustc 版本
# 需要 >= 1.95.0
```
 
若你的 `rustc` 不满足要求，请使用 `rustup update` 命令安装或更新你的 Rust 工具链到 `1.95.0`：

```bash
rustup update # 更新 rust 工具链版本
```
 
若你尚未安装 `rustup`，请先访问 [rustup.rs](https://rustup.rs) 安装 `rustup` 工具链管理器，然后再执行上述命令。

## 2. 将 `mingling` 添加到你的项目中

创建项目后，使用 `cargo add` 将 `mingling` 添加到你的依赖中：

```bash
cargo add mingling@0.5.0 --features mini # 添加 Mingling 并启用最小特性集
```
 
或者直接编辑 `Cargo.toml`，在最底部添加如下内容

```toml
[dependencies.mingling] # 使用单独行注册 Mingling 依赖
version = "0.5.0" # 指定 Mingling 版本
features = [ "mini" ] # 开启 `mini` 特性组，提供参数解析和补全的能力
```
 
## 3. 添加入口代码

将 Mingling 添加到项目中后，你可以将 `main.rs` 的代码改为如下内容来快速检查框架是否可用：

```rust
// src/main.rs
use mingling::prelude::*;
 
fn main() {
    ThisProgram::new().exec_and_exit(); // 创建、运行、退出程序
}
 
#[derive(Grouped)] // 将该类型添加到 ThisProgram 中
struct ResultHelloWorld; // 创建一个结构体用于代表结果
 
#[command] // 向 ThisProgram 注册一个命令
fn greet() -> ResultHelloWorld { // 注册命令 `greet`，返回 `ResultHelloWorld`
    ResultHelloWorld // 返回 `ResultHelloWorld` 到调度器
}
 
#[renderer] // 向 ThisProgram 注册一个渲染器
fn render_hello_world(_: ResultHelloWorld) -> String { // 注册渲染器，接收 `ResultHelloWorld`，渲染为 `String`
    "Hello, World!".into() // 输出 String
}
 
gen_program!(); // 收集前文所有注册的内容，展开为 ThisProgram
```
 
编译成功后，使用 `cargo run -- greet` 你将获得如下输出：

```bash,simulation
~# cargo run -- greet
Hello, World!
```
 
---

🎉 恭喜！至此你的 `Mingling` 项目就已经构建完成了！

回顾你的项目结构，它应该是这样的：

```file-tree
/Cargo.toml # Cargo 配置文件
/Cargo.lock # Cargo 锁文件
/src/main.rs # 项目主入口
```
