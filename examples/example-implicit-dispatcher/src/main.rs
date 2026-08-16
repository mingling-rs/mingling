//! Example Implicit Dispatcher
//!
//! > This example demonstrates how to use the implicit `dispatcher!` definition syntax enabled by `extras`

use mingling::prelude::*;

// When using implicit syntax, the entry name will be automatically derived
// from the command name (the dispatcher struct is generated internally)
dispatcher!("remote.add" /* => EntryRemoteAdd */);
dispatcher!("remote.remove", EntryRemoteRemove);

fn main() {
    ThisProgram::new().exec_and_exit();
}

gen_program!();
