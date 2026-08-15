/// Defines how to parse user confirmation input.
///
/// A type implementing this trait determines which user input strings are treated as "yes" or "no".
pub trait ConfirmPredicate {
    /// Parses the user's input string, returning whether it is "yes".
    ///
    /// Returns `Some(true)` for yes, `Some(false)` for no,
    /// and `None` if the input cannot be parsed (requiring re-entry).
    fn is_yes(str: &str) -> Option<bool>;
}

/// A `ConfirmPredicate` implementation that accepts "y"/"yes" as yes and "n"/"no" as no.
///
/// Input comparison is case-insensitive and automatically trims leading/trailing whitespace.
///
/// # Examples
///
/// ```
/// use mingling::res::ResConfirm;
/// use mingling::confirm::YesConfirm;
///
/// let confirm = ResConfirm::default();
/// let confirmed = confirm.ask::<YesConfirm>("Continue? [y/n] ");
/// ```
pub struct YesConfirm;

/// A `ConfirmPredicate` implementation that accepts "true"/"t" as yes and "false"/"f" as no.
///
/// Input comparison is case-insensitive and automatically trims leading/trailing whitespace.
///
/// # Examples
///
/// ```
/// use mingling::res::ResConfirm;
/// use mingling::confirm::TrueConfirm;
///
/// let confirm = ResConfirm::default();
/// let confirmed = confirm.ask::<TrueConfirm>("Enable this feature? [true/false] ");
/// ```
pub struct TrueConfirm;

impl ConfirmPredicate for YesConfirm {
    fn is_yes(str: &str) -> Option<bool> {
        match str.trim().to_lowercase().as_str() {
            "y" | "yes" => Some(true),
            "n" | "no" => Some(false),
            _ => None,
        }
    }
}

impl ConfirmPredicate for TrueConfirm {
    fn is_yes(str: &str) -> Option<bool> {
        match str.trim().to_lowercase().as_str() {
            "true" | "t" => Some(true),
            "false" | "f" => Some(false),
            _ => None,
        }
    }
}
