use crate::{Pickable, PickerFlag};

/// Represents a command-line argument/flag tag for a picker.
///
/// This struct defines how a picker argument should be parsed from the command line,
/// including its short name (e.g., `-n`), long name (e.g., `--name`), and aliases.
///
/// # Fields
///
/// * `short` - The short form of the tag, typically a single character (e.g., `-n`).
/// * `long` - The long form of the tag (e.g., `--name`).
/// * `alias` - Alternative names for the tag (e.g., `["-N", "--nickname"]`).
/// * `positional` - Whether this tag is a positional argument (no `-` or `--` prefix).
/// * `optional` - Whether this tag is optional or required.
/// * `multi` - Whether this tag can accept multiple values.
/// * `is_flag` - Whether this tag participates in parsing after a `--` separator.
pub struct PickerTag<'a> {
    /// The short form of the tag, e.g. `'n'` for `-n`.
    pub short: Option<char>,
    /// The long form of the tag, e.g. `"name"` for `--name`.
    pub long: Option<&'a str>,
    /// Alternative names for the tag, e.g. `["-N", "--nickname"]`.
    pub alias: Option<Vec<&'a str>>,
    /// Whether this tag is a positional argument (no `-` or `--` prefix).
    pub positional: bool,
    /// Whether this tag is optional or required.
    pub optional: bool,
    /// Whether this tag can accept multiple values.
    pub multi: bool,
    /// Whether this tag participates in parsing after a `--` separator.
    pub is_flag: bool,
}

impl<'a, T> From<PickerFlag<'a, T>> for PickerTag<'a>
where
    T: Pickable<'a> + Default,
{
    fn from(value: PickerFlag<'a, T>) -> Self {
        let (long, alias) = match value.full.len() {
            0 => (None, None),
            _ => {
                let long = Some(value.full[0]);
                let alias = if value.full.len() > 1 {
                    Some(value.full[1..].to_vec())
                } else {
                    None
                };
                (long, alias)
            }
        };

        Self {
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

impl<'a, T: Pickable<'a> + Default> From<&'a PickerFlag<'a, T>> for PickerTag<'a> {
    fn from(value: &'a PickerFlag<'a, T>) -> Self {
        let (long, alias) = match value.full.len() {
            0 => (None, None),
            _ => {
                let long = Some(value.full[0]);
                let alias = if value.full.len() > 1 {
                    Some(value.full[1..].to_vec())
                } else {
                    None
                };
                (long, alias)
            }
        };

        Self {
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

impl<'a> PickerTag<'a> {
    /// Create a new `PickerTag` with default values.
    pub fn new() -> Self {
        Self {
            short: None,
            long: None,
            alias: None,
            positional: false,
            optional: false,
            multi: false,
            is_flag: false,
        }
    }

    /// Set the short flag (e.g., `'n'` for `-n`).
    pub fn with_short(mut self, short: char) -> Self {
        self.short = Some(short);
        self
    }

    /// Set the long flag (e.g., `"name"` for `--name`).
    pub fn with_long(mut self, long: &'a str) -> Self {
        self.long = Some(long);
        self
    }

    /// Set aliases for the tag.
    pub fn with_alias(mut self, alias: Vec<&'a str>) -> Self {
        self.alias = Some(alias);
        self
    }

    /// Mark the tag as positional.
    pub fn with_positional(mut self, positional: bool) -> Self {
        self.positional = positional;
        self
    }

    /// Mark the tag as optional.
    pub fn with_optional(mut self, optional: bool) -> Self {
        self.optional = optional;
        self
    }

    /// Mark the tag as multi-value.
    pub fn with_multi(mut self, multi: bool) -> Self {
        self.multi = multi;
        self
    }

    /// Mark the tag as a flag that participates in parsing after `--`.
    pub fn with_is_flag(mut self, is_flag: bool) -> Self {
        self.is_flag = is_flag;
        self
    }

    /// Set the short flag (e.g., `'n'` for `-n`).
    pub fn set_short(&mut self, short: char) -> &mut Self {
        self.short = Some(short);
        self
    }

    /// Set the long flag (e.g., `"name"` for `--name`).
    pub fn set_long(&mut self, long: &'a str) -> &mut Self {
        self.long = Some(long);
        self
    }

    /// Set aliases for the tag.
    pub fn set_alias(&mut self, alias: Vec<&'a str>) -> &mut Self {
        self.alias = Some(alias);
        self
    }

    /// Set whether this tag is positional.
    pub fn set_positional(&mut self, positional: bool) -> &mut Self {
        self.positional = positional;
        self
    }

    /// Set whether this tag is optional.
    pub fn set_optional(&mut self, optional: bool) -> &mut Self {
        self.optional = optional;
        self
    }

    /// Set whether this tag accepts multiple values.
    pub fn set_multi(&mut self, multi: bool) -> &mut Self {
        self.multi = multi;
        self
    }

    /// Set whether this tag participates in parsing after a `--` separator.
    pub fn set_is_flag(&mut self, is_flag: bool) -> &mut Self {
        self.is_flag = is_flag;
        self
    }
}

impl<'a> Default for PickerTag<'a> {
    fn default() -> Self {
        Self::new()
    }
}
