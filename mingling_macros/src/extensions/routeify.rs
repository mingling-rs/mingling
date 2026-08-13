// Doc Not Optimize
//! The `#[routeify]` extension — transforms `expr?` into `route!(expr)`.
//!
//! Designed as an extension for the Mingling attribute macro system, intended
//! to be used with `#[chain(routeify)]` or standalone as `#[routeify]`.
//!
//! # How it works
//!
//! The macro parses the function AST and replaces every `Expr::Try` node with an
//! equivalent `route!(expr)` invocation.

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::visit_mut::VisitMut;
use syn::{Expr, ItemFn, parse_macro_input};

struct RouteifyTransform;

impl VisitMut for RouteifyTransform {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        syn::visit_mut::visit_expr_mut(self, expr);

        if let Expr::Try(try_expr) = expr {
            let inner = &*try_expr.expr;
            let inner_tokens = inner.to_token_stream();

            // Set the span of the generated `route` ident to the `?` token's span,
            // so that rust-analyzer resolves the `?` position to the `route!` macro
            // instead of the standard Try trait, showing the route macro's docs on hover.
            let q_span = try_expr.question_token.span();
            let route_ident = proc_macro2::Ident::new("route", q_span);

            if let Ok(macro_expr) = syn::parse2::<Expr>(quote::quote! {
                ::mingling::macros::#route_ident!(#inner_tokens)
            }) {
                *expr = macro_expr;
            }
        }
    }
}

pub(crate) fn routeify_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input_fn = parse_macro_input!(item as ItemFn);
    RouteifyTransform.visit_item_fn_mut(&mut input_fn);
    input_fn.to_token_stream().into()
}
