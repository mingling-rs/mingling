<h1 align="center">命名规范</h1>
<p align="center">
    让你的 <b>Mingling</b> 项目类型和函数名井井有条
</p>

## 前言

好的命名是大型 CLI 项目的基石。Mingling 的管线模式引入了几类新的类型（Entry、State、Result、Error、Resource），如果不加约束，项目里很快就会出现 `Data`、`Info`、`Context` 这类无法区分职责的名字。

下面这套命名规范来自实践，已在生产项目中使用。它不是框架的强制规则，但建议跟随。

## 类型命名

### 资源

资源是通过资源注入系统共享给所有 chain 和 renderer 的全局实例。

```
Res + 名称
```
 
| 示例            | 说明         |
| --------------- | ------------ |
| `ResCurrentDir` | 当前工作目录 |
| `ResExitCode`   | 进程退出码   |
| `ResREPL`       | REPL 状态    |

### 构建

构建是在程序启动时执行的初始化步骤，由 `with_setup` 注册。

```
名称 + Setup
```
 
| 示例                      | 说明                                           |
| ------------------------- | ---------------------------------------------- |
| `BasicSetup`              | 基础初始化（`--quiet`、`--help`、`--confirm`） |
| `StructuralRendererSetup` | 通用渲染器初始化（`--json`、`--yaml` 等）      |

### 分发器

分发器是命令的入口点。命令名用 `.` 分隔层级，与用户输入的命令参数一一对应。

```
命令名
```
 
| 命令         |
| ------------ |
| `greet`      |
| `remote.add` |
| `remote.rm`  |

### 入口

入口是分发器创建的管线起始类型，包裹 `Vec<String>`。

```
Entry + 命令层级
```
 
| 命令         | 入口                |
| ------------ | ------------------- |
| `greet`      | `EntryGreet`        |
| `remote.add` | `EntryRemoteAdd`    |
| `remote.rm`  | `EntryRemoteRemove` |

### 状态

状态是介于入口和结果之间的管线中间类型，代表经过部分处理后的数据。

```
State + 描述
```
 
| 示例                    | 说明                 |
| ----------------------- | -------------------- |
| `StateOperationRemotes` | 正在操作远程仓库列表 |
| `StateCheckRepository`  | 正在检查仓库状态     |

典型的管线链：`EntryRemoteAdd → StateOperationRemotes → StateCheckRepository`

### 结果

结果是管线中由 `to_render()` 发往 Renderer 的最终类型。

```
Result + 内容
```
 
| 示例                 | 说明         |
| -------------------- | ------------ |
| `ResultGreetSomeone` | 问候结果     |
| `ResultFruitList`    | 水果列表结果 |

结果结构体期望被 Renderer 消费，内部结构应该为了渲染美观而设计。一般用 `#[derive(Grouped)]` 标注结构体，以获得更灵活的字段控制。

### 错误

错误与结果不同。错误可以被 `to_chain()` 或 `to_render()` 发送，用于将执行路由到错误处理路径。

```
Error + 描述
```
 
| 示例                      | 说明       |
| ------------------------- | ---------- |
| `ErrorRepositoryNotFound` | 仓库未找到 |

`StateOperationRemotes → ErrorRepositoryNotFound`

## 函数命名

### Chain 函数

| 处理类型 | 命名模式                                 | 示例                                                               |
| -------- | ---------------------------------------- | ------------------------------------------------------------------ |
| Entry    | `handle_` + entry 名（snake_case）       | `handle_remote_add(prev: EntryRemoteAdd)`                          |
| State    | `handle_state_` + state 名（snake_case） | `handle_state_operation_remotes(prev: StateOperationRemotes)`      |
| Error    | `handle_error_` + error 名（snake_case） | `handle_error_repository_not_found(prev: ErrorRepositoryNotFound)` |
| Result   | ❌ 不要为 Result 写 chain                | —                                                                  |

### Renderer 函数

| 处理类型 | 命名模式                    | 示例                                                               |
| -------- | --------------------------- | ------------------------------------------------------------------ |
| Entry    | `render_entry_` + entry 名  | `render_entry_remote_add(prev: EntryRemoteAdd)`                    |
| State    | ❌ 不要为 State 写 renderer | —                                                                  |
| Error    | `render_error_` + error 名  | `render_error_repository_not_found(prev: ErrorRepositoryNotFound)` |
| Result   | `render_` + result 名       | `render_greet_someone(prev: ResultGreetSomeone)`                   |

**原则**：不要为不会被用户看到或不需要独立渲染的中间类型写 renderer。

---

## 函数签名参数命名

| 类型           | 推荐参数名                       |
| -------------- | -------------------------------- |
| Entry          | `args`                           |
| State          | `prev`（或具名如 `remotes`）     |
| Result         | `result`（或具名如 `fruits`）    |
| Error          | `err`                            |
| 资源（不可变） | `cwd`、`db`、`config` 等         |
| 资源（可变）   | `counter`、`cache`、`session` 等 |

```rust
@@@ #[derive(Grouped, Wrap)]
@@@ pub struct EntryRemoteAdd(Vec<String>);
@@@ #[derive(Default, Clone)]
@@@ struct ResDatabase {  }
@@@ #[derive(Default, Clone)]
@@@ struct ResCurrentDir {  }
#[chain]
fn handle_remote_add(args: EntryRemoteAdd, cwd: &ResCurrentDir, db: &mut ResDatabase) {
    // args: 入口数据
    // cwd:  注入的不可变资源
    // db:   注入的可变资源
}
```
 
---

## 完整示例

```rust
@@@use mingling::macros::buffer;
@@@ #[derive(Default, Clone)]
@@@ struct ResDatabase {  }
@@@ impl ResDatabase { fn has_remote(&self, remote: &String) -> bool { true } }
@@@ #[derive(Grouped, Wrap, Default)]
@@@ pub struct StateOperationRemotes(String);
@@@ #[derive(Grouped, Wrap)]
@@@ pub struct ResultRemoteAdded(String);
@@@ #[derive(Grouped, Wrap)]
@@@ pub struct ErrorRepositoryNotFound(String);
// 分发器
dispatcher!("remote.add", EntryRemoteAdd);
 
// 入口 → 状态
#[chain]
fn handle_remote_add(args: EntryRemoteAdd) -> Next {
    StateOperationRemotes::default().to_chain()
}
 
// 状态 → 错误或结果
#[chain]
fn handle_state_operation_remotes(state: StateOperationRemotes, db: &ResDatabase) -> Next {
    if db.has_remote(&state.0) {
        ErrorRepositoryNotFound(state.0).to_render()
    } else {
        ResultRemoteAdded(state.0).to_render()
    }
}
 
// 结果渲染
 
#[renderer(buffer)]
fn render_remote_added(result: ResultRemoteAdded) {
    r_println!("Remote added: {}", result.0);
}
 
// 错误渲染
#[renderer(buffer)]
fn render_error_repository_not_found(err: ErrorRepositoryNotFound) {
    r_println!("Error: remote '{}' not found", err.0);
}
```
 
<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
