<h1 align="center">The Next-Gen Mingling Pipeline</h1>

## Preface

The current Mingling pipeline model is extremely simple:

```plaintext
Function -> Type -> Function
```
 
As can be seen, it cannot handle overly complex situations. Starting from 0.4.0, Mingling plans to build a replaceable Pipeline backend that supports parallel task processing, truly becoming a "CLI workflow orchestration framework".

The new model is as follows:

```plaintext
Entry -> Function group satisfying all constraints -> Type combinator -> Function group satisfying all constraints -> Type combinator
```
 
This model will bring comprehensive **breaking changes** to the entire framework, but the benefits are long-term: you can easily gain parallel (or sequential, depending on the definition of the Runtime Adapter) scheduling capabilities by defining functions.

## Vision

The original Mingling only supported the following syntax:

```rust
// First EntryFoo handler
#[chain]
fn handle_foo1(_: EntryFoo) -> StateFoo1 {}
 
// Second EntryFoo handler (duplicate registration! Won't compile)
#[chain]
fn handle_foo2(_: EntryFoo) -> StateFoo2 {}
```
 
Under the new pipeline model, Mingling should theoretically support the following syntax:

```rust
// First EntryFoo handler
#[chain]
fn handle_foo1(_: EntryFoo) -> StateFoo1 {}
 
// Second EntryFoo handler
#[chain]
fn handle_foo2(_: EntryFoo) -> StateFoo2 {}
 
// Will be dispatched by handle_foo1's return value
#[chain]
fn handle_state_foo1(_: StateFoo1) {}
 
// Will be dispatched by handle_foo2's return value
#[chain]
fn handle_state_foo2(_: StateFoo2) {}
 
// If two functions are dispatched at the same stage (e.g., handle_foo1 and handle_foo2 produced by EntryFoo),
// and they respectively return StateFoo1 and StateFoo2, then dispatch this function
#[chain]
fn handle_state_foo(_: StateFoo1, _: StateFoo2) {}
 
// Same as above, but if the Other condition is not satisfied at a certain stage, it won't execute
#[chain]
fn handle_state_foo_not_exec(_: StateFoo1, _: StateFoo2, _ot: Other) {}
```
 
## Current Issues

1. How to design reasonable and general scheduling logic?

2. If multiple chains all contain a `to_render` renderer dispatch, how to control its behavior?

3. How should the existing ProgramHook be refactored?

4. Is it necessary to remove the Chain, Renderer, Completion, and HelpRequest traits, replacing them with a function registry?

5. How to redesign ProgramCollect?

<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
