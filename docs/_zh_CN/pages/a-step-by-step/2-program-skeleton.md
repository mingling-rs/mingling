<h1 align="center">编写程序骨架</h1>
<p align="center">
    通过 <code>gen_program!()</code> 生成程序的骨架
</p>

在上一页，我们使用 `cargo add` 成功在项目中配置了 mingling，而本页我们将编写实际的程序逻辑。

## 1. 准备基础代码

上一页中，我们为了快速验证 Mingling 是否配置成功，复制了一段代码到 `main.rs`，现在请将它删除，我们从头开始编写：

```rust
// main.rs
fn main () {
    println!("Hello, World");
}
```
 
## 2. 使用预导入模块

Mingling 提供了一个 `prelude` 模块，它包含在开发 Mingling 程序时常用的组件，
我们将它导入到项目中：

```rust,diff
+ use mingling::prelude::*; // 导入常用模块
 
fn main () {
    println!("Hello, World");
}
```
 
## 3. 生成程序结构

在程序 **尾部** 增加 `gen_program!();` 宏，该宏将在编译期间展开并创建整个程序的调度逻辑：

```rust,diff
use mingling::prelude::*; // 导入常用模块
 
fn main () {
    println!("Hello, World");
}
 
+ gen_program!(); // 生成程序，它必须放在程序结尾！
```
 
> [!Important]
> `gen_program!()` **必须** 放在当前 Crate 中 `main.rs` 或 `lib.rs` 的尾部，这是 Mingling 的约定。
>
> 详见 [`gen_program!()` 为什么必须放在尾部？](#gen_program-为什么必须放在尾部？)

## 4. 生成程序

`gen_program!()` 宏会完整地生成整个程序的调度逻辑，并将它们存储在一个 `ThisProgram` 的枚举当中。

```rust,diff
// 它生成了 ThisProgram 枚举
gen_program!();
 
// 该宏大致会展开为
- pub enum ThisProgram {
-     // 所有成员 ID 表
- }
-
- impl ProgramCollect for ThisProgram {
-     type Enum = Self;
-     // 所有调度逻辑的实现
- }
```
 
> [!Tip]
> 在整个 crate 中，你可以使用 `crate::ThisProgram` 来代表整个程序。

该枚举实现的 `ProgramCollect` 特征包含如下逻辑：

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

```rust,diff
fn main() {
-    println!("Hello, World");
+    let mut program = ThisProgram::new(); // 创建程序实例 (Program<ThisProgram>)
+    program.exec_and_exit(); // 运行程序，在执行完成后会自动渲染结果并以指定退出码结束进程
}
```
 
完整代码：

```rust
use mingling::prelude::*;
 
fn main() {
    let mut program = ThisProgram::new(); // 创建程序实例 (Program<ThisProgram>)
    program.exec_and_exit(); // 运行程序，在执行完成后会自动渲染结果并以指定退出码结束进程
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
