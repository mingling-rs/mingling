// Using `assert_eq!(x, true)` is clearer than `assert!(x)` for expressing expected values
//
// BECAUSE `assert!` only checks if the boolean value is true,
// while `assert_eq!` explicitly shows the expected value
#![allow(clippy::bool_assert_comparison)]

#[cfg(test)]
mod test;

use mingling_picker::parselib::MaskedArg;

/// Create a single `MaskedArg` from a raw string and its original index.
pub fn make_masked(raw: &str, idx: usize) -> MaskedArg<'_> {
    MaskedArg { raw, raw_idx: idx }
}

/// Create a `Vec<MaskedArg>` from an array of `(raw, raw_idx)` pairs.
pub fn make_args<'a>(pairs: &'a [(&'a str, usize)]) -> Vec<MaskedArg<'a>> {
    pairs
        .iter()
        .map(|&(raw, idx)| MaskedArg { raw, raw_idx: idx })
        .collect()
}
