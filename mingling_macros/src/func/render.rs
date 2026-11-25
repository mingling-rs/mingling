use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

/// Expands `render!` / `render_stdout!` / `render_stderr!`.
///
/// The input expression is expected to already be a reference (`&value`).
pub(crate) fn expand_render(input: TokenStream, mode: &str) -> TokenStream {
    let value = parse_macro_input!(input as syn::Expr);
    let mode = syn::Ident::new(mode, proc_macro2::Span::call_site());

    quote! {{
        ::mingling::render_stream::<crate::ThisProgram, _>(
            #value,
            ::mingling::RenderEmit::#mode,
        );
    }}
    .into()
}

/// Expands `render_owned!` as a wrapper around `render!`.
pub(crate) fn expand_render_owned(input: TokenStream) -> TokenStream {
    let value = parse_macro_input!(input as syn::Expr);

    quote! {{
        let __mingling_render_owned = #value;
        ::mingling::macros::render!(&__mingling_render_owned);
    }}
    .into()
}

/// Expands `render_each!`.
pub(crate) fn expand_render_each(input: TokenStream) -> TokenStream {
    let values = parse_macro_input!(input as syn::Expr);

    quote! {{
        for __mingling_render_item in (#values).into_iter() {
            ::mingling::render_stream::<crate::ThisProgram, _>(
                __mingling_render_item,
                ::mingling::RenderEmit::Auto,
            );
        }
    }}
    .into()
}
