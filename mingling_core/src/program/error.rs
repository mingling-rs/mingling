use std::any::Any;
use std::fmt;

/// Error type returned when a panic occurs during execution.
pub struct ProgramPanic {
    pub payload: Box<dyn Any + Send>,
}

impl fmt::Display for ProgramPanic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(s) = self.payload.downcast_ref::<&str>() {
            write!(f, "{s}")
        } else if let Some(s) = self.payload.downcast_ref::<String>() {
            write!(f, "{s}")
        } else {
            write!(f, "")
        }
    }
}

impl ProgramPanic {
    #[must_use]
    pub fn new(payload: Box<dyn Any + Send>) -> Self {
        Self { payload }
    }
}

impl fmt::Debug for ProgramPanic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.payload)
    }
}

#[cfg(test)]
mod tests {
    use crate::error::ChainProcessError;
    use crate::error::ProgramPanic;
    use crate::program::exec::error::ProgramExecuteError;
    use crate::program::exec::error::ProgramInternalExecuteError;

    // ProgramPanic

    #[test]
    fn test_program_panic_new_with_str() {
        let panic = ProgramPanic::new(Box::new("hello world"));
        assert_eq!(format!("{panic}"), "hello world");
    }

    #[test]
    fn test_program_panic_new_with_string() {
        let panic = ProgramPanic::new(Box::new("owned string".to_string()));
        assert_eq!(format!("{panic}"), "owned string");
    }

    #[test]
    fn test_program_panic_new_with_i32() {
        let panic = ProgramPanic::new(Box::new(42));
        assert_eq!(format!("{panic}"), "");
    }

    #[test]
    fn test_program_panic_debug() {
        let panic = ProgramPanic::new(Box::new("debug me"));
        let debug = format!("{panic:?}");
        assert_eq!(debug, "Any { .. }");
    }

    // ProgramExecuteError

    #[test]
    fn test_program_execute_error_display_dispatcher_not_found() {
        let err = ProgramExecuteError::DispatcherNotFound;
        assert_eq!(format!("{err}"), "No Dispatcher Found");
    }

    #[test]
    fn test_program_execute_error_display_renderer_not_found() {
        let err = ProgramExecuteError::RendererNotFound("markdown".into());
        assert_eq!(format!("{err}"), "No Renderer (`markdown`) Found");
    }

    #[test]
    fn test_program_execute_error_display_panic() {
        let p = ProgramPanic::new(Box::new("oops"));
        let err = ProgramExecuteError::Panic(p);
        let display = format!("{err}");
        assert!(display.starts_with("Panic: "));
    }

    #[test]
    fn test_program_execute_error_display_other() {
        let err = ProgramExecuteError::Other("something went wrong".into());
        assert_eq!(format!("{err}"), "Other error: something went wrong");
    }

    #[test]
    fn test_program_execute_error_error_trait_no_source() {
        use std::error::Error;
        let err = ProgramExecuteError::Other("msg".into());
        assert!(err.source().is_none());

        let err = ProgramExecuteError::DispatcherNotFound;
        assert!(err.source().is_none());

        let err = ProgramExecuteError::RendererNotFound("json".into());
        assert!(err.source().is_none());

        let panic = ProgramPanic::new(Box::new("panic"));
        let err = ProgramExecuteError::Panic(panic);
        assert!(err.source().is_none());
    }

    #[test]
    fn test_from_program_panic_into_program_execute_error() {
        let panic = ProgramPanic::new(Box::new("from panic"));
        let err: ProgramExecuteError = panic.into();
        assert!(matches!(err, ProgramExecuteError::Panic(_)));
    }

    // ProgramInternalExecuteError

    #[test]
    fn test_program_internal_execute_error_display_dispatcher_not_found() {
        let err = ProgramInternalExecuteError::DispatcherNotFound;
        assert_eq!(format!("{err}"), "No Dispatcher Found");
    }

    #[test]
    fn test_program_internal_execute_error_display_renderer_not_found() {
        let err = ProgramInternalExecuteError::RendererNotFound("html".into());
        assert_eq!(format!("{err}"), "No Renderer (`html`) Found");
    }

    #[test]
    fn test_program_internal_execute_error_display_other() {
        let err = ProgramInternalExecuteError::Other("internal issue".into());
        assert_eq!(format!("{err}"), "Other error: internal issue");
    }

    #[test]
    fn test_program_internal_execute_error_display_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let err = ProgramInternalExecuteError::IO(io_err);
        let display = format!("{err}");
        assert!(display.contains("IO error"));
        assert!(display.contains("file missing"));
    }

    #[test]
    fn test_program_internal_execute_error_display_repl_panic() {
        let p = ProgramPanic::new(Box::new("repl crash"));
        let err = ProgramInternalExecuteError::REPLPanic(p);
        let display = format!("{err}");
        assert!(display.starts_with("A single REPL execution failed: "));
        assert!(display.contains("repl crash"));
    }

    #[test]
    fn test_program_internal_execute_error_error_trait_source() {
        use std::error::Error;

        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let err = ProgramInternalExecuteError::IO(io_err);
        assert!(err.source().is_some());

        let err = ProgramInternalExecuteError::DispatcherNotFound;
        assert!(err.source().is_none());

        let err = ProgramInternalExecuteError::RendererNotFound("txt".into());
        assert!(err.source().is_none());

        let err = ProgramInternalExecuteError::Other("msg".into());
        assert!(err.source().is_none());

        let p = ProgramPanic::new(Box::new("msg"));
        let err = ProgramInternalExecuteError::REPLPanic(p);
        assert!(err.source().is_none());
    }

    #[test]
    fn test_from_io_error_into_program_internal_execute_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::TimedOut, "timeout");
        let err: ProgramInternalExecuteError = io_err.into();
        assert!(matches!(err, ProgramInternalExecuteError::IO(_)));
    }

    // From<ProgramInternalExecuteError> for ProgramExecuteError

    #[test]
    fn test_from_internal_to_execute_dispatcher_not_found() {
        let internal = ProgramInternalExecuteError::DispatcherNotFound;
        let err: ProgramExecuteError = internal.into();
        assert!(matches!(err, ProgramExecuteError::DispatcherNotFound));
    }

    #[test]
    fn test_from_internal_to_execute_renderer_not_found() {
        let internal = ProgramInternalExecuteError::RendererNotFound("yaml".into());
        let err: ProgramExecuteError = internal.into();
        assert!(matches!(err, ProgramExecuteError::RendererNotFound(_)));
        assert_eq!(format!("{err}"), "No Renderer (`yaml`) Found");
    }

    #[test]
    fn test_from_internal_to_execute_other() {
        let internal = ProgramInternalExecuteError::Other("custom".into());
        let err: ProgramExecuteError = internal.into();
        assert!(matches!(err, ProgramExecuteError::Other(_)));
        assert_eq!(format!("{err}"), "Other error: custom");
    }

    #[test]
    fn test_from_internal_to_execute_io_becomes_other() {
        let io_err = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "refused");
        let internal = ProgramInternalExecuteError::IO(io_err);
        let err: ProgramExecuteError = internal.into();
        assert!(matches!(err, ProgramExecuteError::Other(_)));
        let display = format!("{err}");
        assert!(display.starts_with("Other error: "));
        assert!(display.contains("refused"));
    }

    #[test]
    fn test_from_internal_to_execute_repl_panic_becomes_other() {
        let p = ProgramPanic::new(Box::new("panic in repl"));
        let internal = ProgramInternalExecuteError::REPLPanic(p);
        let err: ProgramExecuteError = internal.into();
        assert!(matches!(err, ProgramExecuteError::Other(_)));
        let display = format!("{err}");
        assert!(display.contains("A single REPL execution failed"));
        assert!(display.contains("panic in repl"));
    }

    // From<ChainProcessError> for ProgramInternalExecuteError

    #[test]
    fn test_from_chain_process_error_other_into_internal() {
        let chain_err = ChainProcessError::Other("chain other".into());
        let err: ProgramInternalExecuteError = chain_err.into();
        assert!(matches!(err, ProgramInternalExecuteError::Other(_)));
        assert_eq!(format!("{err}"), "Other error: chain other");
    }

    #[test]
    fn test_from_chain_process_error_io_into_internal() {
        let io_err = std::io::Error::new(std::io::ErrorKind::BrokenPipe, "broken");
        let chain_err = ChainProcessError::IO(io_err);
        let err: ProgramInternalExecuteError = chain_err.into();
        assert!(matches!(err, ProgramInternalExecuteError::IO(_)));
        let display = format!("{err}");
        assert!(display.contains("IO error"));
        assert!(display.contains("broken"));
    }
}
