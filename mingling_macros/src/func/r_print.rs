use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, Token};

/// Parsed input for `r_println!` and `r_print!`.
///
/// Two forms:
/// - `(ident, format_args...)`       — explicit buffer
/// - `(format_args...)`              — implicit `__render_result_buffer`
pub(crate) enum PrintInput {
    Explicit { dst: Ident, args: TokenStream2 },
    Implicit { args: TokenStream2 },
}

impl Parse for PrintInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Peek: if the next token is an ident followed by a comma, it's the explicit form
        if input.peek(Ident) && input.peek2(Token![,]) {
            let dst: Ident = input.parse()?;
            let _comma: Token![,] = input.parse()?;
            let args: TokenStream2 = input.parse()?;
            Ok(Self::Explicit { dst, args })
        } else {
            let args: TokenStream2 = input.parse()?;
            Ok(Self::Implicit { args })
        }
    }
}

pub(crate) fn expand_print(input: TokenStream, method: &str) -> TokenStream {
    let parsed: PrintInput = match syn::parse(input) {
        Ok(p) => p,
        Err(e) => return e.to_compile_error().into(),
    };

    let method_ident = Ident::new(method, proc_macro2::Span::call_site());

    let expanded = match parsed {
        PrintInput::Explicit { dst, args } => {
            quote! {
                #dst.#method_ident(format!(#args))
            }
        }
        PrintInput::Implicit { args } => {
            quote! {
                __render_result_buffer.#method_ident(format!(#args))
            }
        }
    };

    expanded.into()
}

pub(crate) fn r_print(input: TokenStream) -> TokenStream {
    expand_print(input, "print")
}

/// Parsed input for `r_append!`.
///
/// Two forms:
/// - `(dst, src)`       — explicit buffer and source
/// - `(src)`            — implicit `__render_result_buffer`
pub(crate) struct AppendInput {
    pub(crate) dst: Option<Ident>,
    pub(crate) src: proc_macro2::TokenStream,
}

impl Parse for AppendInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Ident) && input.peek2(Token![,]) {
            let dst: Ident = input.parse()?;
            let _comma: Token![,] = input.parse()?;
            let src: TokenStream2 = input.parse()?;
            Ok(Self {
                dst: Some(dst),
                src,
            })
        } else {
            let src: TokenStream2 = input.parse()?;
            Ok(Self { dst: None, src })
        }
    }
}
