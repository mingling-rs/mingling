use mingling::{
    Grouped, RenderResult, Routable,
    macros::{buffer, command, r_println, renderer},
    res::ResExitCode,
};

use crate::Next;
use crate::git::{LOCK_FILE, TEMP_COMMIT_MESSAGE, run_git, worktree_clean};
use crate::res::{CargoError, MessagePrinter};

/// Temporarily commits the workspace so CI can run on a stable tree.
///
/// First pins the current HEAD to the `mingling/bkup` backup branch (created
/// or force-reset), then commits everything with a `[DO NOT PUSH] CI TEMP`
/// message. When the tree has no tracked changes, a `MINGLING-CI-CHECKING`
/// marker file is created first so the commit is never empty.
#[command(node = "git-lock")]
pub fn git_lock() -> Next {
    if let Err(e) = run_git(["branch", "-f", "mingling/bkup", "HEAD"]) {
        return ErrorGitLock(e).to_chain();
    }

    if worktree_clean()
        && let Err(e) = std::fs::write(LOCK_FILE, "")
    {
        return ErrorGitLock(format!("failed to create {LOCK_FILE}: {e}")).to_chain();
    }

    if let Err(e) = run_git(["add", "."]) {
        return ErrorGitLock(e).to_chain();
    }
    if let Err(e) = run_git(["commit", "-m", TEMP_COMMIT_MESSAGE]) {
        return ErrorGitLock(e).to_chain();
    }

    ResultGitLock {}.to_chain()
}

#[derive(Grouped)]
pub struct ResultGitLock;

#[derive(Grouped, Default)]
pub struct ErrorGitLock(pub String);

#[renderer(buffer)]
pub fn render_git_lock(_: ResultGitLock) {
    r_println!("Locked: CI temp commit created");
}

#[renderer]
pub fn render_error_git_lock(
    e: ErrorGitLock,
    error: &CargoError,
    exit_code: &mut ResExitCode,
) -> RenderResult {
    let render_result = RenderResult::new();
    error.println(vec![format!("Git-Lock: {}", e.0)]);
    exit_code.exit_code = 1;
    render_result
}
