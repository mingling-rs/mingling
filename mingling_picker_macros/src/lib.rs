use proc_macro::TokenStream;

mod flag;
mod internal_repeat;

/// Core proc-macro: repeats a template body `count` times.
///
/// Internal call signature: `internal_repeat!(count => { template })`
#[proc_macro]
pub fn internal_repeat(input: TokenStream) -> TokenStream {
    internal_repeat::internal_repeat(input)
}

/// Quick builder for `PickerFlag`.
///
/// # Syntax
///
/// ```ignore
/// use mingling_picker_macros::flag;
///
/// let basic = flag![name: String];
/// let with_short_name = flag![name: String, 'n'];
/// let with_short_alias = flag![name: String, 'n', "alias"];
/// let positional = flag![String];
/// let positional_with_name = flag![String, 'n', "alias"];
/// ```
#[proc_macro]
pub fn flag(input: TokenStream) -> TokenStream {
    flag::flag(input)
}
