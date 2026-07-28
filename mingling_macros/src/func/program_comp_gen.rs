use proc_macro::TokenStream;
use quote::quote;

#[cfg(feature = "comp")]
pub(crate) fn program_comp_gen_impl(_input: TokenStream) -> TokenStream {
    #[cfg(feature = "async")]
    let fn_exec_comp = quote! {
        #[doc(hidden)]
        #[::mingling::macros::chain]
        pub async fn __exec_completion(prev: CompletionContext) -> Next {
            use ::mingling::Grouped;

            let read_ctx = ::mingling::ShellContext::try_from(prev.inner);
            match read_ctx {
                Ok(ctx) => {
                    let suggest = ::mingling::CompletionHelper::exec_completion::<crate::ThisProgram>(&ctx);
                    ::mingling::Routable::<crate::ThisProgram>::to_render(CompletionSuggest::new((ctx, suggest)))
                }
                Err(_) => std::process::exit(1),
            }
        }
    };

    #[cfg(not(feature = "async"))]
    let fn_exec_comp = quote! {
        #[doc(hidden)]
        #[::mingling::macros::chain]
        pub fn __exec_completion(prev: CompletionContext) -> Next {
            use ::mingling::Grouped;

            let read_ctx = ::mingling::ShellContext::try_from(prev.inner);
            match read_ctx {
                Ok(ctx) => {
                    let suggest = ::mingling::CompletionHelper::exec_completion::<crate::ThisProgram>(&ctx);
                    ::mingling::Routable::<crate::ThisProgram>::to_render(CompletionSuggest::new((ctx, suggest)))
                }
                Err(_) => std::process::exit(1),
            }
        }
    };

    #[cfg(feature = "dispatch_tree")]
    let internal_dispatcher_comp = quote! {
        use __internal_completion_mod::__internal_dispatcher_comp;
    };

    #[cfg(not(feature = "dispatch_tree"))]
    let internal_dispatcher_comp = quote! {};

    let comp_dispatcher = quote! {
        #[doc(hidden)]
        mod __internal_completion_mod {
            use ::mingling::Grouped;
            ::mingling::macros::dispatcher!("__comp", CMDCompletion => CompletionContext);
            ::mingling::macros::pack!(
                CompletionSuggest = (::mingling::ShellContext, ::mingling::Suggest)
            );
        }
        #internal_dispatcher_comp
        use __internal_completion_mod::CompletionContext;
        use __internal_completion_mod::CompletionSuggest;
        pub use __internal_completion_mod::CMDCompletion;

        #fn_exec_comp

        ::mingling::macros::register_type!(CompletionContext);

        #[allow(unused)]
        #[doc(hidden)]
        #[::mingling::macros::renderer]
        pub fn __render_completion(prev: CompletionSuggest) -> ::mingling::RenderResult {
            let result = ::mingling::RenderResult::default();
            let (ctx, suggest) = prev.inner;
            ::mingling::CompletionHelper::render_suggest::<crate::ThisProgram>(ctx, suggest);
            result
        }
    };

    TokenStream::from(comp_dispatcher)
}
