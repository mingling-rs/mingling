use crate::ThisProgram;
use mingling::{
    Program,
    macros::{dispatcher, program_setup},
};

pub mod checklist_reader;
pub mod generator;
pub mod metadata;
pub mod show_binaries;
pub mod show_directories;

dispatcher!("gen", CMDGenerateProject => EntryGenerateProject);

dispatcher!("show.binaries");
dispatcher!("show.workspace-dir",
    CMDShowWorkspaceDirectory => EntryShowWorkspaceDirectory
);
dispatcher!("show.target-dir",
    CMDShowTargetDirectories => EntryShowTargetDirectories
);

#[program_setup]
pub fn project_manager_setup(p: &mut Program<ThisProgram>) {
    p.with_dispatcher(CMDGenerateProject);

    p.with_dispatcher(CMDShowBinaries);
    p.with_dispatcher(CMDShowWorkspaceDirectory);
    p.with_dispatcher(CMDShowTargetDirectories);
}
