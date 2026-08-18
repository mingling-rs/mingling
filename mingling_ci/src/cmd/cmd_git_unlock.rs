use mingling::{
    Grouped, RenderResult, Routable,
    macros::{buffer, command, r_println, renderer},
    res::ResExitCode,
};

use crate::Next;
use crate::git::{LOCK_FILE, TEMP_COMMIT_MARK, head_message, run_git, worktree_clean};
use crate::res::{CargoError, MessagePrinter};

/// Undoes a CI temporary commit created by [`crate::cmd::cmd_git_lock`].
///
/// Only acts when the HEAD commit message contains `CI TEMP` (case-sensitive),
/// which together with the `MINGLING-CI-CHECKING` marker means the workspace
/// is in a CI phase and all uncommitted state may be discarded. Restores the
/// tree in five steps: unstage, restore tracked files, delete untracked files,
/// roll back the temporary commit, and remove the marker file.
///
/// When the working tree is dirty (e.g. CI left tracked changes behind) the
/// restore still runs, but the command reports a non-zero exit code so the
/// caller knows the CI phase contaminated the repository.
#[command(node = "git-unlock")]
pub fn git_unlock() -> Next {
    let head = head_message().unwrap_or_default();
    if !head.contains(TEMP_COMMIT_MARK) {
        return ErrorGitUnlock(format!("HEAD is not a CI temporary commit: `{head}`")).to_chain();
    }

    // Record dirtiness before restoring: the restore discards those changes.
    let dirty = !worktree_clean();

    if let Err(e) = undo_ci_phase() {
        return ErrorGitUnlock(e).to_chain();
    }

    ResultGitUnlock { dirty }.to_chain()
}

/// The five-step restoration sequence of `git-unlock`.
fn undo_ci_phase() -> Result<(), String> {
    run_git(["reset"])?;
    run_git(["restore", "."])?;
    run_git(["clean", "-f", "-d"])?;
    run_git(["reset", "--hard", "HEAD~1"])?;
    std::fs::remove_file(LOCK_FILE).ok();
    Ok(())
}

/// Whether the working tree was dirty when the unlock started.
#[derive(Grouped)]
pub struct ResultGitUnlock {
    dirty: bool,
}

#[derive(Grouped, Default)]
pub struct ErrorGitUnlock(pub String);

#[renderer(buffer)]
pub fn render_git_unlock(r: ResultGitUnlock, exit_code: &mut ResExitCode) {
    if r.dirty {
        r_println!("Unlocked: workspace restored (working tree was dirty)");
        exit_code.exit_code = 1;
    } else {
        r_println!("Unlocked: workspace restored");
    }
}

#[renderer]
pub fn render_error_git_unlock(
    e: ErrorGitUnlock,
    error: &CargoError,
    exit_code: &mut ResExitCode,
) -> RenderResult {
    let render_result = RenderResult::new();
    error.println(vec![format!("Git-Unlock: {}", e.0)]);
    exit_code.exit_code = 1;
    render_result
}
