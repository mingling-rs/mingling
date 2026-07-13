use proc_macro::TokenStream;

mod internal_repeat;
mod req;

/// Core proc-macro: repeats a template body `count` times.
///
/// Internal call signature: `internal_repeat!(count => { template })`
#[proc_macro]
pub fn internal_repeat(input: TokenStream) -> TokenStream {
    internal_repeat::internal_repeat(input)
}

/// Quick builder for `PickerRequirement`.
///
/// # Syntax
///
/// ```ignore
/// use mingling_picker_macros::req;
///
/// let basic = req![name: String];
/// let with_short_name = req![name: String, 'n'];
/// let with_short_alias = req![name: String, 'n', "alias"];
/// let positional = req![String];
/// let positional_with_name = req![String, 'n', "alias"];
/// ```
#[proc_macro]
pub fn req(input: TokenStream) -> TokenStream {
    req::req(input)
}
