<h1 align="center">关于 ProgramCollect</h1>
<p align="center">
    了解 gen_program!() 是如何生成程序的
</p>

每个 Mingling 程序最后都有一行 `gen_program!()`。它在背后做了三件事，把整个程序的骨架搭建出来。

## gen_program!() 的三件事

### 1. 生成枚举

扫描当前模块中所有 `#[derive(Grouped)]`、`#[chain]`、`#[renderer]` 等宏标记的类型，为每个类型生成一个枚举变体。

这个枚举就是 `AnyOutput<G>` 中 `G` 的类型 —— 调度器靠枚举变体来区分管线中传递的不同数据。

### 2. 生成 ProgramCollect 实现

`ProgramCollect` 是一个 trait，定义了 **"每个枚举变体对应什么类型、由谁处理"** 的映射关系：

- **`do_chain`** —— 根据 `member_id` 调用对应的 `#[chain]` 函数，返回新的 `AnyOutput` 和 `NextProcess`
- **`render`** —— 根据 `member_id` 调用对应的 `#[renderer]` 函数，写入 `RenderResult`
- **`render_help`** —— 根据 `member_id` 调用对应的 `#[help]` 函数
- **`has_chain` / `has_renderer`** —— 判断某个变体有没有对应的处理函数
- **`build_entry_fallback` / `build_renderer_not_found` / `build_empty_result`** —— 三个内置降级类型，处理边界情况

这套映射在运行时通过枚举匹配来完成——编译期只生成了枚举和匹配分支，实际的函数调用发生在运行时。

### 3. 生成 ThisProgram

生成 `ThisProgram` 类型别名，指向 `Program<生成的枚举>`。这就是为什么在 `main` 中可以直接写 `ThisProgram::new()`——它就是你整个程序的完整类型。

---

## 关于 `pathf` 和 `dispatch_tree` 下的差异

以上是默认行为，但在启用特定 feature 时会有变化：

### 1. `dispatch_tree` 特性

Dispatcher 的匹配不再使用 `Vec<Box<dyn Dispatcher>>` 做线性匹配，而是在编译期将子命令结构构建为前缀树（Trie）。

匹配复杂度从 `O(n)` 降到 `O(k)` —— `k` 是输入长度，与命令数量无关。

### 2. `pathf` 特性（Module Pathfinder）

默认情况下所有宏标记的类型必须在同一模块才能被 `gen_program!()` 收集到

启用 `pathf` 后，编译期自动扫描所有子模块，找到所有用宏标记的类型并生成完整的模块路径引用 —— 类型定义在深层子模块也无需手动 `use`。

<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
