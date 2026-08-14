mod state;
pub use state::*;

/// Process `OSC 9;4` status.
///
/// Provides support for the `OSC 9;4` protocol. You can inject it into the execution flow
/// through Mingling's resource injection system, and use it to control your process state.
///
/// Typically, `OSC94` is registered via [`OSC94Setup`], and then injected into functions
/// through Mingling's resource injection system.
///
/// # Registration
///
/// Before use, the [`OSC94Setup`] must be registered with the program:
///
/// ```
/// # use mingling::MockProgramCollect as ThisProgram;
/// use mingling::setup::OSC94Setup;
/// use mingling::Program;
///
/// let mut program = Program::<ThisProgram>::new();
/// program.with_setup(OSC94Setup);
/// ```
///
/// # Example
///
/// ```
/// use mingling::res::{OSC94, OSC94State};
///
/// let osc94 = OSC94::default();
/// let mut guard = osc94.get_mut();
///
/// guard.set_progress(0.5);
/// assert_eq!(guard.state(), OSC94State::Normal(0.5));
/// ```
#[derive(Debug, Default, Clone, Copy)]
pub struct OSC94 {
    pub(crate) is_support: bool,
}

impl OSC94 {
    /// Get a guard for modifying progress.
    ///
    /// The returned [`OSC94Guard`] allows you to set the process state and progress.
    /// If the current environment supports the `OSC 9;4` protocol, state changes will
    /// be sent to the terminal in real time.
    ///
    /// # Returns
    ///
    /// Returns an [`OSC94Guard`] with an initial state of [`OSC94State::Clean`].
    ///
    /// # Example
    ///
    /// ```
    /// use mingling::res::{OSC94, OSC94State};
    ///
    /// let osc94 = OSC94::default();
    /// let guard = osc94.get_mut();
    /// assert_eq!(guard.state(), OSC94State::Clean);
    /// ```
    #[must_use]
    pub const fn get_mut(&self) -> OSC94Guard {
        OSC94Guard {
            is_support: self.is_support,
            msg: OSC94State::Clean,
        }
    }
}

/// A guard for modifying process state.
///
/// Obtained via [`OSC94::get_mut`]. When the guard is dropped, the process state is
/// automatically restored to [`OS94State::Clean`], so no manual cleanup is needed.
///
/// # Example
///
/// Create a guard via [`OSC94`], and the state is automatically restored to Clean
/// when the guard is dropped:
///
/// ```
/// use mingling::res::OSC94;
///
/// let osc94 = OSC94::default();
/// {
///     let mut guard = osc94.get_mut();
///     guard.set_progress(0.5);
///     // When leaving this scope, the guard is dropped and the process state is automatically restored to Clean
/// }
/// ```
pub struct OSC94Guard {
    pub(crate) is_support: bool,
    msg: OSC94State,
}

impl OSC94Guard {
    /// Set the process state to Clean.
    ///
    /// Indicates that the process has finished or is in a normal, problem-free state.
    ///
    /// # Example
    ///
    /// ```
    /// use mingling::res::{OSC94, OSC94State};
    ///
    /// let osc94 = OSC94::default();
    /// let mut guard = osc94.get_mut();
    /// guard.set_progress(0.5);
    /// guard.set_clean_state();
    /// assert_eq!(guard.state(), OSC94State::Clean);
    /// ```
    pub fn set_clean_state(&mut self) {
        self.msg = OSC94State::Clean;
        if self.is_support {
            self.msg.send();
        }
    }

    /// Set the process state to Error.
    ///
    /// Indicates that an error occurred during process execution.
    ///
    /// # Example
    ///
    /// ```
    /// use mingling::res::{OSC94, OSC94State};
    ///
    /// let osc94 = OSC94::default();
    /// let mut guard = osc94.get_mut();
    /// guard.set_error_state();
    /// assert_eq!(guard.state(), OSC94State::Error);
    /// ```
    pub fn set_error_state(&mut self) {
        self.msg = OSC94State::Error;
        if self.is_support {
            self.msg.send();
        }
    }

    /// Set the process state to Warn.
    ///
    /// Indicates that a warning occurred during process execution, but it has not
    /// reached the level of an error.
    ///
    /// # Example
    ///
    /// ```
    /// use mingling::res::{OSC94, OSC94State};
    ///
    /// let osc94 = OSC94::default();
    /// let mut guard = osc94.get_mut();
    /// guard.set_warn_state();
    /// assert_eq!(guard.state(), OSC94State::Warn);
    /// ```
    pub fn set_warn_state(&mut self) {
        self.msg = OSC94State::Warn;
        if self.is_support {
            self.msg.send();
        }
    }

    /// Set the process state to Unknown.
    ///
    /// Indicates that the process state cannot be determined or has not been defined.
    ///
    /// # Example
    ///
    /// ```
    /// use mingling::res::{OSC94, OSC94State};
    ///
    /// let osc94 = OSC94::default();
    /// let mut guard = osc94.get_mut();
    /// guard.set_unknown_state();
    /// assert_eq!(guard.state(), OSC94State::Unknown);
    /// ```
    pub fn set_unknown_state(&mut self) {
        self.msg = OSC94State::Unknown;
        if self.is_support {
            self.msg.send();
        }
    }

    /// Set the progress of the process.
    ///
    /// The `progress` parameter should be between `0.0` and `1.0`. `0.0` indicates
    /// the start of the task, and `1.0` indicates the completion of the task.
    /// Values outside this range are not clamped, but it is recommended to keep them
    /// within this range.
    ///
    /// # Parameters
    ///
    /// * `progress` - The progress value, ranging from `0.0` to `1.0`.
    ///
    /// # Example
    ///
    /// ```
    /// use mingling::res::OSC94;
    ///
    /// let osc94 = OSC94::default();
    /// let mut guard = osc94.get_mut();
    /// guard.set_progress(0.5);
    /// assert_eq!(guard.progress(), 0.5);
    /// ```
    pub fn set_progress(&mut self, progress: f32) {
        self.msg = OSC94State::Normal(progress);
        if self.is_support {
            self.msg.send();
        }
    }

    /// Get the current process state.
    ///
    /// # Returns
    ///
    /// Returns the current [`OSC94State`] value, representing the state of the process.
    ///
    /// # Example
    ///
    /// ```
    /// use mingling::res::{OSC94, OSC94State};
    ///
    /// let osc94 = OSC94::default();
    /// let guard = osc94.get_mut();
    /// assert_eq!(guard.state(), OSC94State::Clean);
    /// ```
    #[must_use]
    pub const fn state(&self) -> OSC94State {
        self.msg
    }

    /// Get the current progress value.
    ///
    /// Returns the actual progress value only when the state is [`OSC94State::Normal`];
    /// otherwise returns `0.0`.
    ///
    /// # Returns
    ///
    /// Returns an `f32` progress value, ranging from `0.0` to `1.0`.
    ///
    /// # Example
    ///
    /// ```
    /// use mingling::res::OSC94;
    ///
    /// let osc94 = OSC94::default();
    /// let mut guard = osc94.get_mut();
    /// guard.set_progress(0.25);
    /// assert_eq!(guard.progress(), 0.25);
    /// ```
    #[must_use]
    pub const fn progress(&self) -> f32 {
        match self.msg {
            OSC94State::Normal(progress) => progress,
            _ => 0.0,
        }
    }
}

impl Drop for OSC94Guard {
    fn drop(&mut self) {
        OSC94State::Clean.send();
    }
}
