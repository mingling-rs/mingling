use mingling::{
    Grouped, RenderResult, Routable,
    macros::{buffer, command, r_println, renderer},
    res::ResExitCode,
};

use crate::Next;
use crate::git::{CI_TEMP_COMMIT_MESSAGE, LOCK_FILE, TEMP_COMMIT_MESSAGE, run_git, worktree_clean};
use crate::res::{CargoError, MessagePrinter};

/// Temporarily commits the workspace so CI can run on a stable tree.
///
/// First pins the current HEAD to the `mingling/bkup` backup branch (created
/// or force-reset). When the tree is dirty, all changes are packed into a
/// plain `TEMP` commit first so they can be restored later; the `CI TEMP`
/// commit then carries only the `MINGLING-CI-CHECKING` marker file, whose
/// content (`true`/`false`) tells `git-unlock` which restore path to take.
#[command(node = "git-lock")]
pub fn git_lock() -> Next {
    if let Err(e) = run_git(["branch", "-f", "mingling/bkup", "HEAD"]) {
        return ErrorGitLock(e).to_chain();
    }

    let dirty = !worktree_clean();
    if dirty {
        if let Err(e) = run_git(["add", "."]) {
            return ErrorGitLock(e).to_chain();
        }
        if let Err(e) = run_git(["commit", "-m", TEMP_COMMIT_MESSAGE]) {
            return ErrorGitLock(e).to_chain();
        }
    }

    let marker = if dirty { "true" } else { "false" };
    if let Err(e) = std::fs::write(LOCK_FILE, marker) {
        return ErrorGitLock(format!("failed to create {LOCK_FILE}: {e}")).to_chain();
    }

    if let Err(e) = run_git(["add", "."]) {
        return ErrorGitLock(e).to_chain();
    }
    if let Err(e) = run_git(["commit", "-m", CI_TEMP_COMMIT_MESSAGE]) {
        return ErrorGitLock(e).to_chain();
    }

    ResultGitLock { dirty }.to_chain()
}

/// Whether the tree was dirty (a base `TEMP` commit exists) when locking.
#[derive(Grouped)]
pub struct ResultGitLock {
    dirty: bool,
}

#[derive(Grouped, Default)]
pub struct ErrorGitLock(pub String);

#[renderer(buffer)]
pub fn render_git_lock(r: ResultGitLock) {
    if r.dirty {
        r_println!("Locked: dirty workspace committed for CI");
    } else {
        r_println!("Locked: clean workspace marked for CI");
    }
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
