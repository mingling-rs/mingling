<h1 align="center">编写程序骨架</h1>
<p align="center">
    通过 <code>gen_program!()</code> 生成程序的骨架
</p>

在上一页，我们使用 `cargo add` 创建了第一个 Mingling 程序，而在本页我们来编写实际的程序逻辑。

## 1. 准备基础代码

上一页中，我们为了快速验证 Mingling 是否安装成功，复制了一段代码到 `main.rs`，现在请将它删除，我们从头开始编写：

```rust
// main.rs
fn main () {
    println!("Hello, World");
}
```
 
## 2. 使用预导入模块

Mingling 提供了一个 `prelude` 模块，它提供了在开发 Mingling 程序时常用的组件，
我们将它导入到项目中：

```rust
use mingling::prelude::*; // 导入常用模块
```
 
## 3. 生成程序结构

然后，在程序的 **尾部** 增加 `gen_program!();` 来创建程序：

```rust
// main.rs
use mingling::prelude::*; // 导入常用模块
 
fn main () {
    println!("Hello, World");
}
 
gen_program!(); // 生成程序，它必须放在程序结尾！
```
 
> [!Important]
> `gen_program!()` **必须** 放在当前 Crate 中 `main.rs` 或 `lib.rs` 的尾部，这是 Mingling 的约定。
>
> 详见 [`gen_program!()` 为什么必须放在尾部？](#gen_program-为什么必须放在尾部？)

## 4. 生成程序

在上一步中，`gen_program!()` 已经完整地生成了一个枚举类型 `ThisProgram`

```rust
// 它生成了 ThisProgram 枚举
gen_program!(); // 展开为 crate::ThisProgram 和其具体实现
```
 
这个生成的 `ThisProgram` 将代表当前的程序，它会生成如下内容：

- 所有的命令节点以及分发逻辑
- 所有的成员 ID
- 所有的渲染调度
- 所有的逻辑调度
- 所有的帮助调度
- 所有的补全行为
- 所有的元数据查询

Mingling 将所有编译期内容集中在此处管理，它将作为程序 `编译时` 和 `运行时` 之间的桥梁。

## 5. 启动程序

最后，我们回到 `fn main()` 函数中，添加如下代码以启动程序：

```rust
fn main() {
    // 创建程序实例
    let mut program = ThisProgram::new(); // 创建 Program<ThisProgram> 而不是 ThisProgram
 
    // 启动并退出程序
    program.exec_and_exit(); // 启动程序，执行完成后自动处理结果渲染并以指定的退出码结束进程
}
```
 
完整代码：

```rust
use mingling::prelude::*;
 
fn main() {
    let mut program = ThisProgram::new();
    program.exec_and_exit();
}
 
gen_program!();
```
 
执行：（因为没有定义行为，所以它什么也没输出）

```bash,simulation
~# cargo run
~#
```
 
---

🎉 恭喜！至此你的 `Mingling` 项目已经正常运作了，没有任何输出是正常现象，在下一页中，我们将开始编写第一条子命令！

---

## 你可能还疑惑的点：

### `gen_program!()` 为什么必须放在尾部？

这是由 Rust 的 **宏展开顺序** 所决定的：

Mingling 需要在编译期知道当前程序的所有成员，而它们的注册是在其自身的宏展开时发生的，
而 `gen_program!` 的职责是收集 + 生成，它必然需要在已知全部成员的情况下展开，所以它必须放在最后。

所以 Mingling 在架构上将 `gen_program!()` 设计为：
在展开处创建一个 `ThisProgram` 的枚举类型，而其他宏的展开处则直接去满足该约定路径 `crate::ThisProgram`，
这意味着你的 `gen_program!()` 宏 **只能放在** 如下位置：

```file-tree
/src/bin/program.rs # 二进制的 rs 文件结尾
/src/lib.rs # Crate 根的 lib.rs 结尾
/src/main.rs # Crate 根的 main.rs 结尾
```
