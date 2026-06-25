use crate::ThisProgram;
use mingling::{
    Program,
    macros::{dispatcher, program_setup},
};

mod generator;
pub use generator::*;

pub mod metadata;

mod checklist_reader;
pub use checklist_reader::*;

mod show_binaries;
pub use show_binaries::*;

mod show_directories;
pub use show_directories::*;

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
