<h1 align="center">补全</h1>
<p align="center">
    使用 comp 特性实现完全动态的补全系统
</p>

Mingling 的补全是**完全动态**的——没有静态的补全文件，而是在运行时根据用户当前输入实时计算补全建议。

## 开启 comp

```toml
# Cargo.toml
[dependencies.mingling]
features = ["comp"]
 
[build-dependencies.mingling]
features = [
    "comp",
    # 启用 `build` 特性以提供构建期支持
    "build"
]
```
 
## 工作原理

当用户按下 `TAB` 时，补全脚本会调用程序的隐藏子命令 `__comp`，它会根据输入的 `ShellContext` 动态地查询最合适的建议。

这个隐藏子命令由 `gen_program!()` 在启用 `comp` 特性时自动生成，对应的分发器是 `CMDCompletion`，你需要使用 `with_dispatcher` 添加到程序中。

补全流程：

1. 二次匹配用户当前输入的 `Dispatcher`
2. 调用对应的 `#[completion]` 函数
3. 函数返回 `Suggest`（文件补全或建议列表）
4. 通知 Shell 将建议呈现出来

## 定义补全

用 `#[completion(EntryType)]` 为 Entry 定义补全逻辑：

```rust
// Features: ["comp"]
@@@use mingling::prelude::*;
@@@use mingling::{ShellContext, Suggest, SuggestItem};
@@@use std::collections::BTreeSet;
@@@dispatcher!("greet", CMDGreet => EntryGreet);
 
#[completion(EntryGreet)]
fn complete_greet(ctx: &ShellContext) -> Suggest {
    if ctx.previous_word == "greet" {
        let mut items = BTreeSet::new();
        items.insert(SuggestItem::new_with_desc("Alice".into(), "Likes to receive messages".into()));
        items.insert(SuggestItem::new("World".into()));
        Suggest::Suggest(items)
    } else {
        Suggest::FileCompletion
    }
}
```
 
`suggest!` 宏是更简洁的写法，效果相同：

```rust
// Features: ["comp"]
@@@use mingling::macros::suggest;
@@@fn example() {
suggest! {
    "Alice": "Likes to receive messages",
    "World"
};
@@@}
```
 
`ShellContext` 包含用户当前的输入状态（`previous_word`、`current_word`、`all_words` 等），`Suggest` 有两种变体：`Suggest::Suggest(list)` 返回建议列表，`Suggest::FileCompletion` 交给 shell 自己做文件补全。

## 生成补全脚本

在 `build.rs` 中调用 `build_comp_scripts` 生成补全脚本（需要 `builds` + `comp` 特性）。

详见 [example-completion](https://mingling-rs.github.io/mingling/docs/example-viewer.html?name=example-completion)。

<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
