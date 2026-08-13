// Doc Not Optimize
use proc_macro::TokenStream;

use crate::attr::chain;

pub(crate) fn register_chain_impl(input: TokenStream) -> TokenStream {
    chain::register_chain(input)
}
