use std::io::{IsTerminal, Read};

use mingling_core::{
    Program, ProgramCollect, hook::ProgramHook, setup::ProgramSetup, utils::ArgumentSplitter,
};

/// Uses the standard input as arguments for the program
///
/// This Setup can take standard input supplied via a pipe or redirect,
/// split it according to whitespace and quoting rules, and append
/// the resulting arguments to the end of the command argument list.
///
/// # Usage
///
/// This Setup can be registered using the
/// [`Program`](https://docs.rs/mingling/latest/mingling/struct.Program.html)
/// `with_setup` method, for example:
///
/// ```rust
/// # use mingling::MockProgramCollect as ThisProgram;
/// use mingling::Program;
/// use mingling::setup::StandardInputArgsSetup;
///
/// let mut program = Program::<ThisProgram>::new();
/// program.with_setup(StandardInputArgsSetup);
/// ```
///
/// # Behavior
///
/// - Standard input is only read when it is not a terminal (i.e., when
///   there is piped or redirected input).
/// - The read content is split into multiple arguments according to
///   whitespace and quoting rules.
/// - If the standard input content is empty, no arguments are produced.
/// - All input is converted to UTF-8 encoding (lossy conversion is used
///   when strict parsing is not possible).
///
/// # Notes
///
/// - This Setup applies uniformly to all subcommands of the entire program
///   and does not provide fine-grained control. If you need different
///   standard input behavior across different subcommands (e.g., some
///   subcommands read stdin while others ignore it), **do not use this Setup**.
/// - This Setup does **not** provide any validation rules. Content provided
///   via standard input is treated as trusted arguments and appended directly.
///   As a result, the input source can also inject arbitrary arguments into
///   the command, so you should be careful when processing untrusted input.
pub struct StandardInputArgsSetup;

impl<C> ProgramSetup<C> for StandardInputArgsSetup
where
    C: ProgramCollect<Enum = C>,
{
    fn setup(self, program: &mut Program<C>) {
        program.with_hook(ProgramHook::empty().on_pre_dispatch(|ctx| {
            let pipe_input = read_stdin();
            if let Some(pipe_input) = pipe_input {
                ctx.arguments.append(&mut pipe_input.trim().split_args());
            }
        }));
    }
}

fn read_stdin() -> Option<String> {
    // Check if stdin is a terminal (no piped input) or has data available
    if std::io::stdin().is_terminal() {
        return None;
    }

    let mut bytes = Vec::new();
    match std::io::stdin().read_to_end(&mut bytes) {
        Ok(_) => {
            if bytes.is_empty() {
                return None;
            }
            // Handle encoding differences, ensure output is always UTF-8.
            // First try strict UTF-8 parsing; fall back to lossy conversion
            Some(String::from_utf8_lossy(&bytes).into_owned())
        }
        Err(_) => None,
    }
}
