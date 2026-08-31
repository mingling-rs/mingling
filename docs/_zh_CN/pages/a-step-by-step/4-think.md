<h1 align="center">回顾和思考？</h1>
<p align="center">
    回顾之前的代码，思考还有哪里可以做的更好。
</p>

完整看看上一页的代码：

```rust
use mingling::prelude::*;
 
fn main() {
    let mut program = ThisProgram::new();
    program.exec_and_exit();
}
 
#[derive(Grouped)]
pub struct ResultGreet {
    name: String
}
 
#[command]
pub fn greet(args: Vec<String>) -> ResultGreet {
    let name = args
        .first()
        .map(|s| s.as_str())
        .unwrap_or("Mingling")
        .to_string();
    ResultGreet { name }
}
 
#[renderer]
pub fn render_greet(r: ResultGreet) -> String {
    format!("Hello, {}\n", r.name).into()
}
 
gen_program!();
```
 
**相信你一定发现了几个开发或者交互上的问题：**

1. 解析方式是否过于原始了？

```rust
#[command]
pub fn greet(args: Vec<String>) -> ResultGreet {
    let name = args // 直接操作 Vec<String>
        .first() // 直接操作 Vec<String>
        .map(|s| s.as_str()) // 直接操作 Vec<String>
        .unwrap_or("Mingling") // 直接操作 Vec<String>
        .to_string(); // 直接操作 Vec<String>
    ResultGreet { name }
}
```
 
2. 对于错误的命令输入，无任何反应？

```bash,simulation
~# my-cli greet
Hello, World!
~# my-cli hello
~#
```
 
3. 当用户需要 `--help`，如何绘制？

```bash,simulation
~# my-cli --help
~# my-cli greet --help
~# 
```
