use proc_macro::TokenStream;

use crate::func::r_print::expand_print;

pub(crate) fn r_println(input: TokenStream) -> TokenStream {
    expand_print(input, "println")
}
