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
 
相信你一定发现了几个用户交互上的问题：

1. 当用户没输入、输错命令的时候，是否可以增加一个提示告知用户？
2. 参数解析的方式是否 **过于原始** 了？
3. 我们是否可以给 `greet` 命令添加一个 `--help` 符号？
