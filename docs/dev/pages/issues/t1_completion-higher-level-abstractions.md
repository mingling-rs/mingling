<h1 align="center">[T1] Higher-Level Abstractions for the Completion System</h1>
<p align="center">
    Feature: smarter state descriptions in <code>ShellContext</code> and a <code>picker_comp</code> module
</p>

## Background

Mingling's completion system filled a number of behavioral gaps in 0.4 and fixed many edge cases. It's now time to introduce more powerful higher-level abstractions.

Currently, completion logic relies on manually identifying user behavior through fields like `previous_word`, which is fragile and requires every completion function to re-derive the user's intent.

## Plan

### 1. Utility functions on `ShellContext`

Add a set of utility functions to `ShellContext`, enabling a smarter description of user state, rather than simply relying on manually identifying user behavior through fields like `previous_word`.

### 2. `picker_comp` module

Additionally, when the `picker` feature (introduced in 0.3.0) is enabled together with the `comp` feature, a module named `picker_comp` will be activated to enable more completion behaviors — e.g. completing picker-style flags (`--key=value`, multi-flag forms, etc.) using knowledge of the picker parsing model.

## Tasks

- [ ] Design the `ShellContext` utility API (state descriptions / high-level queries over the current input state)
- [ ] Implement the utility functions and add tests
- [ ] Implement the `picker_comp` module, gated on `picker` + `comp`
- [ ] Add completion behaviors specific to picker argument formats
- [ ] Update docs and examples
- [ ] Verify existing 0.4 completion edge-case fixes are preserved

## 🕘 Progress

- [ ] In Progress
- [ ] Complete

<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
