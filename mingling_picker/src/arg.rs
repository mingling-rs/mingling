use crate::Pickable;
use std::marker::PhantomData;

/// Represents a constraint definition for a parameter selection.
///
/// This structure describes the constraints that a command-line parameter (Picker parameter item)
/// should satisfy, including its full name list (with aliases), short name form, and whether it is
/// positional.
///
/// # Field Descriptions
///
/// - `full`: Full name or alias list. For example, `["config", "cfg"]` means the parameter can be
///   matched with either `--config` or `--cfg`. Must contain at least one non-empty string.
///
/// - `short`: Short name (single character). For example, `Some('c')` means it can be passed using
///   the `-c` form. If set to `None`, the short name form is not supported.
///
/// - `positional`: Whether the parameter is positional (i.e., an argument without a flag).
///   - `true`: The parameter is positional; it is matched by its position in the command line rather
///     than by a `--name` or `-n` flag.
///   - `false`: The parameter is a named (flag-based) parameter.
///
/// - `_type`: PhantomData to hold the type parameter.
#[derive(Default, Clone, Copy)]
pub struct PickerArg<'a, Type>
where
    Type: Pickable<'a>,
{
    /// Full name, may include variant names (aliases), e.g., `["config", "cfg"]`.
    pub full: &'a [&'a str],

    /// Short name, e.g., `'c'`.
    pub short: Option<char>,

    /// Whether the parameter is positional (no flag, matched by position).
    pub positional: bool,

    /// PhantomData to hold the type parameter.
    pub internal_type: PhantomData<Type>,
}

impl<'a, Type> From<&'a PickerArg<'a, Type>> for PickerArg<'a, Type>
where
    Type: Pickable<'a>,
{
    fn from(value: &'a PickerArg<'a, Type>) -> Self {
        PickerArg {
            full: value.full,
            short: value.short,
            positional: value.positional,
            internal_type: PhantomData,
        }
    }
}

impl<'a, Type> PickerArg<'a, Type>
where
    Type: Pickable<'a>,
{
    /// Creates a new `PickerArg` with the provided parameters.
    pub fn new(full: &'a [&'a str], short: Option<char>, positional: bool) -> Self {
        Self {
            full,
            short,
            positional,
            internal_type: PhantomData,
        }
    }

    /// Returns the full name list (including aliases).
    pub fn full(&self) -> &'a [&'a str] {
        self.full
    }

    /// Returns the short name, if any.
    pub fn short(&self) -> Option<char> {
        self.short
    }

    /// Returns whether the parameter is positional.
    ///
    /// If `full` is empty or `short` is `None`, the parameter is considered positional
    /// regardless of the stored value.
    pub fn is_positional(&self) -> bool {
        if self.full.is_empty() && self.short.is_none() {
            true
        } else {
            self.positional
        }
    }

    /// Sets the full name list.
    pub fn set_full(&mut self, full: &'a [&'a str]) {
        self.full = full;
    }

    /// Sets the short name.
    pub fn set_short(&mut self, short: Option<char>) {
        self.short = short;
    }

    /// Sets whether the parameter is positional.
    pub fn set_positional(&mut self, positional: bool) {
        self.positional = positional;
    }

    /// Sets the full name list and returns self.
    pub fn with_full(mut self, full: &'a [&'a str]) -> Self {
        self.full = full;
        self
    }

    /// Clears the full name list (sets it to an empty slice) and returns self.
    pub fn without_full(mut self) -> Self {
        self.full = &[];
        self
    }

    /// Sets the short name to the given character and returns self.
    pub fn with_short(mut self, short: char) -> Self {
        self.short = Some(short);
        self
    }

    /// Clears the short name (sets it to None) and returns self.
    pub fn without_short(mut self) -> Self {
        self.short = None;
        self
    }

    /// Sets whether the parameter is positional and returns self.
    pub fn with_positional(mut self, positional: bool) -> Self {
        self.positional = positional;
        self
    }
}

/// Describes the attribute (behavior) of a command-line parameter.
///
/// The ordering reflects parse priority (higher = parsed first):
/// `PositionalMulti < Positional < Flag < Single < Multi`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PickerArgAttr {
    /// Positional argument that accepts multiple values (e.g., multiple input files).
    PositionalMulti,

    /// Positional argument matched by its position (e.g., an input file).
    #[default]
    Positional,

    /// Boolean flag with no associated value (e.g., `--verbose`).
    Flag,

    /// Accepts a single value (e.g., `--name Alice`).
    Single,

    /// Accepts multiple values (e.g., `--file a.txt --file b.txt`).
    Multi,
}

impl PickerArgAttr {
    /// Determines if the given `PickerArg` represents a positional parameter.
    ///
    /// If the flag is positional (determined by `flag.is_positional()`), returns
    /// `PickerArgAttr::Positional`. Otherwise, invokes the `other` closure to
    /// produce and return a `PickerArgAttr`.
    ///
    /// # Parameters
    ///
    /// - `flag`: A reference to the [`PickerArg`] to evaluate.
    /// - `other`: A closure that returns a [`PickerArgAttr`] when the flag is
    ///   **not** positional.
    #[inline(always)]
    pub fn positional_or_else<'a, T>(
        flag: &PickerArg<'a, T>,
        other: fn() -> PickerArgAttr,
    ) -> PickerArgAttr
    where
        T: Pickable<'a>,
    {
        if flag.is_positional() {
            PickerArgAttr::Positional
        } else {
            other()
        }
    }

    /// Determines if the given `PickerArg` represents a positional parameter and returns
    /// `PickerArgAttr::Positional` if so. Otherwise, returns the provided `default` attribute.
    ///
    /// # Parameters
    ///
    /// - `flag`: A reference to the [`PickerArg`] to evaluate.
    /// - `default`: The [`PickerArgAttr`] to return if the flag is not positional.
    #[inline(always)]
    pub fn positional_or<'a, T>(flag: &PickerArg<'a, T>, default: PickerArgAttr) -> PickerArgAttr
    where
        T: Pickable<'a>,
    {
        if flag.is_positional() {
            PickerArgAttr::Positional
        } else {
            default
        }
    }

    /// Determines if the given `PickerArg` represents a positional parameter and returns
    /// `PickerArgAttr::Positional` if so. Otherwise, returns `PickerArgAttr::Single`.
    ///
    /// # Parameters
    ///
    /// - `flag`: A reference to the [`PickerArg`] to evaluate.
    #[inline(always)]
    pub fn positional_or_single<'a, T>(flag: &PickerArg<'a, T>) -> PickerArgAttr
    where
        T: Pickable<'a>,
    {
        if flag.is_positional() {
            PickerArgAttr::Positional
        } else {
            PickerArgAttr::Single
        }
    }

    /// Determines if the given `PickerArg` represents a positional parameter and returns
    /// `PickerArgAttr::PositionalMulti` if so. Otherwise, returns `PickerArgAttr::Multi`.
    ///
    /// # Parameters
    ///
    /// - `flag`: A reference to the [`PickerArg`] to evaluate.
    #[inline(always)]
    pub fn positional_or_multi<'a, T>(flag: &PickerArg<'a, T>) -> PickerArgAttr
    where
        T: Pickable<'a>,
    {
        if flag.is_positional() {
            PickerArgAttr::PositionalMulti
        } else {
            PickerArgAttr::Multi
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_picker_flag_attr_ordering() {
        // Multi > Single > Flag > Positional > PositionalMulti
        assert!(PickerArgAttr::Multi > PickerArgAttr::Single);
        assert!(PickerArgAttr::Multi > PickerArgAttr::Flag);
        assert!(PickerArgAttr::Multi > PickerArgAttr::Positional);
        assert!(PickerArgAttr::Multi > PickerArgAttr::PositionalMulti);

        assert!(PickerArgAttr::Single > PickerArgAttr::Flag);
        assert!(PickerArgAttr::Single > PickerArgAttr::Positional);
        assert!(PickerArgAttr::Single > PickerArgAttr::PositionalMulti);

        assert!(PickerArgAttr::Flag > PickerArgAttr::Positional);
        assert!(PickerArgAttr::Flag > PickerArgAttr::PositionalMulti);

        assert!(PickerArgAttr::Positional > PickerArgAttr::PositionalMulti);

        // PartialOrd
        assert!(PickerArgAttr::Multi >= PickerArgAttr::Single);
        assert!(PickerArgAttr::Single >= PickerArgAttr::Flag);
        assert!(PickerArgAttr::Flag >= PickerArgAttr::Positional);
        assert!(PickerArgAttr::Positional >= PickerArgAttr::PositionalMulti);

        assert!(PickerArgAttr::PositionalMulti < PickerArgAttr::Positional);
        assert!(PickerArgAttr::Positional < PickerArgAttr::Flag);
        assert!(PickerArgAttr::Flag < PickerArgAttr::Single);
        assert!(PickerArgAttr::Single < PickerArgAttr::Multi);
    }

    #[test]
    fn test_picker_flag_attr_sorting() {
        // Sort
        let mut values = vec![
            PickerArgAttr::Flag,
            PickerArgAttr::Single,
            PickerArgAttr::Positional,
            PickerArgAttr::Multi,
            PickerArgAttr::PositionalMulti,
        ];
        values.sort();
        assert_eq!(
            values,
            vec![
                PickerArgAttr::PositionalMulti,
                PickerArgAttr::Positional,
                PickerArgAttr::Flag,
                PickerArgAttr::Single,
                PickerArgAttr::Multi,
            ]
        );
    }
}
