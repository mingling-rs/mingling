# About Mingling CI Process

Mingling's CI process is built into the project, with its execution logic located in `dev_tools/src/bin/ci.rs`. You can run it locally via the `cargo ci` command, which produces the same results as the `CI` workflow in GitHub Actions.

During development, you can run `cargo ci` at any time to verify that your code hasn't introduced regressions.

## Running Locally

An alias is defined in `.cargo/config.toml` at the project root:

```toml
[alias]
ci = "run --manifest-path dev_tools/Cargo.toml --bin ci --quiet --"
```

Simply execute:

```bash
cargo ci
```

## CI Execution Flow

`cargo ci` runs the following stages in order:

### 1. Code Checking Stage (run by default, or individually via `--test-codes`)

- **Scan and build all crates**: Recursively finds all `Cargo.toml` files in the project and runs `cargo build` for each crate in parallel.
- **Run Clippy on all crates**: Executes `cargo clippy ... -- -D warnings` in parallel; any warning will cause a failure.
- **Run unit tests for all crates**: Executes `cargo test` in parallel.

### 2. Documentation and Example Checking Stage (run by default, or individually via `--test-docs`)

- **Test all examples**: Runs the `test-examples` tool.
- **Verify Markdown code blocks compile**: Runs the `test-all-markdown-code` tool to check code blocks in all `*.md` files. See [ABOUT_CODE_VERIFY](docs/_ABOUT_CODE_VERIFY.md) for details.
- **Check if documentation is up to date**: Runs the following documentation refresh tools in sequence:
    - `docs-code-box-fix`
    - `docsify-sidebar-gen`
    - `refresh-docs`
    - `refresh-feature-mod`
    - `sync-examples`
- Finally, runs `cargo fmt` to unify code formatting.

### 3. File Normalization

Runs `git add --renormalize .` to ensure file attributes such as line endings conform to the repository configuration.

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
- Runs `cargo ci` in parallel on `ubuntu-latest` and `windows-latest`.
- After CI passes, the `unreleased` tag is automatically moved to the latest commit on `main`.
