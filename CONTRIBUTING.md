# Contribution Guide

First of all, thank you for your interest in Mingling! 🎉 Whether it's fixing bugs, improving documentation, adding new features, or making suggestions, we welcome all contributions.

Before contributing, we recommend reading [README](README.md) to get an overview of the project.

## 1. Project Structure 📦

| Category                    | Path/Name                 | Description                                                        |
| --------------------------- | ------------------------- | ------------------------------------------------------------------ |
| **Entry crate**             | `mingling/`               | Project entry point                                                |
| **Core library**            | `mingling_core/`          | Imported as an external dependency                                 |
| **Macro library**           | `mingling_macros/`        | Imported as an external dependency                                 |
| **Mingling Pathfinder**     | `mingling_pathf/`         | Build-time module path resolution for types                        |
| **Mingling Picker2**        | `mingling_picker/`        | Mingling Arguments Parser                                          |
| **Mingling Picker2 Macros** | `mingling_picker_macros/` | Mingling Arguments Parser Macros                                   |
| **Scaffolding tool**        | `mling/`                  | Scaffolding tool `mingling-cli`                                    |
| **Examples**                | `examples/`               | To add expected output tests, modify `examples/test-examples.toml` |
| **Documents**               | `docs/`                   | All documents                                                      |
| **Dev Documents**           | `docs/dev/`               | Internal documents                                                 |
| **Resources**               | `docs/res/`               | All resources                                                      |
| **Development tools**       | `.run/src/bin`            | Contains scripts and Rust tools                                    |
| **CI**                      | `.run/src/bin/ci.rs`      | Can be invoked directly via `cargo ci`                             |
| **Temporary files**         | `.temp/`                  | Ignored by `.gitignore`                                            |

## 2. How to Contribute

### Code Contribution

If you'd like to contribute to `mingling`, `mingling_core`, `mingling_macros`, or `mingling_pathf`, first share your idea on the [Github Issue](https://github.com/mingling-rs/mingling/issues) page to confirm before starting work.

- **Before making changes**, make sure your branch stays **as close as possible** to the upstream `main` branch.
- **After finishing**, run `cargo ci` locally (see [ABOUT CI](https://mingling-rs.github.io/mingling/docs/dev/#/pages/abouts/ci) for how it works). If `cargo ci` passes locally, your changes are most likely correct.

### Example Code Contribution

To add or modify examples under `examples/`, follow these rules:

- Place each example in `examples/&lt;example-dir&gt;/`
- Each dir must contain a `page.toml` file describing the example's metadata
- `page.toml` format:

```toml
[example]
id = "example-id"          # Unique identifier
name = "Example Name"      # Display name (optional, defaults to dir name)
icon = "📦"                # Icon (optional, defaults to "📦")
category = ""              # Category (optional)
desc = "Description"       # Description (optional)
tags = ["tag1", "tag2"]    # Tags (optional)
files = ["Cargo.toml", "src/main.rs"]
```

If you change expected behavior, update the test assertions in `examples/test-examples.toml`.

After editing examples, run these scripts to keep things in sync:

```bash
# Ensure code compiles
./run.sh build-all

# Ensure code style
./run.sh clippy

# Sync page.toml info to docs/example-pages/examples.json
./run.sh sync-examples

# Check all examples behave as expected
./run.sh test-examples

# Sync examples content into mingling/src/example_docs.rs
./run.sh refresh-docs

# (Optional) Preview the Example Viewer in a browser
# Requires: Python
./run.sh http-page-preview
# http://127.0.0.1:3000/
```

### Documentation Contribution

To contribute docs, edit files under `docs/`. For other language translations, refer to the structure under `docs/zh_CN`.

- **When editing docs**, prioritize **Chinese docs** first, then **English docs** — since I ([@Weicao-CatilGrass](https://github.com/Weicao-CatilGrass)) am a native Chinese speaker, this is more efficient.
- If your changes involve code, check the [Code Verify System](https://mingling-rs.github.io/mingling/docs/dev/#/pages/abouts/code-verify-system), which explains how CI checks code.
- **Before submitting**, always run:

```bash
# Fix code block issues in docsify
./run.sh docs-code-box-fix

# Generate sidebar
./run.sh docsify-sidebar-gen

# Verify all Markdown code blocks compile
./run.sh test-all-markdown-code
```

### Web Frontend Contribution

No strict requirements here — just modify the relevant `*.html` files. Preview with:

```bash
# Requires: Python
./run.sh http-page-preview
# http://127.0.0.1:3000/
```

### Dev Tool Contribution

`Mingling CI` code is under strict review. If you want to improve `mingling`'s CI pipeline or other dev tools (under `.run/`), **please** first file an [Issue](https://github.com/mingling-rs/mingling/issues) and contact [Weicao-CatilGrass](https://github.com/Weicao-CatilGrass)!

## 3. Submission Guide 🖊

1. **Pull Request**
   - Submit a GitHub Pull Request and @Reviewer **[Weicao-CatilGrass](https://github.com/Weicao-CatilGrass)** for review
   - Or send patches to **catil_grass@qq.com**

2. **Commit Messages**
   - Clearly and concisely describe the changes, no stringent requirements
   - Provide more detail for complex changes, keep it brief for simple changes
   - But: if you use [Conventional Commits](https://www.conventionalcommits.org/), it would make me even happier :)

3. **CHANGELOG**
   - If the submission includes functional changes or fixes, **the PR must include modifications to CHANGELOG.md** to describe the changes
   - For minor changes like typo fixes, **CHANGELOG.md modification is not required**, and we will merge faster

4. **Multi-commit PR**

   - A PR can contain multiple commits
   - However, at least one commit must modify CHANGELOG.md

5. **Review**
   - After submission, please notify [Weicao-CatilGrass](https://github.com/Weicao-CatilGrass) for review — this is the most efficient way to get feedback

6. **Binary Resources**
   - For binary resource files (images, etc.), please be cautious about adding them to avoid repository bloat

## 4. Documentation Contribution 📕

### Documentation Location

- English documentation: `docs/pages/`
- Chinese documentation: `docs/_zh_CN/pages/`

### Documentation Build

After editing documentation, refresh relevant files:

```bash
# Refresh sidebar and README sync
./run.sh docsify-sidebar-gen
./run.sh refresh-docs

# Fix code block blank line issues
./run.sh docs-code-box-fix
```

These steps are included in `cargo ci`; running `cargo ci` will execute them automatically.

> [!TIP]
> You can check the [ABOUT CI](https://mingling-rs.github.io/mingling/docs/dev/#/pages/abouts/ci) section to learn how "Mingling CI" works.

## 5. Regarding AI Agent Usage 🤖

- You are free to use AI agents to assist development — no restrictions
- **Humans are the final decision-makers**, everything is subject to human judgment
- Please **DO NOT** leave AI instruction files like `CLAUDE.md` in the repository root. Mingling currently has no plans to introduce **Harness Engineering**

## 6. License 📖

Mingling uses the **MIT + Apache 2.0** dual license. For details, please see:

- [LICENSE-MIT](LICENSE-MIT)
- [LICENSE-APACHE](LICENSE-APACHE)
