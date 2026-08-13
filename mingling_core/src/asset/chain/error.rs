use crate::error::ProgramInternalExecuteError;

/// Represents error types that occur in a chained processing pipeline.
///
/// This enum is used to uniformly encapsulate various exceptions that may
/// occur during the execution of an entire chain, including IO errors and
/// other custom error messages.
#[derive(Debug)]
pub enum ChainProcessError {
    /// Other unclassified generic errors, stored as a string description.
    Other(String),
    /// Errors resulting from a failed IO operation, holding the standard library's [`std::io::Error`]
    IO(std::io::Error),
}

impl std::fmt::Display for ChainProcessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Other(s) => write!(f, "Other error: {s}"),
            Self::IO(e) => write!(f, "IO error: {e}"),
        }
    }
}

impl std::error::Error for ChainProcessError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IO(e) => Some(e),
            Self::Other(_) => None,
        }
    }
}

impl From<std::io::Error> for ChainProcessError {
    fn from(e: std::io::Error) -> Self {
        Self::IO(e)
    }
}

impl From<ProgramInternalExecuteError> for ChainProcessError {
    fn from(value: ProgramInternalExecuteError) -> Self {
        match value {
            ProgramInternalExecuteError::DispatcherNotFound => {
                Self::Other("DispatcherNotFound".into())
            }
            ProgramInternalExecuteError::RendererNotFound(r) => {
                Self::Other(format!("RendererNotFound: {r}"))
            }
            ProgramInternalExecuteError::Other(e) => Self::Other(e),
            ProgramInternalExecuteError::IO(e) => Self::Other(format!("IOError: {e:?}")),
            ProgramInternalExecuteError::REPLPanic(program_panic) => {
                Self::Other(format!("REPLPanic: {program_panic}"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{ProgramInternalExecuteError, ProgramPanic};
    use std::error::Error;

    #[test]
    fn test_chain_process_error_display_other() {
        let err = ChainProcessError::Other("something went wrong".into());
        assert_eq!(format!("{err}"), "Other error: something went wrong");
    }

    #[test]
    fn test_chain_process_error_display_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = ChainProcessError::IO(io_err);
        let display = format!("{err}");
        assert!(display.contains("IO error"));
        assert!(display.contains("file not found"));
    }

    #[test]
    fn test_chain_process_error_source_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let err = ChainProcessError::IO(io_err);
        assert!(err.source().is_some());
    }

    #[test]
    fn test_chain_process_error_source_other() {
        let err = ChainProcessError::Other("msg".into());
        assert!(err.source().is_none());
    }

    #[test]
    fn test_from_io_error_into_chain_process_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::TimedOut, "timeout");
        let err: ChainProcessError = io_err.into();
        assert!(matches!(err, ChainProcessError::IO(_)));
    }

    #[test]
    fn test_from_program_internal_execute_error_dispatcher_not_found() {
        let internal = ProgramInternalExecuteError::DispatcherNotFound;
        let err: ChainProcessError = internal.into();
        assert!(matches!(err, ChainProcessError::Other(_)));
        assert_eq!(format!("{err}"), "Other error: DispatcherNotFound");
    }

    #[test]
    fn test_from_program_internal_execute_error_renderer_not_found() {
        let internal = ProgramInternalExecuteError::RendererNotFound("json".into());
        let err: ChainProcessError = internal.into();
        assert_eq!(format!("{err}"), "Other error: RendererNotFound: json");
    }

    #[test]
    fn test_from_program_internal_execute_error_other() {
        let internal = ProgramInternalExecuteError::Other("custom error".into());
        let err: ChainProcessError = internal.into();
        assert_eq!(format!("{err}"), "Other error: custom error");
    }

    #[test]
    fn test_from_program_internal_execute_error_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "refused");
        let internal = ProgramInternalExecuteError::IO(io_err);
        let err: ChainProcessError = internal.into();
        let display = format!("{err}");
        assert!(display.contains("IOError"));
    }

    #[test]
    fn test_from_program_internal_execute_error_repl_panic() {
        let panic_payload = ProgramPanic {
            payload: Box::new("repl crash"),
        };
        let internal = ProgramInternalExecuteError::REPLPanic(panic_payload);
        let err: ChainProcessError = internal.into();
        assert!(format!("{err}").contains("REPLPanic"));
    }
}
