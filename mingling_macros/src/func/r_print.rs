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
enum PrintInput {
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
            Ok(PrintInput::Explicit { dst, args })
        } else {
            let args: TokenStream2 = input.parse()?;
            Ok(PrintInput::Implicit { args })
        }
    }
}

fn expand_print(input: TokenStream, method: &str) -> TokenStream {
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

pub(crate) fn r_println(input: TokenStream) -> TokenStream {
    expand_print(input, "println")
}

pub(crate) fn r_print(input: TokenStream) -> TokenStream {
    expand_print(input, "print")
}

pub(crate) fn r_eprintln(input: TokenStream) -> TokenStream {
    expand_print(input, "eprintln")
}

pub(crate) fn r_eprint(input: TokenStream) -> TokenStream {
    expand_print(input, "eprint")
}

/// Parsed input for `r_append!`.
///
/// Two forms:
/// - `(dst, src)`       — explicit buffer and source
/// - `(src)`            — implicit `__render_result_buffer`
struct AppendInput {
    dst: Option<Ident>,
    src: proc_macro2::TokenStream,
}

impl Parse for AppendInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Ident) && input.peek2(Token![,]) {
            let dst: Ident = input.parse()?;
            let _comma: Token![,] = input.parse()?;
            let src: TokenStream2 = input.parse()?;
            Ok(AppendInput {
                dst: Some(dst),
                src,
            })
        } else {
            let src: TokenStream2 = input.parse()?;
            Ok(AppendInput { dst: None, src })
        }
    }
}

/// `r_append!` macro: appends the contents of another `RenderResult` to this one.
///
/// Two forms:
/// - `r_append!(dst, src)` — appends `src` into `dst`
/// - `r_append!(src)`      — appends `src` into `__render_result_buffer`
pub(crate) fn r_append(input: TokenStream) -> TokenStream {
    let parsed: AppendInput = match syn::parse(input) {
        Ok(p) => p,
        Err(e) => return e.to_compile_error().into(),
    };

    let dst_ident = parsed.dst.clone();
    let src_tokens = parsed.src;

    let expanded = match dst_ident {
        Some(dst) => {
            quote! {
                #dst.append_other(#src_tokens);
            }
        }
        None => {
            quote! {
                __render_result_buffer.append_other(#src_tokens);
            }
        }
    };

    expanded.into()
}
