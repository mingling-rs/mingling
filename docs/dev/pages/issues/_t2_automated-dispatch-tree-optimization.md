<h1 align="center">[Solved] [T2] Automated dispatcher_tree Optimization Decisions</h1>
<p align="center">
    Feature: let Mingling decide when <code>dispatch_tree</code> pays off (implemented)
</p>

> [!NOTE]
>
> This item is **implemented**. It depends on [Remove with_dispatcher and with_dispatchers](t0_remove-with-dispatcher).

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
 
## Final Implementation

The automated dispatch-strategy selection is now in place. A new `dispatch_auto` module (the default when no dispatch feature is enabled) picks at macro-expansion time from three strategies — **linear list**, **char trie**, and **perfect hash** — based on a cost model calibrated against the `dev/bench/dispatch` benchmark matrix.

### Two new dispatch features

In addition to the existing `dispatch_tree`, two new mutually-exclusive features now exist:

- **`dispatch_linear`** — force linear longest-prefix list (the former default).
- **`dispatch_phf`** — force a CHD minimal perfect hash (constant-time lookup, O(1) code size).
- **`dispatch_tree`** — force the char-level trie.
- **(none)** — **auto mode**: pick the best strategy from the command table.

Enabling more than one triggers a `compile_error!`.

### Auto-selection heuristic

`dispatch_auto::select_strategy` inspects the normalized command table (names, depth, nesting) and picks:

- **deep nested chains at modest sizes** (`max_words ≥ 8`, `n ≤ 128`) → linear list (short memcmps beat the trie's per-level char walk plus fallback calls);
- **single-word tables with long names** (avg_len ≥ 16–24) → perfect hash (one hash beats the char walk);
- **small tables** (`n ≤ 64`) → linear vs trie by an internal cost model;
- **everything else** → char trie (O(depth) hits, linear code size after the fallback-chain refactor).

The heuristic is empirical and may drift as the benchmark matrix grows.

### Benchmark harness

A workspace-internal harness `dev/bench/dispatch` (`cargo dispatch-bench`) measures all four strategies across a `len×count×type` matrix (4/8/16/32 × 128/256 × single/multi/nested4/nested10), reporting per-cell ns/op for hits and misses, geometric means, and how often auto matches the per-cell best / stays within 5%.

### Trie code-size fix

The trie generator was rewritten so the longest-prefix fallback is a single shared `__trie_fallback` method (called, not inlined, per arm) rather than inlined into every arm. This keeps generated code linear in the table size — a 1024×16 nested table previously emitted ~13 MB of tokens.

<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
