<h1 align="center">[T2] Automated dispatcher_tree Optimization Decisions</h1>
<p align="center">
    Feature: let Mingling decide when <code>dispatch_tree</code> pays off (under consideration)
</p>

> [!NOTE]
>
> This item is **under consideration**. It depends on [Remove with_dispatcher and with_dispatchers](t0_remove-with-dispatcher).

## Background

`dispatch_tree` provides a faster dispatch path, but it is not always a win. Currently users must manually enable the `dispatch_tree` feature and make the trade-off themselves.

After dispatcher registration becomes compile-time collected (see [Remove with_dispatcher and with_dispatchers](t0_remove-with-dispatcher)), Mingling can know the full set and depth of registered commands at compile time — making automated decisions implementable.

## Plan

Mingling can automatically decide whether to use `dispatcher_tree` to optimize dispatch efficiency based on the current number and depth of registered commands, so users no longer need to manually enable the `dispatch_tree` feature.

### Conditions

`dispatch_tree` has an advantage in cases where command depth is too high and the number of commands is too large. However, if the number of commands is too small, the increased CPU prediction failure rate will inevitably make it less efficient than linear lookup; specifics need to be tuned during implementation.

### Resolve the `pathf` + `dispatch_tree` build-dependency issue

Additionally, the issue where `pathf` + `dispatch_tree` must be explicitly specified in `[build-dependencies]` will be resolved:

```toml
# Before
[build-dependencies.mingling]
version = "0.4.0"
features = [ "build", "pathf", "dispatch_tree" ] # `dispatch_tree` must be explicitly specified for `pathf` to recognize it
 
# After
[build-dependencies.mingling]
version = "0.4.0"
features = [ "build", "pathf" ] # No `dispatch_tree` feature; `pathf` no longer needs to consider its branches
```
 
## Tasks

- [ ] Collect statistics about registered commands (count, depth) at compile time
- [ ] Benchmark / tune the threshold between linear lookup and `dispatch_tree`
- [ ] Implement the automatic decision and wire it into dispatch code generation
- [ ] Remove the manual `dispatch_tree` feature toggle (or keep it as an override?)
- [ ] Refactor `pathf` so it no longer branches on `dispatch_tree`
- [ ] Update examples, tests, and docs

## 🕘 Progress

- [ ] Under Consideration
- [ ] In Progress
- [ ] Complete

<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
