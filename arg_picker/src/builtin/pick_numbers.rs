// Doc Not Optimize
use crate::{BoundaryCheck, SinglePickable, pickable_needed::*};

macro_rules! impl_single_pickable_num {
    ($($t:ty),+ $(,)?) => {
        $(impl SinglePickable for $t {
            fn pick_single(str: Option<&str>) -> PickerArgResult<Self> {
                match str {
                    Some(s) => s.parse::<$t>().map(PickerArgResult::Parsed).unwrap_or(PickerArgResult::NotFound),
                    None => PickerArgResult::NotFound,
                }
            }
        })+
    };
}

/// Returns `true` if `raw` looks like an integer (digits, optional `+`/`-` prefix,
/// no decimal point or exponent).
fn is_int_like(raw: &str) -> bool {
    let s = raw.trim();
    let bytes = s.as_bytes();
    if bytes.is_empty() {
        return false;
    }
    let i = usize::from(bytes[0] == b'-' || bytes[0] == b'+');
    if i >= bytes.len() {
        return false;
    }
    for &b in &bytes[i..] {
        if !b.is_ascii_digit() {
            return false;
        }
    }
    true
}

/// Returns `true` if `raw` looks like a float (contains `.`, `e`, or `E`).
fn is_float_like(raw: &str) -> bool {
    let s = raw.trim();
    s.contains('.') || s.contains('e') || s.contains('E')
}

impl_single_pickable_num! {
    i8, i16, i32, i64, i128, isize,
    u8, u16, u32, u64, u128, usize,
    f32, f64,
}

// Integer boundary: only accept strings that look like integers.
// Float-like strings trigger a boundary.
macro_rules! impl_boundary_check_int {
    ($($t:ty),+ $(,)?) => {
        $(impl BoundaryCheck for $t {
            fn check_boundary(raw: &str) -> bool {
                !is_int_like(raw)
            }
        })+
    };
}

impl_boundary_check_int! {
    i8, i16, i32, i64, i128, isize,
    u8, u16, u32, u64, u128, usize,
}

// Float boundary: only accept strings that look like floats.
// Integer-like strings trigger a boundary.
impl BoundaryCheck for f32 {
    fn check_boundary(raw: &str) -> bool {
        !is_float_like(raw) || raw.parse::<Self>().is_err()
    }
}

impl BoundaryCheck for f64 {
    fn check_boundary(raw: &str) -> bool {
        !is_float_like(raw) || raw.parse::<Self>().is_err()
    }
}
