// Doc Not Optimize
//! The `#[renderify]` extension — transforms `expr?` into `render_route!(expr)`.
//!
//! Designed as an extension for the Mingling attribute macro system, intended
//! to be used with `#[renderer(renderify)]`, `#[help(renderify)]`,
//! or standalone as `#[renderify]`.
//!
//! # How it works
//!
//! The macro parses the function AST and replaces every `Expr::Try` node with an
//! equivalent `render_route!(expr)` invocation, which routes errors to the
//! rendering pipeline via `crate::ThisProgram::render(AnyOutput::new(e))`.

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::visit_mut::VisitMut;
use syn::{Expr, ItemFn, parse_macro_input};

struct RenderifyTransform;

impl VisitMut for RenderifyTransform {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        syn::visit_mut::visit_expr_mut(self, expr);

        if let Expr::Try(try_expr) = expr {
            let inner = &*try_expr.expr;
            let inner_tokens = inner.to_token_stream();

            // Set the span of the generated `render_route` ident to the `?` token's span,
            // so that rust-analyzer resolves the `?` position to the `render_route!` macro
            // instead of the standard Try trait, showing the render_route macro's docs on hover.
            let q_span = try_expr.question_token.span();
            let route_ident = proc_macro2::Ident::new("render_route", q_span);

            if let Ok(macro_expr) = syn::parse2::<Expr>(quote::quote! {
                ::mingling::macros::#route_ident!(#inner_tokens)
            }) {
                *expr = macro_expr;
            }
        }
    }
}

pub(crate) fn renderify_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input_fn = parse_macro_input!(item as ItemFn);
    RenderifyTransform.visit_item_fn_mut(&mut input_fn);
    input_fn.to_token_stream().into()
}
