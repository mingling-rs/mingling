# Contribution Guide

First of all, thank you for your interest in Mingling! 🎉
Whether it's fixing bugs, improving documentation, adding new features, or making suggestions,
we welcome all contributions.

Before contributing, we recommend reading [README](README.md) to get an overview of the project.

## 1. Project Structure 📦

| Category                | Path/Name            | Description                                                             |
| ----------------------- | -------------------- | ----------------------------------------------------------------------- |
| Main                    |                      |                                                                         |
| **Entry crate**         | `mingling/`          | Project entry point                                                     |
| **Mingling CLI**        | `mingling_cli/`      | Mingling's toolchains                                                   |
| **Core library**        | `mingling_core/`     | Imported as an external dependency                                      |
| **Macro library**       | `mingling_macros/`   | Imported as an external dependency                                      |
| **Mingling Pathfinder** | `mingling_pathf/`    | Build-time module path resolution for types                             |
| **Arg Picker**          | `arg_picker/`        | Mingling Arguments Parser                                               |
| **Arg Picker Macros**   | `arg_picker_macros/` | Mingling Arguments Parser Macros                                        |
| Documents               |                      |                                                                         |
| **Examples**            | `examples/`          | To add expected output tests, add a `test.toml` in the example's dir    |
| **Help Documents**      | `docs/pages/`        | User-facing help documents (raw)                                        |
| **Help Documents**      | `docs/[LANG]/pages`  | User-facing help documents (translations)                               |
| **Dev Documents**       | `docs/dev/`          | Internal documents                                                      |
| **Resources**           | `docs/res/`          | All resources                                                           |
| Dev Tools               |                      |                                                                         |
| **CI system**           | `mingling_ci/`       | CI crate built on the Mingling framework, invoked via `cargo ci`        |
| **CI configs**          | `.config/`           | `ci-ignored-dirs.txt`, `verified-docs.toml`, `docs-lang.txt`            |
| **CI orchestration**    | `.run/src/bin/ci.py` | Full pipeline script (lock → checks → refresh → unlock)                 |
| **Development tools**   | `.run/src/bin`       | Contains scripts and Rust tools (`deploy-api-docs`, `install-mling`, …) |
| Misc                    |                      |                                                                         |
| **Temporary files**     | `.temp/`             | Ignored by `.gitignore`                                                 |

## 2. How to Contribute

To ensure your contribution goes smoothly, please choose the guide that best fits your contribution area.

### Code Contribution

If you'd like to contribute to `mingling`, `mingling_core`, `mingling_macros`, or `mingling_pathf`,
first share your idea on the [Github Issue](https://github.com/mingling-rs/mingling/issues) page to confirm before starting work.

- **Before making changes**, make sure your branch stays **as close as possible** to the upstream `main` branch.
- **After finishing**, run the full CI pipeline locally (see [ABOUT CI](https://mingling-rs.github.io/mingling/docs/dev/#/pages/abouts/ci) for how it works):

```bash
./run.sh ci
```

If it passes locally, your changes are most likely correct. (`./run.sh ci` runs `python .run/src/bin/ci.py`,
which locks the workspace, runs every check and refresh step, then unlocks — the final unlock fails
if the run left the tree dirty.)

### Example Code Contribution

To add or modify examples under `examples/`, follow these rules:

- Place each example in `examples/{{example-dir}}/`
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

Optionally, each example may contain a `test.toml` file declaring expected output tests, which are executed by CI (`cargo ci example-check`):

```toml
[[runs]]
input = ["greet", "Alice"]

expect.exit-code = 0
expect.result = "Hello, Alice!"
```

- `input` is the list of CLI arguments passed to the example binary
- `expect.exit-code` / `expect.result` assert the expected process exit code and stdout output

If you change expected behavior, update the assertions in the example's `test.toml`.

After editing examples, run these commands to keep things in sync:

```bash
# Check all examples behave as expected
cargo ci example-check

# Sync examples content into mingling/src/example_docs.rs and examples.json
cargo ci example-refresh

# (Optional) Preview the Example Viewer in a browser
# Requires: Python
./run.sh http-page-preview
# http://127.0.0.1:3000/
```

### Documentation Contribution

To contribute to the documentation, please edit the files under `docs/`.
For translations into other languages, refer to the directory structure under `docs/zh_CN`.

> [!Warning]
>
> Mingling's API is currently unstable, so adding documentation content may not be
> the best use of effort at this stage.
>
> It is recommended to **only fix typos, grammar issues, etc.**

#### Verify

Documentation is checked by `cargo ci markdown-check-all` and `cargo ci markdown-compare-all` to ensure that **all** languages have consistent documentation structure and that the code compiles.

You can verify the documentation with the following commands:

```bash
# Fix code box issues and regenerate sidebars
cargo ci docsify-refresh

# Verify all Markdown code blocks compile
cargo ci markdown-check-all

# Verify translated docs mirror the reference structure
cargo ci markdown-compare-all
```

### Web Frontend Contribution

No strict requirements here — just modify the relevant `*.html` files. Preview with:

```bash
# Requires: Python
./run.sh http-page-preview
# http://127.0.0.1:3000/
```

### Dev Tool Contribution

`Mingling CI` code is under strict review. If you want to improve `mingling`'s CI pipeline (`mingling_ci/`) or other dev tools (under `.run/`),
**please** first file an [Issue](https://github.com/mingling-rs/mingling/issues) and contact [Weicao-CatilGrass](https://github.com/Weicao-CatilGrass)!

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

## 5. Regarding AI Agent Usage 🤖

- You are free to use AI agents to assist development — no restrictions
- **Humans are the final decision-makers**, everything is subject to human judgment
- Please **DO NOT** leave AI instruction files like `CLAUDE.md` in the repository root. Mingling currently has no plans to introduce **Harness Engineering**

## 6. License 📖

Mingling uses the **MIT + Apache 2.0** dual license. For details, please see:

- [LICENSE-MIT](LICENSE-MIT)
- [LICENSE-APACHE](LICENSE-APACHE)

# Contributors ❤

<img src="https://contrib.rocks/image?repo=mingling-rs/mingling" />
