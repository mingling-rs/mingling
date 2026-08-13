// Doc Not Optimize
use std::fmt::Display;

use crate::{ChainProcess, NextProcess, ProgramCollect};

/// Asserts that the chain process result has the expected output type and next state.
///
/// This function is used to verify that a `ChainProcess` result meets expectations. It optionally checks:
/// - `member_id`: whether the output type identifier matches the expected one.
/// - `next`: whether the next processing step state matches the expected one.
///
/// If the result is not `ChainProcess::Ok` or any checked field does not match, the function will panic with a detailed error message.
///
/// # Parameters
///
/// * `result` - A reference to the chain process result to assert.
/// * `next` - The expected next state (optional). If `None`, this check is skipped.
/// * `member_id` - The expected output type identifier (optional). If `None`, this check is skipped.
///
/// # Generic Constraints
///
/// * `C` - Must implement `ProgramCollect`, `Display`, `PartialEq` and have a `'static` lifetime.
///
/// # Panics
///
/// Panics in the following cases:
/// - The result is `ChainProcess::Err`, outputting the error message.
/// - When `member_id` is not `None` and does not equal the actual output's `member_id`, displaying the expected and actual values.
/// - When `next` is not `None` and does not equal the actual next state, displaying the expected and actual values.
pub fn assert_next_eq<C>(result: &ChainProcess<C>, next: Option<NextProcess>, member_id: Option<C>)
where
    C: ProgramCollect + Display + PartialEq + 'static,
{
    match result {
        ChainProcess::Ok(any) => {
            if let Some(member_id) = member_id
                && member_id != any.0.member_id
            {
                panic!(
                    "Unexpected result type: expected {}, found {}",
                    member_id, any.0.member_id
                );
            }
            if let Some(next) = next
                && next != any.1
            {
                panic!("Unexpected next state: expected {}, found {}", next, any.1);
            }
        }
        ChainProcess::Err(chain_process_error) => {
            panic!("Chain process error: {chain_process_error}");
        }
    }
}

/// Unpacks the inner value from a `ChainProcess::Ok` result by downcasting it to the expected type.
///
/// This function extracts a reference to the output value of type `Type` from a `ChainProcess`
/// result. It is useful when you know the exact output type and just want to get a reference to
/// the inner data, without having to match on the enum yourself.
///
/// # Parameters
///
/// * `result` - A reference to the chain process result to unpack.
///
/// # Generic Constraints
///
/// * `C` - Must implement `ProgramCollect`, `Display`, `PartialEq` and have a `'static` lifetime.
/// * `Type` - The expected output type. Must be `'static` and match the actual output type stored
///   in the result.
///
/// # Panics
///
/// Panics in the following cases:
/// - The result is `ChainProcess::Err`, outputting the error message via `Debug`.
/// - The downcast fails because the actual output type does not match `Type`, panicking with a
///   message indicating a type mismatch.
///
/// # Returns
///
/// A reference to the inner value of type `Type` if the result is `ChainProcess::Ok` and the
/// downcast succeeds.
pub fn unpack_chain_process_result<C, Type: 'static>(result: &ChainProcess<C>) -> &Type
where
    C: ProgramCollect + Display + PartialEq + 'static,
{
    match result {
        ChainProcess::Ok((any, _next)) => any
            .downcast_ref::<Type>()
            .expect("Type mismatch: expected type does not match actual output type"),
        ChainProcess::Err(chain_process_error) => panic!("{chain_process_error:?}"),
    }
}

/// Asserts that a chain process result has the expected output type and next state.
///
/// This macro provides a convenient way to verify that a `ChainProcess` result meets expectations.
/// It wraps `assert_next_eq`, allowing optional checks on the output type (`member_id`) and the
/// next processing state.
///
/// # Parameters
///
/// * `$result` - An expression evaluating to a `ChainProcess<C>`.
/// * `$expected_next` - (Optional) The expected next state, expressed as a path. If provided, the
///   macro checks that the result's next state matches this value.
/// * `$expected_type` - (Optional) The expected output type, expressed as a path. If provided, the
///   macro checks that the result's `member_id` matches this value.
///
/// # Panics
///
/// Panics if the result is `ChainProcess::Err`, or if any provided expectation does not match the
/// actual value.
///
/// # Examples
///
/// ```ignore
/// // Check both next state and output type
/// assert_next!(result, NextProcess::End, MyOutputType);
///
/// // Check only next state
/// assert_next!(result, NextProcess::End);
///
/// // Check only that the result is Ok
/// assert_next!(result);
/// ```
#[macro_export]
macro_rules! assert_next {
    ($result:expr, $expected_next:path, $expected_type:path) => {
        ::mingling::test::assert_next_eq(&$result, Some($expected_next), Some($expected_type))
    };
    ($result:expr, $expected_next:path) => {
        ::mingling::test::assert_next_eq(&$result, Some($expected_next), None)
    };
    ($result:expr) => {
        ::mingling::test::assert_next_eq(&$result, None, None)
    };
}

/// Asserts that a chain process result is `Ok` without checking the output type or next state.
///
/// This macro checks that the `ChainProcess` result is `ChainProcess::Ok`, panicking if it is
/// `ChainProcess::Err`. It does **not** validate the output type identifier (`member_id`) or the
/// next processing state, making it useful when you only need to verify that the chain completed
/// without errors.
///
/// # Parameters
///
/// * `$result` - An expression evaluating to a `ChainProcess<C>`.
///
/// # Panics
///
/// Panics if the result is `ChainProcess::Err`, displaying the error message.
///
/// # Example
///
/// ```ignore
/// assert_chain_result!(result);
/// ```
#[macro_export]
macro_rules! assert_chain_result {
    ($result:expr) => {
        ::mingling::test::assert_next_eq(&$result, None, None)
    };
}

/// Alias for `assert_chain_result`.
///
/// This macro provides a more semantic name when asserting the result of a render operation.
/// It checks that the `ChainProcess` result is `Ok` without verifying specific output types or next states.
///
/// # Parameters
///
/// * `$result` - An expression evaluating to a `ChainProcess<C>`.
///
/// # Panics
///
/// Panics if the result is `ChainProcess::Err`, displaying the error message.
///
/// # Example
///
/// ```ignore
/// assert_render_result!(result);
/// ```
#[macro_export]
macro_rules! assert_render_result {
    ($result:expr) => {
        ::mingling::test::assert_next_eq(&$result, None, None)
    };
}

/// Asserts that the result's output type matches the expected `member_id`.
///
/// This macro checks that the `ChainProcess` result is `Ok` and that its output type identifier
/// matches the expected type. It is a convenience wrapper around `assert_next_eq` with the `next`
/// check skipped.
///
/// # Parameters
///
/// * `$result` - An expression evaluating to a `ChainProcess<C>`.
/// * `$expected_type` - The expected output type (a path like `SomeType`).
///
/// # Panics
///
/// Panics if the result is `ChainProcess::Err` or if the output `member_id` does not match the
/// expected type.
#[macro_export]
macro_rules! assert_member_id {
    ($result:expr, $expected_type:path) => {
        ::mingling::test::assert_next_eq(&$result, None, Some($expected_type))
    };
}

/// Unpacks a `ChainProcess::Ok` result by downcasting to the specified type.
///
/// This macro wraps the function `::mingling::test::unpack_chain_process_result` to provide
/// a more ergonomic way to extract the inner value from a chain process result.
///
/// # Parameters
///
/// * `$result` - A reference to or expression evaluating to a `ChainProcess<C>`.
/// * `$type` - The expected output type to downcast to.
///
/// # Panics
///
/// Panics if the result is `ChainProcess::Err` or if the downcast fails.
#[macro_export]
macro_rules! unpack_chain_process {
    ($result:expr, $type:ty) => {
        ::mingling::test::unpack_chain_process_result::<_, $type>(&$result)
    };
}
