use crate::osc94::{OSC94Guard, OSC94State};

/// Process `OSC 9;4` status.
///
/// Provides support for the `OSC 9;4` protocol. You can inject it into the execution flow
/// through Mingling's resource injection system, and use it to control your process state.
///
/// Typically, `OSC94` is registered via `OSC94Setup`, and then injected into functions
/// through Mingling's resource injection system.
///
/// # Registration
///
/// Before use, the `OSC94Setup` must be registered with the program:
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
/// use mingling::res::OSC94;
/// use mingling::osc94::OSC94State;
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
    /// use mingling::res::OSC94;
    /// use mingling::osc94::OSC94State;
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
