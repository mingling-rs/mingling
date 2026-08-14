use mingling_core::{Program, ProgramCollect, setup::ProgramSetup};

use crate::res::OSC94;

/// `OSC 9;4` Setup for managing terminal progress notification state
///
/// This Setup manages the terminal's `OSC 9;4` protocol support state within the
/// program's resource store. It registers an [`OSC94`] resource that tracks whether
/// the current terminal supports the protocol, and provides a helper resource that
/// can be used to send progress notification messages.
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
/// use mingling::setup::OSC94Setup;
///
/// let mut program = Program::<ThisProgram>::new();
/// program.with_setup(OSC94Setup);
/// ```
///
/// # Behavior
///
/// - Registers an [`OSC94`] resource that tracks whether the current terminal
///   supports the `OSC 9;4` protocol.
/// - The support check inspects various environment variables such as `TERM_PROGRAM`,
///   `WT_SESSION`, `VTE_VERSION`, and `TERM`.
///
/// # Notes
///
/// - The support state is determined at setup time and stored in the resource store.
/// - Use [`OSC94Message`] to construct and send progress notification messages.
pub struct OSC94Setup;

impl<C> ProgramSetup<C> for OSC94Setup
where
    C: ProgramCollect<Enum = C> + 'static,
{
    fn setup(self, program: &mut Program<C>) {
        program.with_resource(OSC94 {
            is_support: is_support_osc94(),
        });
    }
}

/// Check whether the current terminal environment supports the `OSC 9;4` protocol
///
/// This function inspects various environment variables to determine whether the
/// current terminal supports Microsoft's
/// [OSC 9;4 protocol](https://learn.microsoft.com/en-us/windows/terminal/tutorials/progress-bar-sequences),
/// which allows sending task progress notifications via ANSI escape sequences.
///
/// Supported terminal environments include:
/// - **`TERM_PROGRAM`**: `ghostty`, `WezTerm`, `iTerm.app`
/// - **`WT_SESSION`**: Windows Terminal
/// - **`VTE_VERSION`**: VTE-based terminals (such as GNOME Terminal, Konsole, etc.)
/// - **`TERM`**: terminal emulators containing `xterm`
///
/// Returns `true` if the current terminal supports the `OSC 9;4` protocol, so that
/// progress notification escape sequences can be safely sent.
fn is_support_osc94() -> bool {
    if let Ok(program) = std::env::var("TERM_PROGRAM") {
        match program.as_str() {
            "ghostty" | "WezTerm" | "iTerm.app" => return true,
            _ => {}
        }
    }

    if std::env::var("WT_SESSION").is_ok() {
        return true;
    }

    if std::env::var("VTE_VERSION").is_ok() {
        return true;
    }

    if let Ok(term) = std::env::var("TERM")
        && term.contains("xterm")
    {
        return true;
    }

    false
}
