/// `OSC 9;4` protocol message
///
/// Used to send task progress notification messages to the terminal via ANSI escape sequences.
///
/// This protocol follows the [Windows Terminal Progress Bar Sequences](https://learn.microsoft.com/en-us/windows/terminal/tutorials/progress-bar-sequences) specification.
/// The status codes (0-4) represent: clear progress, normal, error, indeterminate, and warning states, respectively.
///
/// # Examples
///
/// ```rust
/// use mingling::osc94::OSC94State;
///
/// // Set progress to 50%
/// let state = OSC94State::Normal(0.5);
/// assert_eq!(state.state_code(), 1);
/// assert_eq!(state.progress(), 50.0);
///
/// // Generate escape sequence string
/// let seq = state.to_escape_sequence();
/// assert_eq!(seq, "\x1b]9;4;1;50\x07");
///
/// // Convert to string (Display implementation)
/// let s = format!("{state}");
/// assert_eq!(s, "\x1b]9;4;1;50\x07");
///
/// // Convert via From
/// let s2: String = state.into();
/// assert_eq!(s2, "\x1b]9;4;1;50\x07");
///
/// // Error state
/// let err = OSC94State::Error;
/// assert_eq!(err.state_code(), 2);
/// ```
///
/// # Use Cases
///
/// In command-line tools or scripts, the [`OSC94State::send`] method can be used to directly send progress notifications to the terminal.
/// Supported terminals include: `Windows Terminal`, `kitty`, `iTerm2`, `WezTerm`, `foot`, etc.
///
/// ```
/// use mingling::osc94::OSC94State;
///
/// // Send progress 100%
/// OSC94State::Normal(1.0).send();
/// // Send completion (clear) message
/// OSC94State::Clean.send();
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OSC94State {
    /// Clear/hide progress
    Clean,
    /// Normal state, corresponding to status code `1`, carries a progress value (0.0 to 1.0)
    Normal(f32),
    /// Error state, corresponding to status code `2`
    Error,
    /// Indeterminate state, corresponding to status code `3`
    Unknown,
    /// Warning state, corresponding to status code `4`
    Warn,
}

impl OSC94State {
    /// Returns the status code for the `OSC 9;4` protocol.
    ///
    /// Status code meanings:
    /// - `0`: Clear progress (`Clean`)
    /// - `1`: Normal state (`Normal`)
    /// - `2`: Error state (`Error`)
    /// - `3`: Indeterminate state (`Unknown`)
    /// - `4`: Warning state (`Warn`)
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling::osc94::OSC94State;
    ///
    /// assert_eq!(OSC94State::Clean.state_code(), 0);
    /// assert_eq!(OSC94State::Normal(0.5).state_code(), 1);
    /// assert_eq!(OSC94State::Error.state_code(), 2);
    /// assert_eq!(OSC94State::Unknown.state_code(), 3);
    /// assert_eq!(OSC94State::Warn.state_code(), 4);
    /// ```
    ///
    /// # Return Value
    ///
    /// Returns the corresponding status code (`u8` type), ranging from `0` to `4`.
    #[must_use]
    pub const fn state_code(&self) -> u8 {
        match self {
            Self::Clean => 0,
            Self::Normal(_) => 1,
            Self::Error => 2,
            Self::Unknown => 3,
            Self::Warn => 4,
        }
    }

    /// Returns the progress value (0-100), used for the `Normal` state, clamped to a valid range.
    ///
    /// This function converts the progress value (between 0.0 and 1.0) stored in the `Normal` variant
    /// into a percentage (0 to 100) and rounds it. For non-`Normal` states (such as `Clean`, `Error`,
    /// `Unknown`, `Warn`), it returns a fixed `0.0`, because only the `Normal` state carries progress information.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling::osc94::OSC94State;
    ///
    /// // Progress conversion in normal state
    /// assert_eq!(OSC94State::Normal(0.5).progress(), 50.0);
    /// assert_eq!(OSC94State::Normal(1.0).progress(), 100.0);
    /// assert_eq!(OSC94State::Normal(0.0).progress(), 0.0);
    ///
    /// // Out-of-range values are clamped to 0-100
    /// assert_eq!(OSC94State::Normal(1.5).progress(), 100.0);
    /// assert_eq!(OSC94State::Normal(-0.5).progress(), 0.0);
    ///
    /// // Rounding behavior
    /// assert_eq!(OSC94State::Normal(0.335).progress(), 34.0);
    /// assert_eq!(OSC94State::Normal(0.999).progress(), 100.0);
    ///
    /// // Non-Normal states return 0.0
    /// assert_eq!(OSC94State::Clean.progress(), 0.0);
    /// assert_eq!(OSC94State::Error.progress(), 0.0);
    /// assert_eq!(OSC94State::Unknown.progress(), 0.0);
    /// assert_eq!(OSC94State::Warn.progress(), 0.0);
    /// ```
    ///
    /// # Return Value
    ///
    /// Returns an `f32` progress percentage, ranging from `0.0` to `100.0` (inclusive).
    /// For the `Normal` state, returns the rounded result of converting its progress value to a percentage;
    /// for other states, always returns `0.0`.
    #[must_use]
    pub const fn progress(&self) -> f32 {
        match self {
            Self::Normal(progress) => (progress.clamp(0.0, 1.0) * 100.0).round(),
            _ => 0.0,
        }
    }

    /// Converts the message to the corresponding `OSC 9;4` escape sequence string.
    ///
    /// This method generates an ANSI escape sequence conforming to the
    /// [Windows Terminal Progress Bar Sequences](https://learn.microsoft.com/en-us/windows/terminal/tutorials/progress-bar-sequences)
    /// protocol based on the current state, in the format `\x1b]9;4;{status_code};{progress}\x07`.
    ///
    /// Escape sequence format description:
    /// - `\x1b]`: ESC character followed by `]`, marking the start of an OSC (Operating System Command) sequence.
    /// - `9;4`: Indicates the `OSC 9;4` protocol (task progress notification).
    /// - `{status_code}`: Task status, ranging from `0` (clear), `1` (normal), `2` (error), `3` (indeterminate), to `4` (warning).
    /// - `{progress}`: Task progress percentage (0-100), only meaningful for the `Normal` state.
    /// - `\x07`: BEL character, marking the end of the OSC sequence.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling::osc94::OSC94State;
    ///
    /// // Clear progress
    /// let clean = OSC94State::Clean;
    /// assert_eq!(clean.to_escape_sequence(), "\x1b]9;4;0;0\x07");
    ///
    /// // Normal state, progress 50%
    /// let normal = OSC94State::Normal(0.5);
    /// assert_eq!(normal.to_escape_sequence(), "\x1b]9;4;1;50\x07");
    ///
    /// // Normal state, progress 100%
    /// let complete = OSC94State::Normal(1.0);
    /// assert_eq!(complete.to_escape_sequence(), "\x1b]9;4;1;100\x07");
    ///
    /// // Error state
    /// let error = OSC94State::Error;
    /// assert_eq!(error.to_escape_sequence(), "\x1b]9;4;2;0\x07");
    ///
    /// // Indeterminate state
    /// let unknown = OSC94State::Unknown;
    /// assert_eq!(unknown.to_escape_sequence(), "\x1b]9;4;3;0\x07");
    ///
    /// // Warning state
    /// let warn = OSC94State::Warn;
    /// assert_eq!(warn.to_escape_sequence(), "\x1b]9;4;4;0\x07");
    /// ```
    ///
    /// # Return Value
    ///
    /// Returns a `String` containing an ANSI escape sequence conforming to the `OSC 9;4` protocol standard.
    /// This string can be directly output to a terminal that supports this protocol (such as `Windows Terminal`,
    /// `kitty`, `iTerm2`, `WezTerm`, `foot`, etc.) to display a task progress notification.
    #[must_use]
    pub fn to_escape_sequence(&self) -> String {
        format!("\x1b]9;4;{};{}\x07", self.state_code(), self.progress())
    }

    /// Sends the OSC 9;4 message to the terminal via stdout.
    ///
    /// This method outputs the escape sequence of the current state to standard output and flushes the buffer,
    /// allowing terminals that support the
    /// [Windows Terminal Progress Bar Sequences](https://learn.microsoft.com/en-us/windows/terminal/tutorials/progress-bar-sequences)
    /// protocol to display the corresponding task progress notification.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling::osc94::OSC94State;
    ///
    /// // Send normal state, progress 50%
    /// OSC94State::Normal(0.5).send();
    ///
    /// // Send error state
    /// OSC94State::Error.send();
    ///
    /// // Send clear progress message
    /// OSC94State::Clean.send();
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the stdout stream cannot be flushed.
    pub fn send(&self) {
        use std::io::Write;
        print!("{}", self.to_escape_sequence());
        std::io::stdout().flush().unwrap();
    }
}

impl std::fmt::Display for OSC94State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_escape_sequence())
    }
}

impl From<OSC94State> for String {
    fn from(msg: OSC94State) -> Self {
        msg.to_escape_sequence()
    }
}

impl From<&OSC94State> for String {
    fn from(msg: &OSC94State) -> Self {
        msg.to_escape_sequence()
    }
}
