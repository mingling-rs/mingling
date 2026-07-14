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
pub struct PickerFlag<'a, Type>
where
    Type: Default + Pickable<'a>,
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

impl<'a, Type> PickerFlag<'a, Type>
where
    Type: Default + Pickable<'a>,
{
    /// Creates a new `PickerFlag` with the provided parameters.
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
/// This enum specifies how a parameter is parsed and processed:
///
/// - `Single`: The parameter accepts a single value. For example, `--name Alice`.
///
/// - `Multi`: The parameter accepts multiple values. For example, `--file a.txt --file b.txt`.
///
/// - `Flag`: The parameter is a boolean flag with no associated value.
///   For example, `--verbose` sets a flag to `true`.
///
/// - `Positional`: The parameter is matched by its position in the command line,
///   without requiring a `--name` or `-n` flag. For example, an input filename as the first argument.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PickerFlagAttr {
    /// Accepts multiple values (e.g., `--file a.txt --file b.txt`).
    Multi,

    /// Accepts a single value (e.g., `--name Alice`).
    Single,

    /// Boolean flag with no associated value (e.g., `--verbose`).
    Flag,

    /// Positional argument matched by its position (e.g., an input file).
    #[default]
    Positional,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_picker_flag_attr_ordering() {
        // Multi > Single > Flag > Positional
        assert!(PickerFlagAttr::Multi > PickerFlagAttr::Single);
        assert!(PickerFlagAttr::Multi > PickerFlagAttr::Flag);
        assert!(PickerFlagAttr::Multi > PickerFlagAttr::Positional);

        assert!(PickerFlagAttr::Single > PickerFlagAttr::Flag);
        assert!(PickerFlagAttr::Single > PickerFlagAttr::Positional);

        assert!(PickerFlagAttr::Flag > PickerFlagAttr::Positional);

        // PartialOrd
        assert!(PickerFlagAttr::Multi >= PickerFlagAttr::Single);
        assert!(PickerFlagAttr::Single >= PickerFlagAttr::Flag);
        assert!(PickerFlagAttr::Flag >= PickerFlagAttr::Positional);

        assert!(PickerFlagAttr::Positional < PickerFlagAttr::Flag);
        assert!(PickerFlagAttr::Flag < PickerFlagAttr::Single);
        assert!(PickerFlagAttr::Single < PickerFlagAttr::Multi);

        // Sort
        let mut values = vec![
            PickerFlagAttr::Flag,
            PickerFlagAttr::Single,
            PickerFlagAttr::Positional,
            PickerFlagAttr::Multi,
        ];
        values.sort();
        assert_eq!(
            values,
            vec![
                PickerFlagAttr::Positional,
                PickerFlagAttr::Flag,
                PickerFlagAttr::Single,
                PickerFlagAttr::Multi,
            ]
        );
    }
}
