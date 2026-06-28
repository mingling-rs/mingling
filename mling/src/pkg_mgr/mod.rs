use crate::ThisProgram;
use mingling::{
    Program,
    macros::{dispatcher, program_setup},
};

pub mod installer;

dispatcher!("install");
dispatcher!("ls.namespace", CMDListNamespace => EntryListNamespace);
dispatcher!("rm.namespace", CMDRemoveNamespace => EntryRemoveNamespace);

#[program_setup]
pub fn package_manager_setup(p: &mut Program<ThisProgram>) {
    p.with_dispatcher(CMDInstall);
    p.with_dispatcher(CMDListNamespace);
    p.with_dispatcher(CMDRemoveNamespace);
}
