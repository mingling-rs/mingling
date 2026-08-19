use std::cmp::Reverse;

use proc_macro2::TokenStream;
use quote::quote;

/// Generate the `dispatch_args()` function body for a `ProgramCollect` impl
/// using linear matching over the compile-time-collected dispatchers.
///
/// Nodes are sorted by display-name length (longest first) so the first
/// matching node is the most specific one, mirroring the "longest registered
/// prefix wins" rule of the old dynamic dispatcher.
pub(crate) fn gen_dispatch_args(entries: &[(String, String, String)]) -> TokenStream {
    let mut nodes: Vec<(String, String)> = entries
        .iter()
        .map(|(name, disp, _)| (name.replace('.', " "), disp.clone()))
        .collect();
    nodes.sort_by_key(|(name, _)| Reverse(name.len()));

    let arms: Vec<TokenStream> = nodes
        .iter()
        .map(|(name, disp_type)| {
            let name_space = format!("{name} ");
            let name_lit = syn::LitStr::new(&name_space, proc_macro2::Span::call_site());
            let disp_ident = proc_macro2::Ident::new(disp_type, proc_macro2::Span::call_site());
            let prefix_word_count = name.split_whitespace().count();
            quote! {
                if raw_str.starts_with(#name_lit) {
                    let prefix_len = #prefix_word_count;
                    let trimmed_args: Vec<String> = raw.iter().skip(prefix_len).cloned().collect();
                    let __cp = <#disp_ident as ::mingling::Dispatcher<Self::Enum>>::begin(
                        &#disp_ident::default(),
                        trimmed_args,
                    );
                    return match __cp {
                        ::mingling::ChainProcess::Ok(any_output) => Ok(any_output.0),
                        ::mingling::ChainProcess::Err(chain_process_error) => {
                            Err(chain_process_error.into())
                        }
                    };
                }
            }
        })
        .collect();

    quote! {
        fn dispatch_args(
            raw: &[String],
        ) -> Result<::mingling::AnyOutput<Self::Enum>, ::mingling::error::ProgramInternalExecuteError>
        {
            let raw_string = format!("{} ", raw.join(" "));
            let raw_str = raw_string.as_str();
            #(#arms)*
            Ok(Self::build_entry_fallback(raw.to_vec()))
        }
    }
}
