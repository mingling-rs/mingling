//! Thin wrappers around the `git` CLI used by the CI phase lock/unlock pair.

use std::ffi::OsStr;
use std::process::Command;

/// Marker file created by `git-lock` in the CI temporary commit; its content
/// is `true` when the tree was dirty (a base TEMP commit exists below) or
/// `false` when it was clean. `git-unlock` reads it to pick the restore path.
pub(crate) const LOCK_FILE: &str = "MINGLING-CI-CHECKING";

/// First temporary commit: packs the dirty workspace changes so they can be
/// restored later. Only created when the tree is dirty.
pub(crate) const TEMP_COMMIT_MESSAGE: &str = "[DO NOT PUSH] TEMP [DO NOT PUSH]";

/// Second temporary commit: carries the marker file, and its message is what
/// `git-unlock` matches to confirm the CI phase.
pub(crate) const CI_TEMP_COMMIT_MESSAGE: &str = "[DO NOT PUSH] CI TEMP [DO NOT PUSH]";

/// Case-sensitive substring that identifies a CI temporary commit in the HEAD
/// commit message.
pub(crate) const TEMP_COMMIT_MARK: &str = "CI TEMP";

/// Runs `git <args>`, returning stdout on success.
///
/// # Errors
///
/// Returns the git error message (stderr) when the command exits non-zero, or
/// when git itself cannot be spawned.
pub(crate) fn run_git<I, S>(args: I) -> Result<String, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = Command::new("git")
        .args(args)
        .output()
        .map_err(|e| format!("failed to run git: {e}"))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

/// Returns `true` when the working tree has no tracked changes relative to
/// HEAD. Git failures count as "not clean" so the caller falls back to the
/// marker-file path.
///
/// Uses the porcelain `git diff --quiet HEAD` rather than the plumbing
/// `git diff-index --quiet HEAD`: after a full compile the source files'
/// mtimes can be newer than the index stat records even though their content
/// is unchanged, and `diff-index` reports that stale stat as a change. The
/// porcelain diff refreshes the index first (via `diff.autoRefreshIndex`),
/// so it only reports real content differences.
pub(crate) fn worktree_clean() -> bool {
    Command::new("git")
        .args(["diff", "--quiet", "HEAD", "--"])
        .status()
        .is_ok_and(|status| status.success())
}

/// The subject line of the HEAD commit.
///
/// # Errors
///
/// Returns the git error message when the log command fails.
pub(crate) fn head_message() -> Result<String, String> {
    run_git(["log", "-1", "--pretty=%s"]).map(|subject| subject.trim().to_string())
}
