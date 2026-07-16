#![doc = include_str!("../README.md")]

use proc_macro::TokenStream;

mod arg;
mod internal_repeat;

/// Core proc-macro: repeats a template body `count` times.
///
/// Internal call signature: `internal_repeat!(count => { template })`
#[proc_macro]
pub fn internal_repeat(input: TokenStream) -> TokenStream {
    internal_repeat::internal_repeat(input)
}

/// Quick builder for `PickerArg`.
///
/// # Syntax
///
/// ```ignore
/// use mingling_picker_macros::flag;
///
/// let basic = arg![name: String];
/// let with_short_name = arg![name: String, 'n'];
/// let with_short_alias = arg![name: String, 'n', "alias"];
/// let positional = arg![String];
/// let positional_with_name = arg![String, 'n', "alias"];
/// ```
#[proc_macro]
pub fn arg(input: TokenStream) -> TokenStream {
    arg::arg(input)
}
