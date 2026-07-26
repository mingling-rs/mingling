use proc_macro::TokenStream;

/// Marker attribute for the Mingling lint system.
/// All it does is pass the item through unchanged.
pub(crate) fn mlint(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
