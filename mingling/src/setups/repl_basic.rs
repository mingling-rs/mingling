// Doc Not Optimize
use std::io::Write;

use mingling_core::{Program, ProgramCollect, hook::ProgramHook, setup::ProgramSetup};

/// Provides basic Readline capability for the REPL.
pub struct BasicREPLReadlineSetup;
impl<C> ProgramSetup<C> for BasicREPLReadlineSetup
where
    C: ProgramCollect<Enum = C>,
{
    fn setup(self, program: &mut Program<C>) {
        program.with_hook(ProgramHook::empty().on_repl_readline(|_| readline().ok()));
    }
}

/// A basic REPL prompt that displays a prompt string and reads input from the user.
///
/// **Note:** This setup uses static [`OnceLock`](std::sync::OnceLock) internally,
/// meaning only the last configured instance will take effect globally.
/// Do not configure multiple prompts with different values — only one will be used.
pub enum BasicREPLPromptSetup {
    /// A static prompt string that is displayed before each REPL input.
    Prompt(String),
    /// A function that returns a dynamic prompt string each time the REPL reads input.
    Func(fn() -> String),
}

impl BasicREPLPromptSetup {
    /// Creates a new [`BasicREPLPromptSetup`] with the given prompt string.
    pub fn simple(prompt: impl Into<String>) -> Self {
        Self::Prompt(prompt.into())
    }

    /// Creates a new [`BasicREPLPromptSetup`] with the given function.
    pub fn func(func: fn() -> String) -> Self {
        Self::Func(func)
    }
}

impl<C> ProgramSetup<C> for BasicREPLPromptSetup
where
    C: ProgramCollect<Enum = C>,
{
    fn setup(self, program: &mut Program<C>) {
        match self {
            Self::Prompt(prompt) => {
                static PROMPT: std::sync::OnceLock<String> = std::sync::OnceLock::new();
                let _ = PROMPT.set(prompt);
                program.with_hook(ProgramHook::empty().on_repl_pre_readline(|_| {
                    print!("{}", PROMPT.get().unwrap());
                    let _ = std::io::stdout().flush();
                }));
            }
            Self::Func(f) => {
                static FUNC: std::sync::OnceLock<fn() -> String> = std::sync::OnceLock::new();
                let _ = FUNC.set(f);
                program.with_hook(ProgramHook::empty().on_repl_pre_readline(|_| {
                    print!("{}", FUNC.get().unwrap()());
                    let _ = std::io::stdout().flush();
                }));
            }
        }
    }
}

/// Prints the result of each REPL command to stdout.
pub struct BasicREPLOutputSetup;
impl<C> ProgramSetup<C> for BasicREPLOutputSetup
where
    C: ProgramCollect<Enum = C>,
{
    fn setup(self, program: &mut Program<C>) {
        program.with_hook(ProgramHook::empty().on_repl_receive_result(|r| {
            if !r.result.is_empty() {
                println!("{}", r.result);
            }
        }));
    }
}

fn readline() -> Result<String, std::io::Error> {
    let mut input = String::new();
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}
