# Roadmap

## Milestone.1 "MVP" 🎉

This milestone completes the minimum viable version of Mingling. It does not introduce [semver](https://semver.org/) semantics; instead, it reaches a usable state through rapid Breaking Patches.

- [x] [[0.1.4](https://docs.rs/mingling/0.1.4/mingling/)] [`core`] [`structural_renderer`] **Mingling** can render data into serializable formats via `--json` and `--yaml` flags
- [x] [[0.1.5](https://docs.rs/mingling/0.1.5/mingling/)] [`core`] [`comp`] **Mingling** can dynamically invoke itself to provide completions for shells like `bash`, `zsh`, `fish`, and `pwsh`
- [x] [[0.1.6](https://docs.rs/mingling/0.1.6/mingling/)] [`core`] [`comp`] **Mingling** can gather more context for smarter completions
- [x] [[0.1.7](https://docs.rs/mingling/0.1.7/mingling/)] [`clap`] Provides a **Clap** compatibility layer, allowing **Mingling** to reuse its powerful parsing capabilities
- [x] [[0.1.7](https://docs.rs/mingling/0.1.7/mingling/)] [`core`] **Mingling** can intercept `-h` or `--help` flags to display custom help text for each subcommand
- [x] [[0.1.7](https://docs.rs/mingling/0.1.7/mingling/)] [`mling`] Provides a basic scaffolding tool (`mling`) for rapid development and debugging
- [x] [[0.1.8](https://docs.rs/mingling/0.1.8/mingling/)] [`core`] [`dispatch_tree`] Converts the subcommand list into a prefix tree to improve command matching speed
- [x] [[0.1.9](https://docs.rs/mingling/0.1.9/mingling/)] [`core`] [`dev_toolkits`] Provides debugging interfaces for developers to capture invocation information when issues arise (`InvokeStackDisplay`) (indirectly implemented via `ProgramHook`)
- [x] [[0.1.9](https://docs.rs/mingling/0.1.9/mingling/)] [`core`] [`repl`] Provides REPL capability (`program.exec_repl();`)
- [x] [[0.2.0](https://docs.rs/mingling/0.2.0/mingling/)] Complete documentation, tests, and examples

## Milestone.2 "More Comfortable Dev and User Experience"

Starting from this milestone, Mingling will fully adhere to [semver](https://semver.org/) semantics, polishing the API and surrounding toolchain to ensure a comfortable and convenient command-line development experience.

- [ ] [`mling` / `mingling-cli`]
    - [x] **Mingling** Linter
    - [x] **Mingling** Project Generator
    - [x] **Mingling** Program Installer & Manager (For development)
    - [ ] Helpdoc Editor
- [x] [`picker`] A more efficient and intelligent argument parser
- [x] [`macros`] ~~Remove r_print! / r_println! macros~~ (see below)
- [x] [`macros`] Make implicit modifications to functions explicit

## Milestone.3 "Unplanned"

- [ ] ...
