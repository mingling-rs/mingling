<h1 align="center">资源系统</h1>
<p align="center">
    Mingling 如何管理全局状态
</p>

命令行程序经常需要共享一些全局的东西——配置文件、数据库连接、计数器、当前工作目录。

在普通 Rust 里你可能会用 `OnceCell` 或 `lazy_static`，在 Mingling 里有一套统一的机制：**资源系统**。

## 什么是资源？

资源就是在多个 Chain 和 Renderer 之间共享的数据。

你只需要定义一个类型、注册到 Program，然后在函数签名里声明你需要它——剩下的注入和生命周期管理都由框架完成。

## 核心机制：ResourceMarker

任何同时实现了 `Default + Clone` 的类型都可以自动成为资源。框架会为它实现 `ResourceMarker` trait，使其具备：

- **`res_clone()`** —— 当多个 Chain 同时访问时，框架可以通过 clone 来避免锁竞争
- **`res_default()`** —— 资源未注册时提供兜底值

如果你需要更精细的生命周期控制，可以使用 `LazyRes<T>`。它允许资源在第一次被访问时才初始化，并且可以在析构时执行回调（比如退出前保存状态到磁盘）。

## 为什么不用全局变量？

传统做法的静态变量是隐式依赖 —— 你看函数签名根本不知道它用了什么全局状态。而 Mingling 的资源注入让 **依赖显式化**：

- 函数需要什么资源，参数列表就写什么
- `&T` 表示只读访问，`&mut T` 表示可修改
- 调用者一眼就能看出这个函数的副作用

例如：

```rust
@@@ use mingling::res::ResExitCode;
@@@ pack!(ErrorFileNotFound = ());
#[chain]
fn handle_error_file_not_found(
    error: ErrorFileNotFound,
    ec: &mut ResExitCode // 通过签名可以看出副作用！
) {
    ec.exit_code = 2; // 这里修改了退出码
}
```
 
## 资源与 Setup 的关系

资源通常通过两个途径注册到 Program：

1. **直接注册** —— 在 `main` 中调用 `program.with_resource(...)`
2. **通过 Setup** —— 使用 `DirectoryEnvironmentSetup` 等内置 Setup 批量注册（如 `ResCurrentDir`、`ResHomeDir`）

Setup 是比资源更高层的抽象，一个 Setup 可以注册多个资源并做其他初始化工作。

详见教程中的 [程序装配](./pages/8-setup-and-resources) 一章。

<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
