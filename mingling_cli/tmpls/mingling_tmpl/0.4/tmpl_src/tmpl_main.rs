pub(crate) mod commands;
pub(crate) mod helps;
pub(crate) mod renderers;

use mingling::macros::buffer;
use mingling::prelude::*;
??? >>> EXIT_CODE
use mingling::setup::ExitCodeSetup;
??? <<<
use mingling::setup::picker::BasicProgramSetup;

fn main() {
    let mut program = ThisProgram::new();

    // Plugins
    program.with_setup(BasicProgramSetup);
??? >>> EXIT_CODE
    program.with_setup(ExitCodeSetup::default());
??? <<<
??? >>> COMPLETION

    // Completion Dispatcher
    program.with_dispatcher(CMDCompletion);
??? <<<
??? >>> NOT_DISPATCH_TREE

    // Dispatchers
    program.with_dispatcher(commands::greet::CMDGreet);
??? <<<

    program.exec_and_exit();
}

#[renderer(buffer)]
fn handle_error_dispatcher_not_found(prev: ErrorDispatcherNotFound) {
    r_println!("Error: cannot match \"{}\" to any command", prev.join(" "));
}

#[renderer(buffer)]
fn handle_error_renderer_not_found(prev: ErrorRendererNotFound) {
    let type_name = prev.inner;
    r_println!("Error: renderer not found for \"{}\"", type_name);
}

gen_program!();
