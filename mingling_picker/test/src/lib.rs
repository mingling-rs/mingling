// Using `assert_eq!(x, true)` is clearer than `assert!(x)` for expressing expected values
//
// BECAUSE `assert!` only checks if the boolean value is true,
// while `assert_eq!` explicitly shows the expected value
#![allow(clippy::bool_assert_comparison)]

#[cfg(test)]
mod test;
