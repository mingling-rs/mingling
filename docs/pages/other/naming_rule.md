<h1 align="center">Naming Conventions</h1>
<p align="center">
    Keep your <b>Mingling</b> project types and function names organized
</p>

## Preface

Good naming is the foundation of large CLI projects. Mingling's pipeline pattern introduces several new types (Entry, State, Result, Error, Resource). Without consistent conventions, projects quickly end up with generic names like `Data`, `Info`, or `Context` that make it impossible to tell different responsibilities apart.

The naming conventions below come from real-world usage in production projects. They are not enforced by the framework, but following them is highly recommended.

## Type Naming

### Resource

Resources are global instances shared with all chains and renderers through the resource injection system.

```
Res + Name
```
 
| Example         | Description               |
| --------------- | ------------------------- |
| `ResCurrentDir` | Current working directory |
| `ResExitCode`   | Process exit code         |
| `ResREPL`       | REPL state                |

### Setup

Setups are initialization steps executed at program startup, registered via `with_setup`.

```
Name + Setup
```
 
| Example                   | Description                                                   |
| ------------------------- | ------------------------------------------------------------- |
| `BasicSetup`              | Basic initialization (`--quiet`, `--help`, `--confirm`)       |
| `StructuralRendererSetup` | structural renderer initialization (`--json`, `--yaml`, etc.) |

### Dispatcher

Dispatchers are the entry points of commands. The command name uses `.` to separate hierarchy levels and matches the arguments typed by the user.

```
command name
```
 
| Command      |
| ------------ |
| `greet`      |
| `remote.add` |
| `remote.rm`  |

### Entry

Entries are the pipeline starting types created by dispatchers, wrapping `Vec<String>`.

```
Entry + Command Hierarchy
```
 
| Command      | Entry               |
| ------------ | ------------------- |
| `greet`      | `EntryGreet`        |
| `remote.add` | `EntryRemoteAdd`    |
| `remote.rm`  | `EntryRemoteRemove` |

### State

States are intermediate pipeline types between Entry and Result, representing data after partial processing.

```
State + Description
```
 
| Example                 | Description                   |
| ----------------------- | ----------------------------- |
| `StateOperationRemotes` | Operating on remote repo list |
| `StateCheckRepository`  | Checking repository status    |

Typical pipeline chain: `EntryRemoteAdd → StateOperationRemotes → StateCheckRepository`

### Result

Results are the final types sent to the Renderer via `to_render()` in the pipeline.

```
Result + Content
```
 
| Example              | Description       |
| -------------------- | ----------------- |
| `ResultGreetSomeone` | Greeting result   |
| `ResultFruitList`    | Fruit list result |

Result structs are expected to be consumed by the Renderer, and their internal structure should be designed for rendering aesthetics. Generally use `#[derive(Grouped)]` instead of `pack!()` wrapping for more flexible field control.

### Error

Errors are different from Results. Errors can be sent via `to_chain()` or `to_render()` to route execution to the error handling path.

```
Error + Description
```
 
| Example                   | Description          |
| ------------------------- | -------------------- |
| `ErrorRepositoryNotFound` | Repository not found |

`StateOperationRemotes → ErrorRepositoryNotFound`

## Function Naming

### Chain Functions

| Processing Type | Naming Pattern                            | Example                                                            |
| --------------- | ----------------------------------------- | ------------------------------------------------------------------ |
| Entry           | `handle_` + entry name (snake_case)       | `handle_remote_add(prev: EntryRemoteAdd)`                          |
| State           | `handle_state_` + state name (snake_case) | `handle_state_operation_remotes(prev: StateOperationRemotes)`      |
| Error           | `handle_error_` + error name (snake_case) | `handle_error_repository_not_found(prev: ErrorRepositoryNotFound)` |
| Result          | ❌ Do not write chains for Result         | —                                                                  |

### Renderer Functions

| Processing Type | Naming Pattern                      | Example                                                            |
| --------------- | ----------------------------------- | ------------------------------------------------------------------ |
| Entry           | `render_entry_` + entry name        | `render_entry_remote_add(prev: EntryRemoteAdd)`                    |
| State           | ❌ Do not write renderers for State | —                                                                  |
| Error           | `render_error_` + error name        | `render_error_repository_not_found(prev: ErrorRepositoryNotFound)` |
| Result          | `render_` + result name             | `render_greet_someone(prev: ResultGreetSomeone)`                   |

**Principle**: Do not write renderers for intermediate types that users won't see or that don't need independent rendering.

---

## Function Signature Parameter Naming

| Type                 | Recommended Parameter Name              |
| -------------------- | --------------------------------------- |
| Entry                | `args`                                  |
| State                | `prev` (or descriptive like `remotes`)  |
| Result               | `result` (or descriptive like `fruits`) |
| Error                | `err`                                   |
| Resource (immutable) | `cwd`, `db`, `config`, etc.             |
| Resource (mutable)   | `counter`, `cache`, `session`, etc.     |

```rust
@@@ pack!(EntryRemoteAdd = Vec<String>);
@@@ #[derive(Default, Clone)]
@@@ struct ResDatabase {  }
@@@ #[derive(Default, Clone)]
@@@ struct ResCurrentDir {  }
#[chain]
fn handle_remote_add(args: EntryRemoteAdd, cwd: &ResCurrentDir, db: &mut ResDatabase) {
    // args: entry data
    // cwd:  injected immutable resource
    // db:   injected mutable resource
}
```
 
---

## Complete Example

```rust
@@@use mingling::macros::buffer;
@@@ #[derive(Default, Clone)]
@@@ struct ResDatabase {  }
@@@ impl ResDatabase { fn has_remote(&self, remote: &String) -> bool { true } }
@@@ pack!(StateOperationRemotes = String);
@@@ pack!(ResultRemoteAdded = String);
@@@ pack!(ErrorRepositoryNotFound = String);
// Dispatcher
dispatcher!("remote.add", EntryRemoteAdd);
 
// Entry → State
#[chain]
fn handle_remote_add(args: EntryRemoteAdd) -> Next {
    StateOperationRemotes::default().to_chain()
}
 
// State → Error or Result
#[chain]
fn handle_state_operation_remotes(state: StateOperationRemotes, db: &ResDatabase) -> Next {
    if db.has_remote(&state.inner) {
        ErrorRepositoryNotFound::new(state.inner).to_render()
    } else {
        ResultRemoteAdded::new(state.inner).to_render()
    }
}
 
// Result rendering
 
#[renderer(buffer)]
fn render_remote_added(result: ResultRemoteAdded) {
    r_println!("Remote added: {}", result.inner);
}
 
// Error rendering
#[renderer(buffer)]
fn render_error_repository_not_found(err: ErrorRepositoryNotFound) {
    r_println!("Error: remote '{}' not found", err.inner);
}
```
 
<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
