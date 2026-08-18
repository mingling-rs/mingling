<h1 align="center">About Mingling CI Process</h1>
<p align="center">
    CI workflow and local execution guide for Mingling
</p>

Mingling's CI process is built into the project itself: the execution logic lives in `mingling_ci/`, a separate crate **built on the Mingling framework** — it dogfoods the very library it validates. You can run it locally via the `cargo ci` command, which produces the same results as the `CI` workflow in GitHub Actions.

During development, you can run `cargo ci <command>` at any time to verify that your code hasn't introduced regressions.

## Running Locally

An alias is defined in `.cargo/config.toml` at the project root:

```toml
[alias]
ci = "run --manifest-path mingling_ci/Cargo.toml --bin ci --quiet --"
```
 
Run a single step:

```bash
cargo ci build-check
```
 
Run the full pipeline (lock → all checks → refresh → unlock) via the orchestration script:

```bash
python .run/src/bin/ci.py
```
 
The script is also picked up by `run.sh`:

```bash
./run.sh ci
```
 
## Commands

Every CI step is one subcommand. `cargo ci` with no subcommand prints the help page.

### UTILS

| Command         | What it does                                                            |
| --------------- | ----------------------------------------------------------------------- |
| `report-collect` | Assembles the collected logs in `.temp/reports/collect/` into `.temp/reports/result.md` |
| `report-clean`  | Deletes all collected logs and the generated report                     |
| `git-lock`      | Locks the workspace for a CI run (temporary commit, see below)          |
| `git-unlock`    | Restores the workspace and checks idempotency (see below)               |
| `show-manifests` | Prints every crate path that CI will check                              |
| `show-features` | Prints the `docs.rs` feature list of `mingling`                         |

### TOOLS (refresh)

| Command             | What it does                                                                      |
| ------------------- | --------------------------------------------------------------------------------- |
| `example-refresh`   | Regenerates `mingling/src/example_docs.rs` and `docs/example-pages/examples.json` |
| `docsify-refresh`   | Fixes docsify code-box blank lines and regenerates `_sidebar.md` files            |
| `features-refresh`  | Regenerates `mingling/src/features.rs` from `mingling/Cargo.toml`                 |

These tools **write files**. Running them inside a `git-lock` / `git-unlock` pair turns them into an up-to-date check: if the generated files are stale, the tree becomes dirty and `git-unlock` fails.

### TASKS (checks)

| Command                 | What it does                                                                                                        |
| ----------------------- | ------------------------------------------------------------------------------------------------------------------- |
| `build-check`           | Finds all `Cargo.toml` files (minus `.config/ci-ignored-dirs.txt`) and runs `cargo build` per crate in parallel.   |
| `clippy-check`          | Runs `cargo clippy ... -- -D warnings` for every crate in parallel; any warning fails the check.                    |
| `test-all`              | Runs `cargo test` for every crate in parallel. Each base crate can override its command in its `mingling-ci.toml` (`[test].command`, with `<<<features>>>` expanded from the docs.rs feature list); `arg-picker` uses this to run `cargo test -p arg-picker`. |
| `example-check`         | Builds every example and runs the expected-output tests declared in `examples/<example>/test.toml`.                 |
| `docs-check`            | Builds the `mingling` API docs with the `[package.metadata.docs.rs]` features and `-D warnings`.                   |
| `markdown-check <PATH>` | Verifies the rust code blocks of a single markdown file compile. See [ABOUT_CODE_VERIFY](docs/_ABOUT_CODE_VERIFY.md). |
| `markdown-check-all`    | Verifies all markdown files declared in `.config/verified-docs.toml`.                                              |
| `markdown-compare <A> <B>` | Compares the *structure* of two markdown files or directories.                                                  |
| `markdown-compare-all`  | Checks every translated docs directory mirrors the reference `./docs/pages/` (per `.config/docs-lang.txt`).         |

## Reports

Every check exports its per-item outcome through the reporter into `.temp/reports/collect/`:

- `{Task}.{Platform}.ok` — one `item = location` per line (aggregated on flush).
- `{Task}.{Platform}.{item}.err` — failures, first line is the location.

`cargo ci report-collect` reads that directory and renders the consolidated report (per-task tables + failure details) to `.temp/reports/result.md`:

```bash
cargo ci report-collect
cat .temp/reports/result.md
```
 
`cargo ci report-clean` wipes both the collect directory and the report.

## Workspace Cleanliness and Temporary Commits

To ensure reproducible CI results, CI runs inside a `git-lock` / `git-unlock` pair.

### git-lock

1. Pins the current HEAD to a backup branch: `git branch -f mingling/bkup HEAD` (created or force-reset).
2. If the working tree is dirty, all changes are packed into a plain temporary commit `[DO NOT PUSH] TEMP [DO NOT PUSH]`.
3. A marker file `MINGLING-CI-CHECKING` is written — content `true` when the tree was dirty, `false` when it was clean.
4. Everything is committed as `[DO NOT PUSH] CI TEMP [DO NOT PUSH]`.

```
clean:  A ── CI TEMP (marker = false)
dirty:  A ── TEMP (your changes) ── CI TEMP (marker = true)
```
 
### git-unlock

Only acts when the HEAD commit message contains `CI TEMP` (case-sensitive); otherwise it refuses with a non-zero exit. The restore path is picked by the marker:

- `true` — hard reset past the marker commit, then soft reset + unstage, so **your pre-lock changes are restored into the working tree**.
- `false` — a single hard reset back to the original HEAD.

If the working tree is dirty when unlocking (e.g. CI left tracked changes behind, such as stale generated docs), the restore still runs but the command reports a **non-zero exit code** — this is the idempotency check. In CI, that fails the job.

> **Warning**: when unlocking a `true` lock, changes made *during* CI are discarded. Anything you had before locking comes back.

## GitHub Actions Workflow

`.github/workflows/ci.yml` defines the project's CI:

- Triggered on `push` to the `main` branch.
- A `Check` job runs in a **item × platform** matrix (`ubuntu-latest`, `windows-latest`, `macos-latest`), each combination being `cargo ci <command>` inside a `git-lock` / `git-unlock` pair:

| Matrix item    | Command                                                         |
| -------------- | -------------------------------------------------------------- |
| `build`        | `cargo ci build-check`                                         |
| `clippy`       | `cargo ci clippy-check`                                        |
| `test`         | `cargo ci test-all`                                            |
| `arg-picker`   | `cargo ci test-all` (covered via its `mingling-ci.toml` override) |
| `markdown-code` | `cargo ci markdown-check-all && cargo ci markdown-compare-all` |
| `examples`     | `cargo ci example-check`                                       |
| `docs-refresh` | `cargo ci example-refresh` + `docsify-refresh` + `features-refresh` |
| `api-docs`     | `cargo ci docs-check`                                          |

- Every matrix job uploads its `.temp/reports/collect/` as an artifact — **even on failure**, so failures are always collected.
- A `Report` job (runs even when some checks failed) downloads all collect artifacts, runs `cargo ci report-collect`, and publishes `result.md` to the job summary via `$GITHUB_STEP_SUMMARY`.
- After CI passes, the `unreleased` tag is automatically moved to the latest commit on `main`.

For non-`main` branches and pull requests, `.github/workflows/ci-check-only.yml` runs the same matrix and report collection without moving the tag or deploying.

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
