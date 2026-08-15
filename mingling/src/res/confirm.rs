use std::io::{BufRead, Write};

use crate::confirm::{ConfirmCount, ConfirmPredicate};

/// A confirm for interactive confirmation.
///
/// This structure caches the confirmed state to avoid repeated prompts.
///
/// Typically, `ResConfirm` is registered via `ConfirmSetup`, and then injected into functions
/// through Mingling's resource injection system.
///
/// # Registration
///
/// Before use, the `ConfirmSetup` must be registered with the program:
///
/// ```
/// # use mingling::MockProgramCollect as ThisProgram;
/// use mingling::setup::ConfirmSetup;
/// use mingling::Program;
///
/// let mut program = Program::<ThisProgram>::new();
/// program.with_setup(ConfirmSetup);
/// ```
///
/// # Examples
///
/// ```
/// use mingling::res::ResConfirm;
/// use mingling::confirm::YesConfirm;
///
/// // In actual use, obtain the registered Confirm through the resource injection system
/// let confirm = ResConfirm::new_confirmed();
/// assert!(confirm.ask::<YesConfirm>("Continue? [y/n] "));
/// ```
#[derive(Debug, Default, Clone, Copy)]
pub struct ResConfirm {
    pub(crate) confirmed: bool,
}

impl ResConfirm {
    /// Creates a new `ResConfirm` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling::res::ResConfirm;
    ///
    /// let confirm = ResConfirm::new();
    /// ```
    #[must_use]
    pub const fn new() -> Self {
        Self { confirmed: false }
    }

    /// Creates a `Confirm` instance in the confirmed state.
    ///
    /// The returned `Confirm` will directly return `true` when calling [`ask`](ResConfirm::ask) or
    /// [`try_ask`](ResConfirm::try_ask), without prompting the user.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling::res::ResConfirm;
    /// use mingling::confirm::YesConfirm;
    ///
    /// let confirm = ResConfirm::new_confirmed();
    /// assert!(confirm.ask::<YesConfirm>("Continue? [y/n] "));
    /// ```
    #[must_use]
    pub const fn new_confirmed() -> Self {
        Self { confirmed: true }
    }

    /// Marks the Confirm as confirmed.
    ///
    /// After calling this method, subsequent calls to [`ask`](ResConfirm::ask) or
    /// [`try_ask`](ResConfirm::try_ask) on this Confirm will directly return `true`
    /// without prompting the user.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling::res::ResConfirm;
    /// use mingling::confirm::YesConfirm;
    ///
    /// let mut confirm = ResConfirm::new();
    /// confirm.set_confirmed();
    /// assert!(confirm.ask::<YesConfirm>("Continue? [y/n] "));
    /// ```
    pub const fn set_confirmed(&mut self) {
        self.confirmed = true;
    }

    /// Asks the user a confirmation question, with at most one attempt.
    ///
    /// Returns `false` if the user provides an unrecognizable answer.
    /// Returns `true` directly if already confirmed previously.
    ///
    /// # Parameters
    ///
    /// * `ask` - The prompt text to display to the user.
    ///
    /// # Returns
    ///
    /// Returns a boolean indicating whether the user confirmed. Returns `false` if the user's input
    /// could not be parsed or the maximum number of attempts was reached.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling::res::ResConfirm;
    /// use mingling::confirm::YesConfirm;
    ///
    /// let confirm = ResConfirm::new_confirmed();
    /// let confirmed = confirm.ask::<YesConfirm>("Delete this file? [y/n] ");
    /// ```
    pub fn ask<P: ConfirmPredicate>(&self, ask: impl AsRef<str>) -> bool {
        self.try_ask::<P>(ask, ConfirmCount::Max(1))
            .unwrap_or(false)
    }

    /// Asks the user a confirmation question, allowing a specified maximum number of attempts.
    ///
    /// # Parameters
    ///
    /// * `ask` - The prompt text to display to the user.
    /// * `count` - The maximum number of attempts. Passing `0` means unlimited attempts (loop
    ///   indefinitely), passing a positive integer means at most that many attempts.
    ///
    /// # Returns
    ///
    /// Returns `Some(true)` for confirmation, `Some(false)` for rejection.
    /// Returns `None` if the maximum number of attempts is reached without being able to parse
    /// the user's input.
    ///
    /// # Panics
    ///
    /// This function panics when the standard error output (`stderr`) cannot be flushed or when
    /// reading from standard input fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling::res::ResConfirm;
    /// use mingling::confirm::YesConfirm;
    ///
    /// let confirm = ResConfirm::new_confirmed();
    /// let confirmed = confirm.try_ask::<YesConfirm>("Confirm execution? [y/n] ", 3);
    /// ```
    pub fn try_ask<P: ConfirmPredicate>(
        &self,
        ask: impl AsRef<str>,
        count: impl Into<ConfirmCount>,
    ) -> Option<bool> {
        if self.confirmed {
            return Some(true);
        }

        let count = count.into();
        let mut attempts = 0usize;

        loop {
            eprint!("{}", ask.as_ref());
            std::io::stderr().flush().unwrap();

            let stdin = std::io::stdin();
            let mut input = String::new();
            stdin.lock().read_line(&mut input).unwrap();
            if let Some(result) = P::is_yes(&input) {
                return Some(result);
            }

            attempts += 1;
            match count {
                ConfirmCount::Loop => {}
                ConfirmCount::Max(max) => {
                    if attempts >= max {
                        return None;
                    }
                }
            }
        }
    }
}
