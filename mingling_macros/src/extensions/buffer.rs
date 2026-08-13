// Doc Not Optimize
use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{ItemFn, ReturnType, Type, parse_macro_input};

/// Checks whether the return type is unit `()`.
fn is_unit_return_type(sig: &syn::Signature) -> bool {
    match &sig.output {
        ReturnType::Type(_, ty) => match &**ty {
            Type::Tuple(tuple) => tuple.elems.is_empty(),
            _ => false,
        },
        ReturnType::Default => true,
    }
}

/// The `#[buffer]` attribute macro.
///
/// Wraps a unit-returning function to produce a `::mingling::RenderResult` by
/// injecting a local `__render_result_buffer` variable. Inside the function
/// body, the `r_print!` / `r_println!` macros can write into the buffer.
///
/// # Example
///
/// ```rust,ignore
/// use mingling::macros::{buffer, r_println};
///
/// #[buffer]
/// fn render_greeting(prev: Greeting) {
///     r_println!("Hello, {}!", *prev);
/// }
/// ```
///
/// Expands to:
///
/// ```rust,ignore
/// fn render_greeting(prev: Greeting) -> mingling::RenderResult {
///     let mut __render_result_buffer = mingling::RenderResult::new();
///     {
///         r_println!("Hello, {}!", *prev);
///     }
///     __render_result_buffer
/// }
/// ```
pub(crate) fn buffer_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Reject non-empty attribute arguments; #[buffer] must be bare
    if !attr.is_empty() {
        return syn::Error::new(
            attr.into_iter().next().unwrap().span().into(),
            "#[buffer] does not accept arguments",
        )
        .to_compile_error()
        .into();
    }

    // Parse the function item
    let input_fn = parse_macro_input!(item as ItemFn);

    // Validate the function is not async
    if input_fn.sig.asyncness.is_some() {
        return syn::Error::new(input_fn.sig.span(), "Buffer function cannot be async")
            .to_compile_error()
            .into();
    }

    // Validate return type is unit
    if !is_unit_return_type(&input_fn.sig) {
        return syn::Error::new(
            input_fn.sig.span(),
            "#[buffer] function must not have a return value (must return `()`)",
        )
        .to_compile_error()
        .into();
    }

    // Get function attributes (excluding the buffer attribute)
    let mut fn_attrs = input_fn.attrs.clone();
    fn_attrs.retain(|attr| !attr.path().is_ident("buffer"));

    let vis = &input_fn.vis;
    let fn_name = &input_fn.sig.ident;
    let inputs = &input_fn.sig.inputs;
    let fn_body = &input_fn.block;

    let expanded = quote! {
        #(#fn_attrs)*
        #vis fn #fn_name(#inputs) -> ::mingling::RenderResult {
            let mut __render_result_buffer = ::mingling::RenderResult::new();
            #fn_body
            __render_result_buffer
        }
    };

    expanded.into()
}
