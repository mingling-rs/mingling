// Doc Not Optimize
use crate::error::{ChainProcessError, ProgramPanic};
use std::fmt;

/// Errors that can occur during program execution.
///
/// This enum represents the various error conditions that may arise
/// when executing a program, including missing dispatchers/renderers,
/// panics, and other miscellaneous errors.
#[derive(Debug)]
pub enum ProgramExecuteError {
    /// No dispatcher was found to handle the program execution.
    DispatcherNotFound,

    /// No renderer was found for the given name.
    RendererNotFound(String),

    /// The program encountered a panic during execution.
    Panic(ProgramPanic),

    /// An other error occurred.
    Other(String),
}

impl fmt::Display for ProgramExecuteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DispatcherNotFound => write!(f, "No Dispatcher Found"),
            Self::RendererNotFound(s) => {
                write!(f, "No Renderer (`{s}`) Found")
            }
            Self::Panic(p) => write!(f, "Panic: {p:?}"),
            Self::Other(s) => write!(f, "Other error: {s}"),
        }
    }
}

impl std::error::Error for ProgramExecuteError {}

impl From<ProgramPanic> for ProgramExecuteError {
    fn from(value: ProgramPanic) -> Self {
        Self::Panic(value)
    }
}

/// Errors that can occur during internal program execution.
///
/// This enum represents error conditions that arise specifically within
/// the internal execution pipeline of a program, including missing
/// dispatchers/renderers, I/O errors, and other miscellaneous failures.
/// These errors are typically not exposed directly to the end user but
/// are used internally and can be converted into [`ProgramExecuteError`].
#[derive(Debug)]
pub enum ProgramInternalExecuteError {
    /// No dispatcher was found to handle the program execution.
    DispatcherNotFound,

    /// No renderer was found for the given name.
    RendererNotFound(String),

    /// An other internal error occurred.
    Other(String),

    /// A single REPL execution failed
    REPLPanic(ProgramPanic),

    /// An I/O error occurred during execution.
    IO(std::io::Error),
}

impl fmt::Display for ProgramInternalExecuteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DispatcherNotFound => write!(f, "No Dispatcher Found"),
            Self::RendererNotFound(s) => write!(f, "No Renderer (`{s}`) Found"),
            Self::Other(s) => write!(f, "Other error: {s}"),
            Self::IO(e) => write!(f, "IO error: {e}"),
            Self::REPLPanic(panic) => write!(f, "A single REPL execution failed: {panic}"),
        }
    }
}

impl std::error::Error for ProgramInternalExecuteError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IO(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for ProgramInternalExecuteError {
    fn from(e: std::io::Error) -> Self {
        Self::IO(e)
    }
}

impl From<ProgramInternalExecuteError> for ProgramExecuteError {
    fn from(value: ProgramInternalExecuteError) -> Self {
        match value {
            ProgramInternalExecuteError::DispatcherNotFound => Self::DispatcherNotFound,
            ProgramInternalExecuteError::RendererNotFound(s) => Self::RendererNotFound(s),
            ProgramInternalExecuteError::Other(s) => Self::Other(s),
            ProgramInternalExecuteError::IO(e) => Self::Other(format!("{e}")),
            ProgramInternalExecuteError::REPLPanic(p) => {
                Self::Other(format!("A single REPL execution failed: {p}"))
            }
        }
    }
}

impl From<ChainProcessError> for ProgramInternalExecuteError {
    fn from(value: ChainProcessError) -> Self {
        match value {
            ChainProcessError::Other(s) => Self::Other(s),
            ChainProcessError::IO(error) => Self::IO(error),
        }
    }
}
