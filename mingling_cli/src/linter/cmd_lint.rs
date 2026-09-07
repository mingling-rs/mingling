use crate::linter::lint_cache::{
    CacheData, CachedFileInfo, FileCacheEntry, MlintCache, cache_key, content_hash,
    file_mtime_nanos, fill_source, roots_signature, strip_source,
};
use crate::linter::mlint_report::{MlintReport, StateLintReports};
use cargo_metadata::Metadata;
use mingling::consts::REMAINS;
use mingling::macros::{arg, chain, completion, dispatcher, metadata, suggest};
use mingling::metadata::Description;
use mingling::picker::parselib::ParserStyle;
use mingling::picker::value::Flag;
use mingling::picker::{EntryPicker, PickerArg};
use mingling::{Grouped, Wrap};
use mingling::{LazyRes, ShellContext, Suggest};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use tokio::task::JoinSet;

dispatcher!("lint", EntryLint);

const ARG_WITH_CHECKER: PickerArg<Option<String>> = arg![with_checker: Option<String>];

/// `--workspace` lints every workspace member instead of just the current package.
pub static ARG_WORKSPACE: PickerArg<Flag> = arg![workspace: Flag];

#[metadata(EntryLint)]
pub fn desc_lint() -> Description {
    "Mingling Linter".to_string().into()
}

/// Main linting function that processes selected packages in the metadata.
///
/// With `workspace == true` it lints every workspace member; otherwise it lints
/// only the package containing the current directory. For each selected target it
/// recursively expands the module tree into Rust source files (`.rs`), parses
/// them, runs lint checks, and enriches each report with metadata information.
async fn linter_main(metadata: &Metadata, workspace: bool) -> Vec<MlintReport> {
    // Only lint the workspace members (or just the current package) — never
    // third-party dependencies pulled into `metadata.packages`.
    let selected = select_packages(metadata, workspace);

    // Crate roots (all `.rs` target source files) that determine the lint input.
    let mut roots: Vec<String> = Vec::new();
    for package in &selected {
        for target in &package.targets {
            if target.src_path.as_str().ends_with(".rs") {
                roots.push(target.src_path.as_str().to_string());
            }
        }
    }
    let root_sig = roots_signature(&roots);
    let cache = MlintCache::new(Path::new(metadata.target_directory.as_str()));

    // ---- Fast no-op path ----------------------------------------------------
    // If the crate roots are unchanged and every previously discovered module
    // file still has the same mtime, nothing changed: reuse the cached reports
    // without re-parsing / re-hashing any file. Only files that actually produced
    // reports are read (to restore their source text for byte-accurate output).
    let loaded = cache.load();
    if loaded.roots_signature == root_sig && !loaded.files.is_empty() {
        let unchanged = loaded
            .files
            .iter()
            .all(|f| file_mtime_nanos(Path::new(&f.path)) == Some(f.mtime));
        if unchanged {
            let mut reused = Vec::new();
            for info in &loaded.files {
                if let Some(entry) = loaded.entries.get(&cache_key(Path::new(&info.path)))
                    && !entry.reports.is_empty()
                    && let Ok(source) = std::fs::read_to_string(&info.path)
                {
                    let mut reports = entry.reports.clone();
                    fill_source(&mut reports, &source);
                    reused.extend(reports);
                }
            }
            return reused;
        }
    }
    let base = std::sync::Arc::new(loaded);

    // ---- Slow path: discover the module tree and lint ------------------------
    // A single source file to lint, attributed to one compilation target.
    struct FileTask {
        path: String,
        package_id: String,
        target_name: String,
        target_kind: Option<String>,
        target_src_path: String,
    }

    let mut tasks: Vec<FileTask> = Vec::new();
    let mut seen = HashSet::new();
    for package in &selected {
        for target in &package.targets {
            let path = &target.src_path;
            if !path.as_str().ends_with(".rs") {
                continue;
            }
            let path_str = path.as_str().to_string();
            let package_id = package.id.to_string();
            let target_name = target.name.clone();
            let target_kind = target.kind.first().map(|k| k.to_string());
            let target_src_path = path_str.clone();

            for file in collect_module_files(PathBuf::from(path)) {
                let file_str = file.to_string_lossy().into_owned();
                if !seen.insert(file) {
                    continue;
                }
                tasks.push(FileTask {
                    path: file_str,
                    package_id: package_id.clone(),
                    target_name: target_name.clone(),
                    target_kind: target_kind.clone(),
                    target_src_path: target_src_path.clone(),
                });
            }
        }
    }
    let manifest_paths: Vec<String> = tasks.iter().map(|t| t.path.clone()).collect();

    let mut join_set = JoinSet::new();
    for task in tasks {
        let base = base.clone();
        join_set.spawn_blocking(move || {
            let path = PathBuf::from(&task.path);
            let key = cache_key(&path);
            let mtime = file_mtime_nanos(&path);

            // Reuse an unchanged file (mtime match) without re-reading or hashing.
            if let Some(mtime) = mtime
                && let Some(cached) = base.entries.get(&key)
                && cached.mtime == mtime
            {
                if cached.reports.is_empty() {
                    return Some(TaskOutcome {
                        key,
                        mtime,
                        content_hash: cached.content_hash,
                        reports: Vec::new(),
                    });
                }
                if let Ok(source) = std::fs::read_to_string(&path) {
                    let mut reports = cached.reports.clone();
                    fill_source(&mut reports, &source);
                    return Some(TaskOutcome {
                        key,
                        mtime,
                        content_hash: cached.content_hash,
                        reports,
                    });
                }
            }

            // Cache miss: read, parse, lint and re-enrich.
            let source = std::fs::read_to_string(&path).ok()?;
            let ast = syn::parse_file(&source).ok()?;
            let reports = crate::lints::run_all_lints(&ast, &source);
            let enriched: Vec<MlintReport> = reports
                .into_iter()
                .map(|mut r| {
                    r.file_name = task.path.clone();
                    r.source_code = source.clone();
                    r.package_id = Some(task.package_id.clone());
                    r.target_name = Some(task.target_name.clone());
                    r.target_kind = task.target_kind.clone();
                    r.target_src_path = Some(task.target_src_path.clone());
                    r
                })
                .collect();

            Some(TaskOutcome {
                key,
                mtime: mtime.unwrap_or(0),
                content_hash: content_hash(source.as_bytes()),
                reports: enriched,
            })
        });
    }

    let mut all_reports = Vec::new();
    let mut next_entries = HashMap::new();
    while let Some(res) = join_set.join_next().await {
        // `spawn_blocking` panics are propagated, `None` means task skipped.
        let Ok(Some(outcome)) = res else {
            continue;
        };
        all_reports.extend(outcome.reports.clone());
        let mut stored = outcome.reports;
        strip_source(&mut stored);
        next_entries.insert(
            outcome.key,
            FileCacheEntry {
                mtime: outcome.mtime,
                content_hash: outcome.content_hash,
                reports: stored,
            },
        );
    }

    // Persist the refreshed manifest + reports so the next no-op run is instant.
    let files = manifest_paths
        .into_iter()
        .map(|path| CachedFileInfo {
            mtime: file_mtime_nanos(Path::new(&path)).unwrap_or(0),
            path,
        })
        .collect();
    cache.store(&CacheData {
        roots_signature: root_sig,
        files,
        entries: next_entries,
    });

    all_reports
}

/// Per-file result returned by a lint task.
struct TaskOutcome {
    key: [u8; 32],
    mtime: u128,
    content_hash: [u8; 32],
    reports: Vec<MlintReport>,
}

/// Recursively collect `file` and every module file reachable from it via
/// `mod name;` declarations (non-inline modules).
///
/// Mirrors the module-resolution rules used by `mingling_pathf`:
/// - `main.rs` / `lib.rs` / `mod.rs` keep the module base in their own directory;
/// - any other `xxx.rs` resolves its children from the sibling directory `xxx/`;
/// - a child `foo` is looked up as `{base}/foo.rs`, then `{base}/foo/mod.rs`.
fn collect_module_files(root: PathBuf) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut visited = HashSet::new();
    collect_from_file(&root, &mut visited, &mut files);
    files
}

fn collect_from_file(file: &Path, visited: &mut HashSet<PathBuf>, files: &mut Vec<PathBuf>) {
    if !visited.insert(file.to_path_buf()) {
        return;
    }
    files.push(file.to_path_buf());

    // Parse to discover child `mod` declarations; a file that cannot be read or
    // parsed (e.g. a `#[cfg(...)]`-gated module whose file is absent) is skipped
    // rather than aborting the whole traversal.
    let Ok(content) = std::fs::read_to_string(file) else {
        return;
    };
    let Ok(ast) = syn::parse_file(&content) else {
        return;
    };

    for item in &ast.items {
        // `mod foo;` declares an external module file; inline `mod foo { }` does not.
        if let syn::Item::Mod(item_mod) = item
            && item_mod.semi.is_some()
            && let Some(child) = resolve_module_file(file, &item_mod.ident.to_string())
        {
            collect_from_file(&child, visited, files);
        }
    }
}

/// Resolve the file backing the module `name` declared inside `parent_file`,
/// applying standard Rust module path rules. Returns `None` when no candidate exists.
fn resolve_module_file(parent_file: &Path, module_name: &str) -> Option<PathBuf> {
    let parent_dir = parent_file.parent()?;
    let file_stem = parent_file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    // Rust module resolution:
    // - `mod.rs`, `main.rs`, `lib.rs` -> base is their own directory;
    // - `aaa.rs` -> base is the sibling directory `aaa/`.
    let module_base = if matches!(file_stem, "mod" | "main" | "lib") {
        parent_dir.to_path_buf()
    } else {
        parent_dir.join(file_stem)
    };

    [
        module_base.join(format!("{module_name}.rs")),
        module_base.join(module_name).join("mod.rs"),
    ]
    .into_iter()
    .find(|path| path.is_file())
}

/// Choose which packages to lint.
///
/// With `--workspace`, every workspace member is selected. Otherwise only the
/// workspace member whose manifest directory contains the current directory is
/// selected (the package you are running in); if none matches it falls back to
/// the workspace's default members, then all members.
fn select_packages(
    metadata: &cargo_metadata::Metadata,
    workspace: bool,
) -> Vec<&cargo_metadata::Package> {
    let member_ids: HashSet<cargo_metadata::PackageId> =
        metadata.workspace_members.iter().cloned().collect();
    let members = metadata
        .packages
        .iter()
        .filter(|p| member_ids.contains(&p.id))
        .collect::<Vec<_>>();

    if workspace {
        return members;
    }

    if let Ok(cwd) = std::env::current_dir() {
        // Deepest workspace member whose manifest dir is an ancestor of cwd.
        let mut best: Option<(&cargo_metadata::Package, usize)> = None;
        for package in &members {
            let dir = Path::new(&package.manifest_path).parent();
            if let Some(dir) = dir
                && cwd.starts_with(dir)
            {
                let depth = dir.components().count();
                if best.as_ref().is_none_or(|(_, d)| depth > *d) {
                    best = Some((package, depth));
                }
            }
        }
        if let Some((package, _)) = best {
            return vec![package];
        }
    }

    // Fall back to default members, then all members.
    let default_ids: HashSet<cargo_metadata::PackageId> =
        metadata.workspace_default_members.iter().cloned().collect();
    let defaults = members
        .iter()
        .copied()
        .filter(|p| default_ids.contains(&p.id))
        .collect::<Vec<_>>();
    if !defaults.is_empty() {
        defaults
    } else {
        members
    }
}

#[derive(Grouped, Wrap)]
pub struct StateBeginLinter(pub bool);

#[chain]
pub fn handle_lint(args: EntryLint) -> StateBeginLinter {
    let (workspace, with_checker, checker_args) = args
        .pick(&ARG_WORKSPACE)
        .pick_or(&ARG_WITH_CHECKER, || Some("cargo,check".to_string()))
        .pick(&REMAINS)
        .unwrap();
    let workspace = *workspace;

    // If with_checker is not set, proceed directly to the mingling lint phase.
    let Some(with_checker) = with_checker else {
        return StateBeginLinter(workspace);
    };

    let with_checker: Vec<&str> = with_checker.split(',').collect();
    let checker_args: Vec<String> = checker_args.into();

    // Run the outer checker (e.g. cargo check) with output passed through directly.
    execute_checker(&with_checker, checker_args.as_slice());

    StateBeginLinter(workspace)
}

/// Run the outer checker (e.g. cargo check) with output passed through directly.
fn execute_checker(with_checker: &[&str], checker_args: &[String]) {
    if with_checker.is_empty() {
        return;
    }

    let checker_str = with_checker.join(" ");
    let args_str = checker_args.join(" ");
    let full_cmd = if args_str.is_empty() {
        checker_str
    } else {
        format!("{} {}", checker_str, args_str)
    };

    let mut cmd = if cfg!(target_os = "windows") {
        let mut c = std::process::Command::new("cmd");
        c.args(["/C", &full_cmd]);
        c
    } else {
        let mut c = std::process::Command::new("sh");
        c.args(["-c", &full_cmd]);
        c
    };

    // Pass through stdin/stdout/stderr so the user sees everything
    let _ = cmd.status();
}

#[chain]
pub async fn handle_state_begin_linter(
    state: StateBeginLinter,
    metadata: &mut LazyRes<crate::metadata::setup::ResMetadata>,
) -> StateLintReports {
    let metadata = metadata.get_ref().data();
    let reports = linter_main(metadata, state.0).await;
    StateLintReports(reports)
}

#[completion(EntryLint)]
pub fn complete_lint(ctx: ShellContext) -> Suggest {
    if mingling::picker::parselib::build_possible_flags(
        ParserStyle::global_style(),
        &ARG_WITH_CHECKER.into_info(),
    )
    .contains(&ctx.previous_word)
    {
        suggest! {
            "cargo,check": "Also run `cargo check` for checking",
            "cargo,clippy": "Also run `cargo clippy` for checking",
        }
    } else {
        suggest! {
            ARG_WORKSPACE: "Lint all workspace members instead of only the current package",
            ARG_WITH_CHECKER: "Comma-separated Rust Analyzer-compatible checkers to also run, e.g. `cargo,check`",
        }
    }
}
