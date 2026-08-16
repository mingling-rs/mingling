<h1 align="center">声明一个分发器</h1>
<p align="center">
    使用 dispatcher! 宏声明命令，并注册
</p>

Mingling 的管线从 Dispatcher 开始。

它的工作很简单：**匹配用户输入的命令，把参数包装成一个 Entry 类型**。

## `dispatcher!` 宏

`dispatcher!` 宏会同时生成两个类型：

| 生成物      | 用途                                            |
| ----------- | ----------------------------------------------- |
| `CMDType`   | 分发器本身，需要注册到 Program                  |
| `EntryType` | 入口类型，包裹 `Vec<String>`，作为 Chain 的输入 |

写法是固定的三个部分：

```rust
dispatcher!("命令路径", 入口类型);
```
 
看一个具体的例子：

```rust
dispatcher!("greet", EntryGreet);
```
 
> [!NOTE]
> 命令名（`"greet"`）会自动转换为 kebab-case。即使你写 `"GreetUser"`，匹配时也会变成 `greet-user`。

## 多级命令

如果你的程序有层级结构——比如 `remote add`、`remote rm`——只需要在命令名里加点号分隔：

```rust
dispatcher!("remote.add", EntryRemoteAdd);
dispatcher!("remote.rm",  EntryRemoteRm);
```
 
用户在终端输入 `remote add` 时，Mingling 会依次匹配 `remote` 和 `add` 两个层级。

## 入口类型 `EntryGreet`

你可能会好奇 `EntryGreet` 里面到底有什么。它本质上就是一个包装了 `Vec<String>` 的结构体：

```rust
// 示意，dispatcher! 宏实际生成的代码
pub struct EntryGreet {
    pub inner: Vec<String>,
}
```
 
用户在命令行输入 `greet Alice Bob`，`EntryGreet.inner` 就是 `vec!["Alice", "Bob"]`。

> [!IMPORTANT]
> Entry 的 `inner` 只包含 **匹配后剩余的参数**。
>
> 以 `remote add origin` 为例，`remote` 和 `add` 用于匹配命令路径，只有 `origin` 会进入 `EntryRemoteAdd.inner`。

## 进阶：隐式声明

以上是标准写法。如果你启用了 `extras` 特性，还可以更简洁：

```rust
// Features: ["extras"]
// 省略 CMDType 和 EntryType，名字自动推导
   dispatcher!("greet");
// dispatcher!("greet", EntryGreet);
```
 
这种写法会自动生成 `CMDGreet` 和 `EntryGreet`，效果跟显式声明完全一样。

不过在教程阶段，我们继续用显式写法——更清晰，也不依赖额外特性。

详见[特性列表](pages/other/features)。

## 下一步

接下来我们写一个 Chain 来接收 Entry，处理真正的业务逻辑。

<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
