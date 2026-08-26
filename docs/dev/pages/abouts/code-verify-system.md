<h1 align="center">Markdown Code Verification System</h1>
<p align="center">
    A system that verifies every identified code block can be compiled
</p>

This system automatically extracts and compiles Rust code blocks from docs, ensuring all example code stays usable in CI.

## Config

Specify which Markdown files to verify via `dev/configs/verified-docs.toml`:

```toml
[verified]
readme = "./README.md"
getting_started = "./GETTING-STARTED.md"
documents_en_us = "./docs/pages/**"
documents_zh_cn = "./docs/_zh_CN/pages/**"
```
 
Each key is a label used to name the report items; values are single files, directories, or `**` globs.

Run all configured files:

```sh
cargo ci markdown-check-all
```
 
You can also test a single file via command-line arg (path is joined onto the current directory):

```sh
cargo ci markdown-check docs/pages/1-getting-started.md
```
 
## Default Rules

Every verified ` ```rust ` code block gets the following injected automatically at compile time — no need to write them explicitly in the block:

### 1. `#![allow(dead_code)]` and `#![allow(unused)]`

Added at the top of the generated `main.rs` to suppress dead-code warnings from partial code snippets.

### 2. `use mingling::prelude::*;`

If the block already has `use mingling::prelude::*;`, it won't be inserted again.

Otherwise it's inserted automatically (with `#[allow(unused_imports)]`).

### 3. `fn main() {}`

If the block **does not contain** a `fn main` definition, an empty `fn main() {}` is appended,

so the block can compile as a standalone binary project.

### 4. `mingling::macros::gen_program!();`

If the block **does not contain** a `gen_program!()` call,

`mingling::macros::gen_program!();` is appended automatically.

This call is required by the mingling framework.

### 5. Build Cache Dedup — Shared Dep Hash

Code blocks with the same `Features` and `Dependencies` are automatically grouped into the same compile group, sharing one `Cargo.toml` and build artifacts, avoiding redundant compilations.

> [!NOTE]
>
> Hash input (all sorted):
>
> 1. Feature list
> 2. External dep name list
> 3. External dep version list
> 4. `name=version` pairs
>
> Uses FNV-1a 64-bit hash, stable across runs.

## Verification Steps

After the **default rules** are applied, each block goes through:

### 1. Block Extraction

- Any fenced code block whose fence starts with ` ```rust ` is extracted
  (e.g. ` ```rust `, ` ```rust,diff `, ` ```rust,simulation `) — not just an
  exact ` ```rust `.
- Empty blocks (no code lines) are skipped.
- Blocks with `// NOT VERIFIED` alone are skipped.

### 1.1 Diff Blocks (` ```rust,diff `)

A ` ```rust,diff ` fence is a diff of rust code:

- Lines starting with `+` are added back to the compilation with their `+`
  prefix stripped (e.g. `+fn foo() {}` compiles as `fn foo() {}`).
- Lines starting with `-` are removed from the compilation entirely.
- Other lines (context/metadata) are compiled as-is.

This lets a diff-style snippet still be verified: the final state of the
changed file is what gets compiled.

### 2. Temp Project Generation

Each dedup-hash group gets its own Cargo project:

```
.temp/doc-test/<hash>/
├── Cargo.toml
└── src/
    └── main.rs
```
 
### 3. Build Verification

Compiled with `cargo check --manifest-path ... --color=always`, stderr inherited to the terminal for real-time progress. Blocks within a group are serial (they share the crate directory); groups run in parallel.

- **Build OK** → **PASS**
- **Build FAIL** → **FAIL**, last 20 lines of error captured.

### 4. Report

Each file's result is exported through the reporter: an `ok` entry when every block passed, otherwise an `err` entry carrying the failed blocks' details. All entries land in `.temp/reports/collect/` (e.g. `Markdown-Check-All.Linux.ok`, `Markdown-Check-All.Linux.<item>.err`).

`cargo ci report-collect` then assembles everything into `.temp/reports/result.md` (also published to the GitHub Actions job summary).

### 5. Exit Code

- Any block fails → non-zero exit code (blocks CI pipeline).
- All pass → zero exit code.

---

## Metadata Tag Rules

At the start of a ` ```rust ` block (before code content), use these comment headers to declare metadata. Headers are parsed in order; everything after them is treated as code:

### `// NOT VERIFIED`

Marks the block **not to be compiled**. Use for illustrative snippets that can't compile on their own.

```rust
// NOT VERIFIED
// This block is illustrative only, won't be compiled
fn placeholder() {}
```
 
### `// BUILD TIME`

Marks the block as a `build.rs` script instead of `src/main.rs`. The block code is wrapped in `fn main() { }` and written to `build.rs`. A stub `fn main() {}` is generated for `src/main.rs`. The declared features are mirrored into `[build-dependencies]` so `build.rs` sees the same feature set.

```rust
// BUILD TIME
// Dependencies:
// serde = "1"
fn main() {
    // build-time work, e.g. writing generated sources into OUT_DIR
}
```
 
### `// Features: [...]`

Declares the mingling crate features needed by this block, as a JSON string array. These features are written into `Cargo.toml`'s `[dependencies]`.

```rust
// Features: ["full", "serde"]
```
 
### `// Dependencies:`

Declares external crate deps needed by the block. After `// Dependencies:`, each dep goes on one line: `// crate_name = "version"`.

```rust
// Dependencies:
// serde = "1"
// clap = "4"
```
 
> [!TIP]
>
> **Special handling**:
>
> For deps named `serde` or `clap` with a plain string version,
>
> `features = ["derive"]` is auto-added.
>
> If the version uses a TOML inline table (e.g. `{ version = "1", features = ["derive"] }`),
>
> it's kept as-is.

---

## `@@@` Lines (Hidden Compilation)

Lines starting with `@@@` are **hidden from the rendered documentation** but still included in compilation.

This is useful when you want to show only the core logic while keeping the block fully compilable:

```rust
// This line is visible in docs
@@@// This line is hidden but still compiled
@@@fn setup() { /* hidden boilerplate */ }
```
 
### How it works

| Stage                 | Handling                                                                                         |
| --------------------- | ------------------------------------------------------------------------------------------------ |
| **docsify rendering** | `@@@` lines are stripped before markdown is rendered (via `beforeEach` plugin)                   |
| **CI verification**   | `@@@` prefix is stripped during block parsing, remaining content is treated as regular Rust code |

### Convention

Use `@@@` for:

- `fn main() {}` / `gen_program!()` when the block doesn't need to show them
- Common `use` imports that would distract from the example
- Type definitions (`pack!`, `#[derive]`) that are necessary for compilation but not the focus
- Helper functions that the reader doesn't need to see

> [!TIP]
> `@@@` is the replacement for `// NOT VERIFIED` — instead of marking a block as uncompilable,
> hide the boilerplate and keep everything compiling.

---

## Structure Overview

| Module                                    | Responsibility                                                                       |
| ----------------------------------------- | ------------------------------------------------------------------------------------ |
| `dev/ci/src/markdown/project.rs`          | Block parsing, Cargo.toml/main.rs generation, FNV-1a dep hash                        |
| `dev/ci/src/markdown/test.rs`             | Grouping by dep hash, parallel `cargo check` execution                               |
| `dev/ci/src/task/cmd_markdown_check.rs`   | `markdown-check` / `markdown-check-all` commands: read config, collect files, report |
| `dev/ci/src/markdown/compare.rs`          | Structural signature comparison (for `markdown-compare`)                             |
| `dev/ci/src/task/cmd_markdown_compare.rs` | `markdown-compare` / `markdown-compare-all` commands                                 |
| `dev/configs/verified-docs.toml`          | Specifies which doc files to verify                                                  |

### Structure Comparison

`markdown-compare` (two files or directories) and `markdown-compare-all` (all languages from `dev/configs/docs-lang.txt`, whose first line is the reference directory) check that every translated docs directory **mirrors the structure** of the reference docs exactly: one token per line classifying headings, fenced code blocks (with language tag), `@@@` lines, blank lines, blockquotes, lists and plain text. Translated text may differ; the structure may not.

## Full Example

````markdown
```rust
// Features: ["picker"]
// Dependencies:
// serde = "1"

// Example code ...
```
````
 
The above block compiles equivalently to:

```rust
#![allow(dead_code)]
#![allow(unused)]
 
#[allow(unused_imports)]
use mingling::prelude::*;
 
// Example code ...
 
fn main() {}
 
mingling::macros::gen_program!();
```
 
`Cargo.toml` will contain:

```toml
[dependencies]
mingling = { path = "../../mingling", features = ["picker"] }
serde = { version = "1", features = ["derive"] }
```
