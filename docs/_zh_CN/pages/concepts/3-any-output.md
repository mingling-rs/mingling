<h1 align="center">任意输出机制</h1>
<p align="center">
    关于 AnyOutput 和 ChainProcess 的运作模式
</p>

Dispatcher → Chain → Renderer 三阶段之间传递的数据是什么？

Chain 的输出可能是一个成功结果、一个错误、或者还需要继续交给下一个 Chain——这些类型各不相同，管线如何在编译期不知道具体类型的情况下，把它们送到正确的地方？

## AnyOutput：类型擦除 + 组标签

Mingling 的解法是把**所有类型擦除到同一个包装里**，然后用一个**枚举标签**来区分它们：

```
AnyOutput<G>
├── inner: Box<dyn Any + Send>    ← 真正的数据，类型已被擦除
├── type_id: TypeId               ← 运行时类型 ID，用于安全 downcast
└── member_id: G                  ← 枚举标签，标记"这是谁"
```
 
这里的 `G` 就是 `gen_program!()` 生成的程序枚举（也就是你熟知的 `ThisProgram`）。

每个被 `pack!` 或 `#[derive(Groupped)]` 标记的类型都被分配到这个枚举的一个变体。

## ChainProcess：数据 + 路由

在 `AnyOutput` 的基础上，`ChainProcess<G>` 加了一个**路由信息**：

```
ChainProcess<G>
├── Ok(AnyOutput<G>, NextProcess)    ← 携带数据，告诉调度器下一步去哪
│   ├── NextProcess::Chain           ← "还没完，继续交给下一个 Chain"
│   └── NextProcess::Renderer        ← "出结果了，展示给用户"
└── Err(ChainProcessError)           ← "出错了，终止程序"
```
 
这就是为什么 Chain 函数返回的不是裸数据，而是 `ChainProcess`——它把 **"下一步去哪"** 和 **"数据"** 打包在一起。

调度器根据 `NextProcess` 决定是继续循环还是退出渲染。

## Groupped：谁是谁

调度器如何知道 `AnyOutput` 里装的是 `ResultName` 还是 `ErrorUserBlocked`？答案是 `Groupped` trait：

```
trait Groupped<G> {
    fn member_id() -> G;
}
```
 
当你用 `pack!(ResultName = String)` 时，宏自动为 `ResultName` 实现 `Groupped`，`member_id()` 返回枚举中对应的变体。调度器一看 `member_id`，就去找对应的 Chain 或 Renderer。

`to_chain()` 和 `to_render()` 本质上是 `AnyOutput` 的快捷方法，分别构造 `ChainProcess::Ok(any, Chain)` 和 `ChainProcess::Ok(any, Renderer)`。

## 调度的执行

在运行时，主循环的工作就是：

1. 看当前 `AnyOutput` 的 `member_id`
2. 查这个变体有没有对应的 Chain → 有就执行，拿到新的 `AnyOutput` 和 `NextProcess`
3. 如果 `NextProcess` 是 `Chain` → 回到第 1 步
4. 如果 `NextProcess` 是 `Renderer` → 退出循环，渲染

这套机制保证了**类型安全**：`gen_program!()` 生成的调度代码在做 `restore`（从 `Box<dyn Any>` 还原为具体类型）时，一定是在匹配的 `member_id` 分支内做的，不可能把 `ResultName` 的数据当作 `ErrorUserBlocked` 来解包。

> [!TIP]
> 日常开发中你不需要手动操作 `AnyOutput` 或 `ChainProcess`。
>
> `pack!`、`#[chain]`、`#[renderer]` 这些宏帮你处理了所有的包装和解包。

<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
