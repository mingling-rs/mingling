# Doc Code Block Verification System

This system automatically extracts and compiles Rust code blocks from docs, ensuring all example code stays usable in CI.

## Config

Specify which Markdown files to verify via [`verified-docs.toml`](https://github.com/mingling-rs/mingling/blob/main/verified-docs.toml) in the project root.

You can also test a single file via command-line arg:

```sh
./run-tools.sh test-all-markdown-code docs/pages/1-getting-started.md
```
 
```powershell
.\run-tools.ps1 test-all-markdown-code docs/pages/1-getting-started.md
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

- Only ` ```rust ` fenced code blocks are extracted.
- Empty blocks (no code lines) are skipped.
- Blocks with `// NOT VERIFIED` alone are skipped.

### 2. Temp Project Generation

Each block (or each dedup-hash group) gets its own Cargo project:

```
.temp/doc-test/<hash>/
├── Cargo.toml
└── src/
    └── main.rs
```
 
### 3. Build Verification

Compiled with `cargo build --release`, stderr inherited to the terminal for real-time progress.

- **Build OK** → **PASS**
- **Build FAIL** → **FAIL**, last 20 lines of error captured.

### 4. Report

After all tests, a report is written to `.temp/DOCS-TEST-RESULT.md`, containing:

- Total tests, passed, failed
- Table of results per block (block #, file, line, status)
- Detailed errors for failed blocks

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

## Structure Overview

| Module                                        | Responsibility                                                                      |
| --------------------------------------------- | ----------------------------------------------------------------------------------- |
| `dev_tools/src/verify.rs`                     | Block parsing, Cargo.toml/main.rs generation, build exec, hash dedup, report output |
| `dev_tools/src/bin/test-all-markdown-code.rs` | Entry point: read config, collect files, orchestrate tests, aggregate results       |
| `verified-docs.toml`                          | Specifies which doc files to verify                                                 |

---

## Full Example

````markdown
```rust
// Features: ["parser"]
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
mingling = { path = "../../mingling", features = ["parser"] }
serde = { version = "1", features = ["derive"] }
```
