use std::io::{BufRead, Write};

/// A confirmer for interactive confirmation.
///
/// This structure caches the confirmed state to avoid repeated prompts.
///
/// Typically, `Confirmer` is registered via [`ConfirmerSetup`], and then injected into functions
/// through Mingling's resource injection system.
///
/// # Registration
///
/// Before use, the [`ConfirmerSetup`] must be registered with the program:
///
/// ```
/// # use mingling::MockProgramCollect as ThisProgram;
/// use mingling::setup::ConfirmerSetup;
/// use mingling::Program;
///
/// let mut program = Program::<ThisProgram>::new();
/// program.with_setup(ConfirmerSetup);
/// ```
///
/// # Examples
///
/// ```
/// use mingling::res::{Confirmer, YesConfirm};
///
/// // In actual use, obtain the registered confirmer through the resource injection system
/// let confirmer = Confirmer::new_confirmed();
/// assert!(confirmer.ask::<YesConfirm>("Continue? [y/n] "));
/// ```
#[derive(Debug, Default, Clone, Copy)]
pub struct Confirmer {
    pub(crate) confirmed: bool,
}

impl Confirmer {
    /// Creates a new `Confirmer` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling::res::Confirmer;
    ///
    /// let confirmer = Confirmer::new();
    /// ```
    #[must_use]
    pub const fn new() -> Self {
        Self { confirmed: false }
    }

    /// Creates a `Confirmer` instance in the confirmed state.
    ///
    /// The returned `Confirmer` will directly return `true` when calling [`ask`](Confirmer::ask) or
    /// [`try_ask`](Confirmer::try_ask), without prompting the user.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling::res::{Confirmer, YesConfirm};
    ///
    /// let confirmer = Confirmer::new_confirmed();
    /// assert!(confirmer.ask::<YesConfirm>("Continue? [y/n] "));
    /// ```
    #[must_use]
    pub const fn new_confirmed() -> Self {
        Self { confirmed: true }
    }

    /// Marks the confirmer as confirmed.
    ///
    /// After calling this method, subsequent calls to [`ask`](Confirmer::ask) or
    /// [`try_ask`](Confirmer::try_ask) on this confirmer will directly return `true`
    /// without prompting the user.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling::res::{Confirmer, YesConfirm};
    ///
    /// let mut confirmer = Confirmer::new();
    /// confirmer.set_confirmed();
    /// assert!(confirmer.ask::<YesConfirm>("Continue? [y/n] "));
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
    /// use mingling::res::{Confirmer, YesConfirm};
    ///
    /// let confirmer = Confirmer::new_confirmed();
    /// let confirmed = confirmer.ask::<YesConfirm>("Delete this file? [y/n] ");
    /// ```
    pub fn ask<P: ConfirmerPredicate>(&self, ask: impl AsRef<str>) -> bool {
        self.try_ask::<P>(ask, ConfirmerCount::Max(1))
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
    /// use mingling::res::{Confirmer, YesConfirm};
    ///
    /// let confirmer = Confirmer::new_confirmed();
    /// let confirmed = confirmer.try_ask::<YesConfirm>("Confirm execution? [y/n] ", 3);
    /// ```
    pub fn try_ask<P: ConfirmerPredicate>(
        &self,
        ask: impl AsRef<str>,
        count: impl Into<ConfirmerCount>,
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
                ConfirmerCount::Loop => {}
                ConfirmerCount::Max(max) => {
                    if attempts >= max {
                        return None;
                    }
                }
            }
        }
    }
}

/// Specifies the maximum number of attempts for a confirmation prompt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmerCount {
    /// Loop indefinitely until the user gives a parseable answer.
    Loop,
    /// Ask at most the specified number of times.
    Max(usize),
}

macro_rules! impl_from_for_confirmer_count {
    ($($t:ty),*) => {
        $(
            impl From<$t> for ConfirmerCount {
                fn from(n: $t) -> Self {
                    if n == 0 {
                        ConfirmerCount::Loop
                    } else {
                        match usize::try_from(n) {
                            Ok(max) => ConfirmerCount::Max(max),
                            Err(_) => ConfirmerCount::Max(usize::MAX),
                        }
                    }
                }
            }
        )*
    };
}

impl_from_for_confirmer_count!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
);

/// Defines how to parse user confirmation input.
///
/// A type implementing this trait determines which user input strings are treated as "yes" or "no".
pub trait ConfirmerPredicate {
    /// Parses the user's input string, returning whether it is "yes".
    ///
    /// Returns `Some(true)` for yes, `Some(false)` for no,
    /// and `None` if the input cannot be parsed (requiring re-entry).
    fn is_yes(str: &str) -> Option<bool>;
}

/// A `ConfirmerPredicate` implementation that accepts "y"/"yes" as yes and "n"/"no" as no.
///
/// Input comparison is case-insensitive and automatically trims leading/trailing whitespace.
///
/// # Examples
///
/// ```
/// use mingling::res::{Confirmer, YesConfirm};
///
/// let confirmer = Confirmer::default();
/// let confirmed = confirmer.ask::<YesConfirm>("Continue? [y/n] ");
/// ```
pub struct YesConfirm;

/// A `ConfirmerPredicate` implementation that accepts "true"/"t" as yes and "false"/"f" as no.
///
/// Input comparison is case-insensitive and automatically trims leading/trailing whitespace.
///
/// # Examples
///
/// ```
/// use mingling::res::{Confirmer, TrueConfirm};
///
/// let confirmer = Confirmer::default();
/// let confirmed = confirmer.ask::<TrueConfirm>("Enable this feature? [true/false] ");
/// ```
pub struct TrueConfirm;

impl ConfirmerPredicate for YesConfirm {
    fn is_yes(str: &str) -> Option<bool> {
        match str.trim().to_lowercase().as_str() {
            "y" | "yes" => Some(true),
            "n" | "no" => Some(false),
            _ => None,
        }
    }
}

impl ConfirmerPredicate for TrueConfirm {
    fn is_yes(str: &str) -> Option<bool> {
        match str.trim().to_lowercase().as_str() {
            "true" | "t" => Some(true),
            "false" | "f" => Some(false),
            _ => None,
        }
    }
}
