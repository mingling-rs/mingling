// Doc Not Optimize
use proc_macro::TokenStream;
use quote::quote;

use crate::func::r_print::AppendInput;

pub(crate) fn r_append(input: TokenStream) -> TokenStream {
    let parsed: AppendInput = match syn::parse(input) {
        Ok(p) => p,
        Err(e) => return e.to_compile_error().into(),
    };

    let dst_ident = parsed.dst.clone();
    let src_tokens = parsed.src;

    let expanded = dst_ident.map_or_else(
        || {
            quote! {
                __render_result_buffer.append_other(#src_tokens);
            }
        },
        |dst| {
            quote! {
                #dst.append_other(#src_tokens);
            }
        },
    );

    expanded.into()
}
