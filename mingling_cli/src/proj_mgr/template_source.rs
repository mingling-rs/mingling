//! Template source resolution for `proj-init`.
//!
//! A template source is either:
//! - a git remote template addressed as `<ref>@<variant>` (e.g. `0.4@basic`),
//!   where `ref` is a tag, branch or commit hash and `variant` is a
//!   subdirectory of the repository;
//! - a local template directory given by a plain path.

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use mingling::picker::{PickerArgResult, SinglePickable};

/// Default template source when none is configured: the `mingling-rs/tmpl`
/// repository on GitHub.
pub const DEFAULT_TMPL_SOURCE: &str = "mingling-rs/tmpl";

/// The template cache root: `~/.local/share/mingling/cache`.
pub fn cache_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_default()
        .join("mingling")
        .join("cache")
}

/// Normalize a template source spec into a full git URL.
///
/// - `mingling-rs/tmpl` -> `https://github.com/mingling-rs/tmpl.git`
/// - `https://github.com/mingling-rs/tmpl` -> `https://github.com/mingling-rs/tmpl.git`
/// - `https://example.com/tmpl.git` -> unchanged
pub fn normalize_source(source: &str) -> String {
    if source.ends_with(".git") {
        return source.to_string();
    }
    if source.contains("://") {
        // Ensure GitHub URLs share the same cache key as the short form.
        if let Some(rest) = source.strip_prefix("https://github.com/") {
            return format!("https://github.com/{rest}.git");
        }
        return source.to_string();
    }
    if let Some((owner, repo)) = source.split_once('/') {
        return format!("https://github.com/{owner}/{repo}.git");
    }
    source.to_string()
}

/// A template source provided by the user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateSource {
    /// Git remote template: `<ref>@<variant>`.
    Git { reference: String, variant: String },
    /// Local template directory.
    FsDir(PathBuf),
}

impl TemplateSource {
    /// Creates a git source from a `ref@variant` spec.
    pub fn git(reference: impl Into<String>, variant: impl Into<String>) -> Self {
        Self::Git {
            reference: reference.into(),
            variant: variant.into(),
        }
    }

    /// Creates a local-directory source.
    pub fn fs_dir(path: impl Into<PathBuf>) -> Self {
        Self::FsDir(path.into())
    }
}

impl SinglePickable for TemplateSource {
    fn pick_single(str: Option<&str>) -> PickerArgResult<Self> {
        let Some(raw) = str else {
            return PickerArgResult::NotFound;
        };

        // `<ref>@<variant>` — both sides non-empty.
        if let Some((reference, variant)) = raw.split_once('@')
            && !reference.is_empty()
            && !variant.is_empty()
        {
            return PickerArgResult::Parsed(Self::git(reference, variant));
        }

        // Plain path — reuse the PathBuf parsing (handles `~` expansion etc.).
        match <PathBuf as SinglePickable>::pick_single(str) {
            PickerArgResult::Parsed(path) => PickerArgResult::Parsed(Self::FsDir(path)),
            PickerArgResult::NotFound => PickerArgResult::NotFound,
            PickerArgResult::Unparsed => PickerArgResult::Unparsed,
        }
    }
}

/// Resolve a git template source to a local template directory.
///
/// The repository is shallow-cloned into
/// `<cache>/<source-hash>/<ref-hash>/` where `<source-hash>` is the first 16
/// hex chars of the SHA-256 of the source URL and `<ref-hash>` is the first 16
/// hex chars of the resolved commit. Already-cached references are reused. The
/// returned path is `<repo>/<variant>`, validated to contain a `checklist.toml`.
pub fn resolve_git(
    source_url: &str,
    reference: &str,
    variant: &str,
    cache: &Path,
) -> Result<PathBuf, String> {
    let source_hash = sha256_prefix16(source_url);
    let full_hash = resolve_commit(source_url, reference)?;
    let ref_hash = &full_hash[..16];
    let repo_dir = cache.join(source_hash).join(ref_hash);

    if !repo_dir.join(".git").is_dir() {
        shallow_clone(source_url, reference, &full_hash, &repo_dir)?;
    }

    let template_dir = repo_dir.join(variant);
    if !template_dir.join("checklist.toml").is_file() {
        return Err(format!(
            "variant `{variant}` has no checklist.toml in repository {source_url}"
        ));
    }
    Ok(template_dir)
}

/// First 16 hex chars of the SHA-256 digest of `input`.
fn sha256_prefix16(input: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let digest = hasher.finalize();
    digest.iter().take(8).map(|b| format!("{b:02x}")).collect()
}

/// Resolve `reference` (tag / branch / commit hash) to its full commit hash.
///
/// A full 40-hex reference is used as-is; otherwise `git ls-remote` resolves
/// it — first as a ref name (tag / branch), then by prefix-matching against
/// every advertised ref to support abbreviated commit hashes.
fn resolve_commit(source_url: &str, reference: &str) -> Result<String, String> {
    if reference.len() == 40 && reference.chars().all(|c| c.is_ascii_hexdigit()) {
        return Ok(reference.to_string());
    }

    // Ref name resolution (tag / branch).
    let by_ref = git_ls_remote(source_url, Some(reference))?;
    if let Some(hash) = by_ref
        .lines()
        .next()
        .and_then(|line| line.split('\t').next())
    {
        return Ok(hash.to_string());
    }

    // Abbreviated commit hash: prefix-match against all advertised refs.
    let all_refs = git_ls_remote(source_url, None)?;
    for line in all_refs.lines() {
        let Some(hash) = line.split('\t').next() else {
            continue;
        };
        if hash.starts_with(reference) {
            return Ok(hash.to_string());
        }
    }

    Err(format!("reference `{reference}` not found in {source_url}"))
}

/// Run `git ls-remote <url> [pattern]` and return its stdout.
fn git_ls_remote(source_url: &str, pattern: Option<&str>) -> Result<String, String> {
    let mut cmd = Command::new("git");
    cmd.args(["ls-remote", source_url]);
    if let Some(pattern) = pattern {
        cmd.arg(pattern);
    }
    let output = cmd
        .output()
        .map_err(|e| format!("failed to run `git ls-remote`: {e}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Shallow-clone `reference` from `source_url` into `dst`.
///
/// Uses `git init` + `git fetch --depth 1 origin <reference>` so that tags and
/// branches are handled uniformly. When the reference is not a ref name (a
/// commit hash), it falls back to fetching the resolved full hash — supported
/// by GitHub, but not by plain local `file://` protocols.
fn shallow_clone(
    source_url: &str,
    reference: &str,
    full_hash: &str,
    dst: &Path,
) -> Result<(), String> {
    if dst.exists() {
        fs::remove_dir_all(dst).map_err(|e| e.to_string())?;
    }
    fs::create_dir_all(dst).map_err(|e| e.to_string())?;

    run_git(dst, ["init", "-q"])?;
    run_git(dst, ["remote", "add", "origin", source_url])?;

    let fetched_by_ref = run_git(dst, ["fetch", "-q", "--depth", "1", "origin", reference]);
    if fetched_by_ref.is_err() {
        run_git(dst, ["fetch", "-q", "--depth", "1", "origin", full_hash])?;
    }

    // `git fetch` only writes FETCH_HEAD; the fresh repo has no branch yet, so
    // create one explicitly to populate the working tree.
    run_git(dst, ["checkout", "-q", "-B", "cache", "FETCH_HEAD"])?;
    Ok(())
}

/// Run a git command in `cwd`, returning an error message on failure.
fn run_git<const N: usize>(cwd: &Path, args: [&str; N]) -> Result<(), String> {
    let status = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .status()
        .map_err(|e| format!("failed to run git: {e}"))?;
    if !status.success() {
        return Err(format!("git command failed with {status}"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a local git repository at `dir` with one commit tagged `v0.1`
    /// and a `basic` variant subdirectory containing a checklist.
    fn make_repo(dir: &Path) {
        fs::create_dir_all(dir).unwrap();
        run_git(dir, ["init", "-q", "-b", "main"]).unwrap();
        fs::create_dir_all(dir.join("basic")).unwrap();
        fs::write(
            dir.join("basic").join("checklist.toml"),
            "program_name = \"x\"\n",
        )
        .unwrap();
        fs::write(dir.join("basic").join("rule.toml"), "").unwrap();
        run_git(dir, ["add", "."]).unwrap();
        run_git(
            dir,
            [
                "-c",
                "user.name=t",
                "-c",
                "user.email=t@t",
                "commit",
                "-q",
                "-m",
                "init",
            ],
        )
        .unwrap();
        run_git(dir, ["tag", "v0.1"]).unwrap();
    }

    #[test]
    fn parses_ref_at_variant_as_git() {
        let source = TemplateSource::pick_single(Some("0.4@basic")).unwrap();
        assert_eq!(source, TemplateSource::git("0.4", "basic"));
    }

    #[test]
    fn parses_plain_path_as_fs_dir() {
        let source = TemplateSource::pick_single(Some("/some/dir")).unwrap();
        assert_eq!(source, TemplateSource::FsDir(PathBuf::from("/some/dir")));
    }

    #[test]
    fn missing_input_is_not_found() {
        assert!(matches!(
            TemplateSource::pick_single(None),
            PickerArgResult::NotFound
        ));
    }

    #[test]
    fn empty_variant_falls_back_to_path() {
        // `ref@` has an empty variant — treat as a path, not a git source.
        let source = TemplateSource::pick_single(Some("0.4@")).unwrap();
        assert_eq!(source, TemplateSource::FsDir(PathBuf::from("0.4@")));
    }

    #[test]
    fn sha256_prefix_is_stable_and_16_chars() {
        let a = sha256_prefix16("https://example.com/repo.git");
        let b = sha256_prefix16("https://example.com/repo.git");
        let c = sha256_prefix16("https://example.com/other.git");
        assert_eq!(a, b);
        assert_eq!(a.len(), 16);
        assert_ne!(a, c);
    }

    #[test]
    fn normalize_source_default_and_configured() {
        assert_eq!(
            normalize_source(DEFAULT_TMPL_SOURCE),
            "https://github.com/mingling-rs/tmpl.git"
        );
        assert_eq!(
            normalize_source("https://example.com/tmpl.git"),
            "https://example.com/tmpl.git"
        );
        assert_eq!(
            normalize_source("some-one/other-tmpl"),
            "https://github.com/some-one/other-tmpl.git"
        );
        // GitHub URL without `.git` shares the cache key with the short form.
        assert_eq!(
            normalize_source("https://github.com/mingling-rs/tmpl"),
            "https://github.com/mingling-rs/tmpl.git"
        );
    }

    #[test]
    fn resolve_git_clones_tag_and_validates_variant() {
        let tmp =
            std::env::temp_dir().join(format!("mling-tmpl-src-test-{}-clone", std::process::id()));
        let repo = tmp.join("repo");
        let cache = tmp.join("cache");
        let _ = fs::remove_dir_all(&tmp);
        make_repo(&repo);

        let repo_url = format!("file://{}", repo.display());
        let template = resolve_git(&repo_url, "v0.1", "basic", &cache).unwrap();
        assert!(template.join("checklist.toml").is_file());

        // Cached under <source-hash>/<ref-hash> with a working tree.
        let source_hash = sha256_prefix16(&repo_url);
        let full_hash = resolve_commit(&repo_url, "v0.1").unwrap();
        let cached = cache.join(source_hash).join(&full_hash[..16]);
        assert!(cached.join(".git").is_dir());

        // Second resolution reuses the cache.
        let template_again = resolve_git(&repo_url, "v0.1", "basic", &cache).unwrap();
        assert_eq!(template, template_again);

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn resolve_git_rejects_unknown_variant() {
        let tmp =
            std::env::temp_dir().join(format!("mling-tmpl-src-test-{}-reject", std::process::id()));
        let repo = tmp.join("repo");
        let cache = tmp.join("cache");
        let _ = fs::remove_dir_all(&tmp);
        make_repo(&repo);

        let repo_url = format!("file://{}", repo.display());
        let err = resolve_git(&repo_url, "v0.1", "nope", &cache).unwrap_err();
        assert!(err.contains("nope"), "unexpected error: {err}");

        let _ = fs::remove_dir_all(&tmp);
    }
}
