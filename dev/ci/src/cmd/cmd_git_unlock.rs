use mingling::{
    Grouped, RenderResult, Routable,
    macros::{arg, buffer, command, r_println, renderer},
    picker::{EntryPicker, value::Flag},
    res::ResExitCode,
};

use crate::git::{LOCK_FILE, TEMP_COMMIT_MARK, head_message, run_git, worktree_clean};
use crate::res::{CargoError, MessagePrinter};
use crate::{Entry, Next};

/// Undoes a CI temporary commit created by [`crate::cmd::cmd_git_lock`].
///
/// Only acts when the HEAD commit message contains `CI TEMP` (case-sensitive).
/// The restore path is picked by the marker file content:
///
/// - `true`: a base `TEMP` commit with the dirty changes sits below; restore
///   by hard-resetting past the marker commit, then soft-resetting and
///   unstaging to put the user's changes back into the working tree.
/// - `false`: the tree was clean; a single hard reset back to the original
///   HEAD is enough.
///
/// When the working tree is dirty (e.g. CI left tracked changes behind) the
/// restore still runs, but the command reports a non-zero exit code so the
/// caller knows the CI phase contaminated the repository. With `--show-diff`
/// the diff of those changes is printed before they are discarded.
#[command(node = "git-unlock")]
// `#[command]` rewrites an owned first param into the entry type, so the args
// must be passed by value even though the body only reads them.
#[allow(clippy::needless_pass_by_value)]
pub fn git_unlock(args: Entry) -> Next {
    let head = head_message().unwrap_or_default();
    if !head.contains(TEMP_COMMIT_MARK) {
        return ErrorGitUnlock(format!("HEAD is not a CI temporary commit: `{head}`")).to_chain();
    }

    // Record dirtiness before restoring: the restore discards those changes.
    let dirty = !worktree_clean();

    // The marker file lives in the HEAD (CI TEMP) commit, so it is readable
    // from the working tree; a missing marker falls back to the clean path.
    let based_on_dirty =
        std::fs::read_to_string(LOCK_FILE).is_ok_and(|content| content.trim() == "true");

    if dirty && *args.pick(&arg![show_diff: Flag]).unwrap() {
        show_diff();
    }

    if let Err(e) = undo_ci_phase(based_on_dirty) {
        return ErrorGitUnlock(e).to_chain();
    }

    ResultGitUnlock { dirty }.to_chain()
}

/// Prints the tracked changes the CI run left behind, before the restore
/// discards them. Untracked files are not shown (they are removed by clean).
fn show_diff() {
    let Ok(diff) = run_git(["diff", "HEAD"]) else {
        return;
    };
    if diff.is_empty() {
        return;
    }
    println!("{diff}");
}

/// Restores the workspace, keeping the user's pre-lock changes.
///
/// With a base `TEMP` commit (`true`) the marker commit is dropped by a hard
/// reset to `HEAD~1`, the `TEMP` commit is unwrapped into the staging area by
/// a soft reset, and a plain reset unstages it back into the working tree.
/// Without one (`false`) a single hard reset to `HEAD~1` removes the marker
/// commit and lands on the original HEAD.
fn undo_ci_phase(based_on_dirty: bool) -> Result<(), String> {
    run_git(["reset", "--hard", "HEAD~1"])?;
    if based_on_dirty {
        // Unwrap the `TEMP` commit into the staging area, then unstage it
        // back into the working tree.
        run_git(["reset", "--soft", "HEAD~1"])?;
        run_git(["reset"])?;
    }
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
