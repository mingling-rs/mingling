use crate::{Pickable, PickerResult};

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
/// - `result`: The parsed result of this parameter requirement. Initially set to
///   `PickerResult::Unparsed`. After parsing, contains either the successfully parsed value or an
///   error.
#[derive(Default)]
pub struct PickerRequirement<'a, Type>
where
    Type: Default + Pickable,
{
    /// Full name, may include variant names (aliases), e.g., `["config", "cfg"]`.
    full: &'a [&'a str],

    /// Short name, e.g., `'c'`.
    short: Option<char>,

    /// Whether the parameter is positional (no flag, matched by position).
    positional: bool,

    /// The parsed result of this parameter requirement.
    result: PickerResult<Type>,
}

impl<'a, Type> PickerRequirement<'a, Type>
where
    Type: Default + Pickable,
{
    /// Creates a new `PickerRequirement` with the provided parameters.
    pub fn new(full: &'a [&'a str], short: Option<char>, positional: bool) -> Self {
        Self {
            full,
            short,
            positional,
            result: PickerResult::Unparsed,
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

    /// Returns a reference to the current parse result.
    pub fn result(&self) -> &PickerResult<Type> {
        &self.result
    }

    /// Returns a mutable reference to the current parse result.
    pub fn result_mut(&mut self) -> &mut PickerResult<Type> {
        &mut self.result
    }

    /// Sets the parse result.
    pub fn set_result(&mut self, result: PickerResult<Type>) {
        self.result = result;
    }

    /// Replaces the parse result with `PickerResult::Unparsed` and returns self.
    pub fn reset_result(mut self) -> Self {
        self.result = PickerResult::Unparsed;
        self
    }

    /// Sets the parse result and returns self.
    pub fn with_result(mut self, result: PickerResult<Type>) -> Self {
        self.result = result;
        self
    }
}
