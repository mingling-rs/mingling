use proc_macro::TokenStream;
use quote::quote;

/// Entry point for `gen_program!()`.
///
/// Generates the `Next` type alias, `Routable` impl for `ChainProcess`,
/// and delegates to `program_comp_gen!()`, `program_fallback_gen!()`,
/// and `program_final_gen!()`.
pub(crate) fn gen_program_impl(_input: TokenStream) -> TokenStream {
    #[cfg(feature = "comp")]
    let comp_gen = quote! {
        ::mingling::macros::program_comp_gen!();
    };

    #[cfg(not(feature = "comp"))]
    let comp_gen = quote! {};

    TokenStream::from(quote! {
        /// Alias for the current program type `crate::ThisProgram`
        pub type Next = ::mingling::ChainProcess<crate::ThisProgram>;

        impl ::mingling::Routable<crate::ThisProgram> for ::mingling::ChainProcess<crate::ThisProgram>
        {
            fn to_chain(self) -> ::mingling::ChainProcess<crate::ThisProgram> {
                match self {
                    ::mingling::ChainProcess::Ok((any, _)) => {
                        ::mingling::ChainProcess::Ok((any, mingling::NextProcess::Chain))
                    }
                    other => other,
                }
            }

            fn to_render(self) -> ::mingling::ChainProcess<crate::ThisProgram> {
                match self {
                    ::mingling::ChainProcess::Ok((any, _)) => {
                        ::mingling::ChainProcess::Ok((any, mingling::NextProcess::Renderer))
                    }
                    other => other,
                }
            }
        }

        #comp_gen
        ::mingling::macros::program_fallback_gen!();
        ::mingling::macros::program_final_gen!();
    })
}
