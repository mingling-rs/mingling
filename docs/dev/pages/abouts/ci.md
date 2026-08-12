<h1 align="center">About Mingling CI Process</h1>
<p align="center">
    CI workflow and local execution guide for Mingling
</p>

Mingling's CI process is built into the project, with its execution logic located in `.run/src/bin/ci.rs`. You can run it locally via the `cargo ci` command, which produces the same results as the `CI` workflow in GitHub Actions.

During development, you can run `cargo ci` at any time to verify that your code hasn't introduced regressions.

## Running Locally

An alias is defined in `.cargo/config.toml` at the project root:

```toml
[alias]
ci = "run --manifest-path .run/Cargo.toml --bin ci --quiet --"
```
 
Simply execute:

```bash
cargo ci
```
 
## CI Steps

Every CI step is an independent switch (`--check-*`). Running `cargo ci` with no options executes **all** steps in the order below; pass one or more `--check-*` flags to run only the selected steps.

| Step            | Flag                    | What it does                                                                                                                                       |
| --------------- | ----------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------- |
| Build           | `--check-build`         | Recursively finds all `Cargo.toml` files and runs `cargo build` for each crate in parallel (workspace members build with all documented features). |
| Clippy          | `--check-clippy`        | Runs `cargo clippy ... -- -D warnings` for every crate in parallel; any warning fails the check.                                                   |
| Test            | `--check-test`          | Runs `cargo test` for every crate in parallel (workspace tests run with all documented features; `arg-picker` is excluded).                        |
| Arg picker      | `--check-arg-picker`    | Runs `cargo test -p arg-picker` with its default features.                                                                                         |
| Markdown code   | `--check-markdown-code` | Runs the `test-all-markdown-code` tool to verify code blocks in all `*.md` files compile. See [ABOUT_CODE_VERIFY](docs/_ABOUT_CODE_VERIFY.md).     |
| Examples        | `--check-examples`      | Runs the `test-examples` tool to verify all examples behave as expected.                                                                           |
| Docs up to date | `--check-docs-refresh`  | Runs the documentation refresh tools and `cargo fmt`, then fails if the working tree is no longer clean (i.e. the docs were stale).                |
| API docs        | `--check-api-docs`      | Builds API docs with the `[package.metadata.docs.rs]` features and fails if `docs/api-docs/` is out of date.                                       |

### Docs up to date in detail

`--check-docs-refresh` runs the following documentation refresh tools in sequence:

- `docs-code-box-fix`
- `docsify-sidebar-gen`
- `refresh-docs`
- `refresh-feature-mod`
- `sync-examples`

Finally, it runs `cargo fmt` to unify code formatting. Because the refresh tools regenerate derived files, running this check against stale documentation modifies the working tree — and `ci.rs` fails the run in that case. (Using `--dirty` skips the cleanliness check, which makes this flag behave like a plain "refresh docs" command.)

### Combining steps

When several `--check-*` flags are combined, the steps run in the order listed above. In "run all" mode (no flags given), the documentation steps all execute even if one of them fails, so every problem is reported in a single run.

## File Normalization

Regardless of which steps run, `cargo ci` finishes with `git add --renormalize .` to ensure file attributes such as line endings conform to the repository configuration.

## Workspace Cleanliness and Temporary Commits

To ensure reproducible CI results, `ci.rs` imposes strict requirements on the workspace state:

- If the current workspace is not clean and `--dirty` has not been specified, the script will prompt whether to create a temporary commit:
    - The commit message is `[DO NOT PUSH] CI TEMP [DO NOT PUSH]`.
    - Use `-y` to auto-confirm without interaction.
- After CI finishes, the script automatically restores the workspace:
    - First, `git reset --hard` discards all changes.
    - If a temporary commit was created, it then runs `git reset --soft HEAD~1` and unstages everything, restoring the state to before CI started.
- If `--dirty` is specified, the temporary commit and the final cleanliness check are skipped.

> **Warning**: `git reset --hard` is executed at the end of CI. If you use `--dirty`, ensure you have no unsaved important changes.

## GitHub Actions Workflow

`.github/workflows/ci.yml` defines the project's CI:

- Triggered on `push` to the `main` branch.
- A single `Check` job runs every `--check-*` step in a **step × platform** matrix (`ubuntu-latest` and `windows-latest`), i.e. `cargo ci --check-<item>` for each combination. The `.temp` build cache is no longer used; every matrix job starts from a clean workspace.
- After CI passes, the `unreleased` tag is automatically moved to the latest commit on `main`.

For non-`main` branches and pull requests, `.github/workflows/ci-check-only.yml` runs the same matrix without moving the tag or deploying.

### API Documentation Deployment

After all checks pass, the `Deploy-Github-Pages` job:

- **Runs `deploy-api-docs`**: Executes `cargo run --manifest-path .run/Cargo.toml --bin deploy-api-docs`, which reads the `[package.metadata.docs.rs]` features from `mingling/Cargo.toml` and builds the crate's documentation via `cargo doc --no-deps`. The output is placed at `docs/api-docs/`.
- **Deploys to GitHub Pages**: The entire repository (including the generated `docs/api-docs/`) is uploaded and published to GitHub Pages.

You can view the published API documentation at:

> [https://mingling-rs.github.io/mingling/docs/api-docs/mingling/](https://mingling-rs.github.io/mingling/docs/api-docs/mingling/)

To generate API docs locally:

```bash
cargo run --manifest-path .run/Cargo.toml --bin deploy-api-docs
```
