use std::path::{self, PathBuf};

use mingling::{Groupped, macros::{chain, pack, r_println, renderer, route}};

use crate::{EntryGenerateProject, Next, res::ResCurrentDir};

pack!(StateGenerateProjectReady = PathBuf);
pack!(ResultGenerateProjectChecklistCreated = PathBuf);

pack!(StateGenerateProjectExecBegin = PathBuf);
pack!(StateGenerateProjectExecuting = ());

const CHECK_LIST_NAME: &str = "CHECKLIST.md";
const CHECK_LIST_CONTENT: &str = include_str!("../../res/CHECKLIST.md");

#[chain]
pub fn handle_generate(_args: EntryGenerateProject, cwd: &ResCurrentDir) -> Next {
    let checklist_path = cwd.path.join(CHECK_LIST_NAME);

    if !checklist_path.exists() {
        StateGenerateProjectReady::new(checklist_path).to_chain()
    } else {
        StateGenerateProjectExecBegin::new(checklist_path).to_chain()
    }
}

#[chain]
pub fn handle_state_gen_proj_ready(prev: StateGenerateProjectReady) -> Next {
    let path = prev.inner;
    route!(std::fs::write(&path, CHECK_LIST_CONTENT));
    ResultGenerateProjectChecklistCreated::new(path).to_render()
}

#[renderer]
pub fn render_gen_proj_checklist_created(result: ResultGenerateProjectChecklistCreated) {
    r_println!("Successfully create {} at \"{}\"", CHECK_LIST_NAME, result.to_string_lossy());
    r_println!("");
    r_println!("Please fill in {CHECK_LIST_NAME} and run `mling gen` again to continue generating");
}
