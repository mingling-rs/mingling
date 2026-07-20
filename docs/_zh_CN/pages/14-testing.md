<h1 align="center">测试你的程序</h1>
<p align="center">
    为 Chain 和 Renderer 编写单元测试
</p>

管线模型附带的一个好处就是 **可测试性**。

Chain 只是一个接收输入、返回输出的函数，Renderer 也只是接收输入、写入内容的函数 —— 没有全局状态的黑魔法，测试起来很直接。

## 测试 Renderer

Renderer 是最容易测试的——调用函数，断言返回结果：

```rust
@@@pack!(ResultName = String);
#[renderer]
fn render_greet(result: ResultName) -> RenderResult {
    let mut r = RenderResult::new();
    r_println!(r, "Hello, {}!", *result);
    r
}
 
#[test]
fn test_render_name() {
    let result = render_name(ResultName::new("Alice".to_string()));
    assert_eq!(result.to_string().as_str(), "Hello, Alice!\n");
}
```
 
注意到 Renderer 的返回值改成了 `-> String`——`#[renderer]` 会把 `RenderResult` 自动转换成你指定的返回类型（默认是 `()`）。返回 `String` 后你就可以直接断言输出内容了。

## 测试 Chain

测试 Chain 稍微复杂一点，因为它的返回值是 `Next`（实际是 `impl Into<ChainProcess<ThisProgram>>`）。需要用框架提供的断言宏：

```rust
@@@use mingling::{assert_member_id, assert_render_result, unpack_chain_process};
@@@dispatcher!("hello", CMDHello => EntryHello);
@@@pack!(ResultName = String);
@@@pack!(ErrorNoName = ());
@@@#[chain]
@@@fn handle_hello(args: EntryHello) -> Next {
@@@    let name = args.inner.first().cloned().unwrap_or_default();
@@@    if name.is_empty() {
@@@        ErrorNoName::default().to_render()
@@@    } else {
@@@        ResultName::new(name).to_render()
@@@    }
@@@}
#[test]
fn test_handle_hello_with_name() {
    let chain_process = handle_hello(EntryGreet::new(vec!["Alice".to_string()])).into();
    // 断言这是一个渲染结果（不是继续 chain）
    assert_render_result!(chain_process);
    // 断言 member_id 是 ResultName
    assert_member_id!(chain_process, ResultName);
    // 解包出内部值
    let result_name = unpack_chain_process!(chain_process, ResultName);
    assert_eq!(result_name.inner, "Alice");
}
```
 
三个测试宏的作用：

| 宏                      | 功能                                          |
| ----------------------- | --------------------------------------------- |
| `assert_render_result!` | 断言 Chain 返回的是渲染路径（而非继续 chain） |
| `assert_member_id!`     | 断言返回值的成员 ID 是某个类型                |
| `unpack_chain_process!` | 从 ChainProcess 中解包出原始类型              |

## 用 entry! 宏构造数据

如果启用了 `extra_macros`，可以用 `entry!` 快速构造 Entry：

```rust
// Features: ["extra_macros"]
 
@@@use mingling::{assert_member_id, unpack_chain_process};
@@@use mingling::macros::entry;
@@@dispatcher!("hello", CMDHello => EntryHello);
@@@pack!(ResultName = String);
@@@#[chain]
@@@fn handle_hello(args: EntryHello) -> Next {
@@@    let name = args.inner.first().cloned().unwrap_or_default();
@@@    ResultName::new(name).to_render()
@@@}
#[test]
fn test_with_entry_macro() {
    // entry! 从字符串字面量构造 Entry
    let entry = entry!("--name", "Alice");
    let chain_process = handle_hello(entry).into();
    let result_name = unpack_chain_process!(chain_process, ResultName);
    assert_eq!(result_name.inner, "Alice");
}
```
 
## 测试资源注入

如果 Chain 使用了资源，测试时需要提供资源实例：

```rust
@@@use mingling::{assert_render_result, unpack_chain_process};
@@@#[derive(Default, Clone)]
@@@struct ResPrefix(String);
@@@dispatcher!("hello", CMDHello => EntryHello);
@@@pack!(ResultGreeting = String);
@@@
#[chain]
fn handle_hello(args: EntryHello, prefix: &ResPrefix) -> Next {
    let name = args.inner.first().cloned().unwrap_or_default();
    ResultGreeting::new(format!("{}, {}", prefix.0, name)).to_render()
}
 
#[test]
fn test_handle_with_resource() {
    // 资源需要在测试中手动传入
    let result = handle_hello(
        EntryHello::new(vec!["World".to_string()]),
        &ResPrefix("Hello".to_string()),
    );
    let greeting = unpack_chain_process!(result, ResultGreeting, ThisProgram);
    assert_eq!(greeting.inner, "Hello, World");
}
```
 
管线模型让测试变得简单：每个 Chain 和 Renderer 都是相对独立的函数，构造输入、断言输出即可。

<p align="center" style="font-size: 0.85em; color: clear;">
    Written by @Weicao-CatilGrass
</p>
