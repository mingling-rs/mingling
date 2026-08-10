use std::{
    fmt::{Debug, Display},
    ops::{Deref, Not},
};

/// Parsed result of a boolean-style command-line flag.
///
/// `Flag` is a **value type** that can be declared in [`PickerArg`].
/// When the user passes `--verbose` on the command line, the parsed result is `Flag::Active`;
/// when the flag is absent, the result is `Flag::Inactive`.
///
/// # Why not just `bool`?
///
/// Unlike a raw `bool`, `Flag` carries **explicit semantics** about whether
/// the flag was actually provided by the user (`Active`) or simply omitted
/// (`Inactive`). This distinction matters when you want to distinguish
/// "the user intentionally omitted the flag" from "the flag was processed but
/// resolved to false" — the `Pickable` implementation for `Flag` always
/// returns `Parsed(Flag::Inactive)` when no matching argument is found,
/// rather than `NotFound`, making it always succeed with a meaningful default.
///
/// # Conversions
///
/// `Flag` interoperates seamlessly with `bool`: `Flag::Active` is `true`,
/// `Flag::Inactive` is `false`. The [`Deref`] impl allows using a `Flag`
/// directly in boolean contexts:
///
/// ```
/// # use arg_picker::value::Flag;
/// let flag = Flag::Active;
/// if *flag { /* runs */ }
/// ```
///
/// [`PickerArg`]: crate::PickerArg
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum Flag {
    /// The flag was **not** present on the command line.
    ///
    /// This is the default state, equivalent to `false`.
    #[default]
    Inactive,

    /// The flag **was** present on the command line.
    ///
    /// Equivalent to `true`.
    Active,
}

impl Debug for Flag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Inactive => write!(f, "inactive"),
            Self::Active => write!(f, "active"),
        }
    }
}

impl Display for Flag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Inactive => write!(f, "inactive"),
            Self::Active => write!(f, "active"),
        }
    }
}

impl Flag {
    /// Converts this `Flag` into a `bool`.
    ///
    /// Returns `true` if the flag is [`Active`], `false` if [`Inactive`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use arg_picker::value::Flag;
    /// assert!(Flag::Active.bool());
    /// assert!(!Flag::Inactive.bool());
    /// ```
    ///
    /// [`Active`]: Flag::Active
    /// [`Inactive`]: Flag::Inactive
    #[must_use]
    #[inline]
    pub fn bool(&self) -> bool {
        *self == Self::Active
    }
}

impl PartialEq<bool> for Flag {
    fn eq(&self, other: &bool) -> bool {
        self.bool() == *other
    }
}

/// Compares `bool` with `Flag` using `==`.
impl PartialEq<Flag> for bool {
    fn eq(&self, other: &Flag) -> bool {
        *self == other.bool()
    }
}

impl From<bool> for Flag {
    fn from(value: bool) -> Self {
        if value { Self::Active } else { Self::Inactive }
    }
}

impl From<Flag> for bool {
    fn from(val: Flag) -> Self {
        val == Flag::Active
    }
}

/// Allows `Flag` to be used in boolean contexts via `*flag`.
///
/// # Examples
///
/// ```
/// # use arg_picker::value::Flag;
/// let flag = Flag::Active;
/// if *flag {
///     println!("flag is set");
/// }
/// ```
impl Deref for Flag {
    type Target = bool;

    fn deref(&self) -> &bool {
        match self {
            Self::Active => &true,
            Self::Inactive => &false,
        }
    }
}

impl Not for Flag {
    type Output = Self;

    fn not(self) -> Self {
        match self {
            Self::Active => Self::Inactive,
            Self::Inactive => Self::Active,
        }
    }
}
