use crate::{Pickable, PickerArgInfo, parselib::ParserStyle};
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
/// - `_type`: [`PhantomData`] to hold the type parameter.
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

    /// [`PhantomData`] to hold the type parameter.
    pub internal_type: PhantomData<Type>,
}

impl<'a, Type> From<&'a Self> for PickerArg<'a, Type>
where
    Type: Pickable<'a>,
{
    fn from(value: &'a Self) -> Self {
        Self {
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
    #[must_use]
    pub const fn new(full: &'a [&'a str], short: Option<char>, positional: bool) -> Self {
        Self {
            full,
            short,
            positional,
            internal_type: PhantomData,
        }
    }

    /// Returns the full name list (including aliases).
    #[must_use]
    pub const fn full(&self) -> &'a [&'a str] {
        self.full
    }

    /// Returns the short name, if any.
    #[must_use]
    pub const fn short(&self) -> Option<char> {
        self.short
    }

    /// Returns whether the parameter is positional.
    ///
    /// If `full` is empty or `short` is `None`, the parameter is considered positional
    /// regardless of the stored value.
    #[must_use]
    pub const fn is_positional(&self) -> bool {
        if self.full.is_empty() && self.short.is_none() {
            true
        } else {
            self.positional
        }
    }

    /// Sets the full name list.
    pub const fn set_full(&mut self, full: &'a [&'a str]) {
        self.full = full;
    }

    /// Sets the short name.
    pub const fn set_short(&mut self, short: Option<char>) {
        self.short = short;
    }

    /// Sets whether the parameter is positional.
    pub const fn set_positional(&mut self, positional: bool) {
        self.positional = positional;
    }

    /// Sets the full name list and returns self.
    #[must_use]
    pub const fn with_full(mut self, full: &'a [&'a str]) -> Self {
        self.full = full;
        self
    }

    /// Clears the full name list (sets it to an empty slice) and returns self.
    #[must_use]
    pub const fn without_full(mut self) -> Self {
        self.full = &[];
        self
    }

    /// Sets the short name to the given character and returns self.
    #[must_use]
    pub const fn with_short(mut self, short: char) -> Self {
        self.short = Some(short);
        self
    }

    /// Clears the short name (sets it to None) and returns self.
    #[must_use]
    pub const fn without_short(mut self) -> Self {
        self.short = None;
        self
    }

    /// Sets whether the parameter is positional and returns self.
    #[must_use]
    pub const fn with_positional(mut self, positional: bool) -> Self {
        self.positional = positional;
        self
    }

    /// Converts this `PickerArg` into a `PickerArgInfo` value.
    ///
    /// This is a convenience method equivalent to calling `PickerArgInfo::from(self)`.
    #[must_use]
    pub fn into_info(self) -> PickerArgInfo<'a> {
        let value = self;
        let (long, alias) = if value.full.is_empty() {
            (None, None)
        } else {
            let long = Some(value.full[0]);
            let alias = if value.full.len() > 1 {
                Some(value.full[1..].to_vec())
            } else {
                None
            };
            (long, alias)
        };

        PickerArgInfo {
            short: value.short,
            long,
            alias,
            positional: value.positional,
            optional: false,
            multi: false,
            is_flag: false,
        }
    }
}

impl<'a, Type> From<PickerArg<'a, Type>> for Vec<String>
where
    Type: Pickable<'a>,
{
    fn from(value: PickerArg<'a, Type>) -> Self {
        let mut result = Self::new();
        let info = PickerArgInfo::from(value);
        let possible_flags =
            crate::parselib::build_possible_flags(ParserStyle::global_style(), &info);
        for flag in possible_flags {
            result.push(flag);
        }
        result
    }
}

/// Describes the attribute (behavior) of a command-line parameter.
///
/// The ordering reflects parse priority (higher = parsed first):
/// `Postprocess < Final < PositionalMulti < Positional < Flag < Single < Multi < Begin < Preprocess`
///
/// # Variants
///
/// - `Postprocess` — Reserved lowest priority, used only in special cases.
/// - `Final` — Reserved post-processing priority, used only in special cases.
/// - `PositionalMulti` — Positional argument that accepts multiple values (e.g., multiple input files).
/// - `Positional` — Positional argument matched by its position (e.g., an input file).
/// - `Flag` — Boolean flag with no associated value (e.g., `--verbose`).
/// - `Single` — Accepts a single value (e.g., `--name Alice`).
/// - `Multi` — Accepts multiple values (e.g., `--file a.txt --file b.txt`).
/// - `Begin` — Reserved pre-processing priority, used only in special cases.
/// - `Preprocess` — Reserved highest priority, used only in special cases.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PickerArgAttr {
    /// Reserved lowest priority, used only in special cases.
    Postprocess,

    /// Reserved post-processing priority, used only in special cases.
    Final,

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

    /// Reserved pre-processing priority, used only in special cases.
    Begin,

    /// Reserved highest priority, used only in special cases.
    Preprocess,
}

impl PickerArgAttr {
    /// Determines if the given `PickerArg` represents a positional parameter.
    ///
    /// If the flag is positional (determined by `flag.is_positional()`), returns
    /// `Self::Positional`. Otherwise, invokes the `other` closure to
    /// produce and return a `Self`.
    ///
    /// # Parameters
    ///
    /// - `flag`: A reference to the [`PickerArg`] to evaluate.
    /// - `other`: A closure that returns a [`PickerArgAttr`] when the flag is
    ///   **not** positional.
    #[inline]
    pub fn positional_or_else<'a, T>(flag: &PickerArg<'a, T>, other: fn() -> Self) -> Self
    where
        T: Pickable<'a>,
    {
        if flag.is_positional() {
            Self::Positional
        } else {
            other()
        }
    }

    /// Determines if the given `PickerArg` represents a positional parameter and returns
    /// `Self::Positional` if so. Otherwise, returns the provided `default` attribute.
    ///
    /// # Parameters
    ///
    /// - `flag`: A reference to the [`PickerArg`] to evaluate.
    /// - `default`: The [`PickerArgAttr`] to return if the flag is not positional.
    #[must_use]
    #[inline]
    pub const fn positional_or<'a, T>(flag: &PickerArg<'a, T>, default: Self) -> Self
    where
        T: Pickable<'a>,
    {
        if flag.is_positional() {
            Self::Positional
        } else {
            default
        }
    }

    /// Determines if the given `PickerArg` represents a positional parameter and returns
    /// `Self::Positional` if so. Otherwise, returns `Self::Single`.
    ///
    /// # Parameters
    ///
    /// - `flag`: A reference to the [`PickerArg`] to evaluate.
    #[must_use]
    #[inline]
    pub const fn positional_or_single<'a, T>(flag: &PickerArg<'a, T>) -> Self
    where
        T: Pickable<'a>,
    {
        if flag.is_positional() {
            Self::Positional
        } else {
            Self::Single
        }
    }

    /// Determines if the given `PickerArg` represents a positional parameter and returns
    /// `Self::PositionalMulti` if so. Otherwise, returns `Self::Multi`.
    ///
    /// # Parameters
    ///
    /// - `flag`: A reference to the [`PickerArg`] to evaluate.
    #[must_use]
    #[inline]
    pub const fn positional_or_multi<'a, T>(flag: &PickerArg<'a, T>) -> Self
    where
        T: Pickable<'a>,
    {
        if flag.is_positional() {
            Self::PositionalMulti
        } else {
            Self::Multi
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
