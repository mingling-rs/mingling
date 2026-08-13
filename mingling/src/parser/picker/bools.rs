// Doc Not Optimize
use crate::parser::Pickable;

/// Represents a boolean-like value with `Yes` and `No` variants.
///
/// `Yes` can be parsed from command-line arguments using positive keywords such as `"y"` or `"yes"`,
/// and defaults to `No`.
#[derive(Debug, Default)]
#[repr(u8)]
pub enum Yes {
    /// The affirmative/positive variant.
    Yes,
    /// The negative/default variant.
    #[default]
    No,
}

impl From<bool> for Yes {
    fn from(b: bool) -> Self {
        if b { Self::Yes } else { Self::No }
    }
}

impl From<Yes> for bool {
    fn from(val: Yes) -> Self {
        match val {
            Yes::Yes => true,
            Yes::No => false,
        }
    }
}

impl std::ops::Deref for Yes {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        static TRUE: bool = true;
        static FALSE: bool = false;
        match self {
            Self::Yes => &TRUE,
            Self::No => &FALSE,
        }
    }
}

impl Yes {
    #[must_use]
    pub const fn is_yes(&self) -> bool {
        matches!(self, Self::Yes)
    }

    #[must_use]
    pub const fn is_no(&self) -> bool {
        matches!(self, Self::No)
    }
}

impl Pickable for Yes {
    type Output = Self;

    fn pick(args: &mut crate::parser::Argument, flag: mingling_core::Flag) -> Option<Self::Output> {
        let value = pick_bool(args, flag, &["y", "yes"]);
        Some(value.into())
    }
}

/// Represents a boolean-like value with `True` and `False` variants.
///
/// `True` can be parsed from command-line arguments using positive keywords such as `"t"` or `"true"`,
/// and defaults to `False`.
#[derive(Debug, Default)]
#[repr(u8)]
pub enum True {
    /// The affirmative/positive variant.
    True,
    /// The negative/default variant.
    #[default]
    False,
}

impl From<bool> for True {
    fn from(b: bool) -> Self {
        if b { Self::True } else { Self::False }
    }
}

impl From<True> for bool {
    fn from(val: True) -> Self {
        match val {
            True::True => true,
            True::False => false,
        }
    }
}

impl std::ops::Deref for True {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        static TRUE: bool = true;
        static FALSE: bool = false;
        match self {
            Self::True => &TRUE,
            Self::False => &FALSE,
        }
    }
}

impl True {
    #[must_use]
    pub const fn is_true(&self) -> bool {
        matches!(self, Self::True)
    }

    #[must_use]
    pub const fn is_false(&self) -> bool {
        matches!(self, Self::False)
    }
}

impl Pickable for True {
    type Output = Self;

    fn pick(args: &mut crate::parser::Argument, flag: mingling_core::Flag) -> Option<Self::Output> {
        let value = pick_bool(args, flag, &["true", "t"]);
        Some(value.into())
    }
}

fn pick_bool(
    args: &mut crate::parser::Argument,
    flag: mingling_core::Flag,
    positive: &[&str],
) -> bool {
    let content = args.pick_argument(flag);
    content.map_or_else(
        || false,
        |content| {
            let s = content.as_str();
            positive.contains(&s)
        },
    )
}
