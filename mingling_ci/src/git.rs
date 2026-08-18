//! Thin wrappers around the `git` CLI used by the CI phase lock/unlock pair.

use std::ffi::OsStr;
use std::process::Command;

/// Marker file created by `git-lock` when the working tree is clean, so that a
/// temporary commit can always be made; `git-unlock` removes it. Its presence
/// marks "CI phase in progress".
pub(crate) const LOCK_FILE: &str = "MINGLING-CI-CHECKING";

/// Temporary commit message used by `git-lock`.
pub(crate) const TEMP_COMMIT_MESSAGE: &str = "[DO NOT PUSH] CI TEMP [DO NOT PUSH]";

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

/// Returns `true` when `git diff-index --quiet HEAD --` succeeds, i.e. the
/// working tree has no tracked changes. Git failures count as "not clean" so
/// the caller falls back to the marker-file path.
pub(crate) fn worktree_clean() -> bool {
    Command::new("git")
        .args(["diff-index", "--quiet", "HEAD", "--"])
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
