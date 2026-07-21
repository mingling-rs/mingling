# 关于 Mingling Linter 设计

Mingling Linter 是独立于 `clippy` 和 `check` 之外的检查器，用于检查 Mingling 项目的代码质量

## 关于 Linter 定义

已完成！所有的定义都被输出到 registry.json 中，可以读取

## 忽略规则

Linter 在检查时，会检查开头的 mlint 标识，用局部读写直到没有#![.*]这样的结构为止，判断当前文件需要参与的 Linter

默认来讲，一个文件所有默认为 deny、warn 的 linter 都会参与，直到被显式 allow 覆盖。

```rust
#![mlint(allow(linter))]
#![mlint(warn(linter))]
#![mlint(deny(linter))]
```

## 检查阶段

1. 为每个包的内部 rs 文件（必须被包含）检查，并按照忽略规则，计算每个文件需要哪些 linter 检查（仅 file 级别的）
2. 使用 tokio 并行检查
3. 同时，在检查过程中，计算更细粒度的哪些 Linter 需要在哪些行（或 AST 层级，待定）检查，检查的程度是什么 （warn还是deny）
4. 使用 tokio 并行检查 所有 细粒度的 Linter
5. 输出检查结果

## 性能

请使用 tokio 并行化执行 linter，但是结果放在缓冲区，计算完成后按照文件层级（主：文件深度，次：名字 a-Z，辅：名字 0-9）排序，后输出

## 关于架构层面

第一版可以直接使用 tokio 并行化，这不会很难
