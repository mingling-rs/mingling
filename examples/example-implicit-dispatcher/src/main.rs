//! Example Implicit Dispatcher
//!
//! > This example demonstrates how to use the implicit `dispatcher!` definition syntax enabled by `extras`

use mingling::prelude::*;

// When using implicit syntax, the entry and dispatcher names will be automatically derived
dispatcher!("remote.add" /*, CMDRemoteAdd    => EntryRemoteAdd */);
dispatcher!("remote.remove", CMDRemoteRemove => EntryRemoteRemove);

fn main() {
    let mut program = ThisProgram::new();

    // --------- IMPORTANT ---------
    program.with_dispatcher(CMDRemoteAdd);
    //                      ^^^^^^^^^^^^\_ CMDRemoteAdd is implicitly created
    // --------- IMPORTANT ---------

    program.with_dispatcher(CMDRemoteRemove);
    program.exec_and_exit();
}

gen_program!();
