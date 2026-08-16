<h1 align="center">Resource System</h1>
<p align="center">
    How Mingling Manages Global State
</p>

CLI programs often need to share global things—config files, database connections, counters, the current working directory.

In vanilla Rust you might reach for `OnceCell` or `lazy_static`. In Mingling there's a unified mechanism: the **resource system**.

## What is a Resource?

A resource is data shared across multiple Chains and Renderers.

You just define a type, register it with the Program, and declare it in your function signature—the framework handles injection and lifecycle management for you.

## Core Mechanism: ResourceMarker

Any type that implements both `Default + Clone` can automatically become a resource. The framework implements the `ResourceMarker` trait for it, giving it:

- **`res_clone()`** — when multiple Chains access it concurrently, the framework can clone to avoid lock contention
- **`res_default()`** — provides a fallback value when the resource hasn't been registered

If you need finer lifecycle control, you can use `LazyRes<T>`. It lets the resource be initialized on first access and can run a callback on drop (e.g., saving state to disk before exit).

## Why Not Global Variables?

The traditional approach with statics creates implicit dependencies—you can't tell from the function signature what global state it uses. Mingling's resource injection makes **dependencies explicit**:

- Whatever resources a function needs go in its parameter list
- `&T` means read-only access, `&mut T` means mutable
- Callers can see the function's side effects at a glance

For example:

```rust
@@@ use mingling::res::ResExitCode;
@@@ #[derive(Grouped, Wrap)]
@@@ pub struct ErrorFileNotFound(());
#[chain]
fn handle_error_file_not_found(
    error: ErrorFileNotFound,
    ec: &mut ResExitCode // the signature reveals the side effect!
) {
    ec.exit_code = 2; // modifying the exit code here
}
```
 
## Resources and Setup

Resources are typically registered with the Program in two ways:

1. **Direct registration** — calling `program.with_resource(...)` in `main`
2. **Via Setup** — using built-in Setups like `DirectoryEnvironmentSetup` to batch-register resources (e.g., `ResCurrentDir`, `ResHomeDir`)

A Setup is a higher-level abstraction than a resource—one Setup can register multiple resources and do other initialization work.

See the [Program Assembly](./pages/8-setup-and-resources) chapter in the tutorial for more details.

<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
