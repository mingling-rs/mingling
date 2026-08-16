// Doc Not Optimize
use std::collections::BTreeMap;

use just_fmt::snake_case;
use proc_macro2::TokenStream;
use quote::quote;

/// Generate the `get_nodes()` function body for a ProgramCollect impl.
pub(crate) fn gen_get_nodes(entries: &[(String, String, String)]) -> TokenStream {
    let mut node_entries = Vec::new();

    for (node_name, _disp_type, _entry_name) in entries {
        let static_name_str = format!("__internal_dispatcher_{}", snake_case!(node_name));
        let static_ident =
            proc_macro2::Ident::new(&static_name_str, proc_macro2::Span::call_site());
        let node_display_name = node_name.replace('.', " ");
        let node_display_lit = syn::LitStr::new(&node_display_name, proc_macro2::Span::call_site());

        node_entries.push(quote! {
            (#node_display_lit.to_string(), &#static_ident)
        });
    }

    quote! {
        fn get_nodes() -> Vec<(String, &'static (dyn ::mingling::Dispatcher<Self::Enum> + Send + Sync))> {
            vec![
                #(#node_entries),*
            ]
        }
    }
}

/// Generate the `dispatch_args()` function body for a ProgramCollect impl.
///
/// Builds a hardcoded match tree: at each depth, group nodes by character.
/// Single-node groups use `starts_with`; multi-node groups recurse with `nth()` match.
pub(crate) fn gen_dispatch_args_trie(entries: &[(String, String, String)]) -> TokenStream {
    let nodes: Vec<(String, String)> = entries
        .iter()
        .map(|(name, disp, _)| (name.replace('.', " "), disp.clone()))
        .collect();

    let dispatch_body = build_dispatch_body(
        &nodes,
        0,
        &quote! {
            return Ok(Self::build_entry_fallback(raw.to_vec()));
        },
    );

    quote! {
        fn dispatch_args(
            raw: &[String],
        ) -> Result<::mingling::AnyOutput<Self::Enum>, ::mingling::error::ProgramInternalExecuteError>
        {
            let raw_string = format!("{} ", raw.join(" "));
            let raw_str = raw_string.as_str();
            let mut raw_chars = raw_str.chars();
            #dispatch_body
        }
    }
}

/// Recursively build the trie match body.
///
/// `nodes`: slice of (display_name, disp_type) for commands that share the same prefix so far.
/// `depth`: The character index currently being matched.
/// `no_match`: fallback code to run when no node in this subtree matches the input.
///
/// Matching follows the same "longest registered prefix" rule used by the
/// dynamic dispatcher: a child (longer) path is preferred over an exact
/// endpoint at the same depth. Only when every descendant fails to match is
/// the exact endpoint here dispatched.
fn build_dispatch_body(
    nodes: &[(String, String)],
    depth: usize,
    no_match: &TokenStream,
) -> TokenStream {
    if nodes.is_empty() {
        return no_match.clone();
    }

    let mut groups: BTreeMap<char, Vec<(String, String)>> = BTreeMap::new();
    let mut exact_nodes: Vec<(String, String)> = Vec::new();

    for (name, disp_type) in nodes {
        if let Some(ch) = name.chars().nth(depth) {
            groups
                .entry(ch)
                .or_default()
                .push((name.clone(), disp_type.clone()));
        } else {
            exact_nodes.push((name.clone(), disp_type.clone()));
        }
    }

    let make_starts_with_arm = |name: &str, disp_type: &str| -> TokenStream {
        let name_space = format!("{} ", name);
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
    };

    // Fallback code for when neither a child path nor the exact endpoint(s)
    // here match: run the exact endpoint checks for this node first (they must
    // win over nothing at all), then pass control back up to the caller.
    let exact_checks: Vec<TokenStream> = exact_nodes
        .iter()
        .map(|(name, disp_type)| make_starts_with_arm(name, disp_type))
        .collect();

    let level_no_match = {
        let mut body = exact_checks.clone();
        body.push(no_match.clone());
        quote! { #(#body)* }
    };

    let mut arms = Vec::new();

    for (&ch, sub_nodes) in &groups {
        let ch_char = ch;

        if sub_nodes.len() == 1 {
            let (name, disp_type) = &sub_nodes[0];
            let arm = make_starts_with_arm(name, disp_type);
            // Try the child first; if it does not match, fall through to the
            // exact endpoint(s) here so the longer path wins when present.
            arms.push(quote! {
                Some(#ch_char) => {
                    #arm
                    #level_no_match
                }
            });
        } else {
            let sub_body = build_dispatch_body(sub_nodes, depth + 1, &level_no_match);
            arms.push(quote! {
                Some(#ch_char) => {
                    #sub_body
                }
            });
        }
    }

    if groups.is_empty() {
        // No children exist for this node; only the exact endpoint(s) apply.
        let mut body = exact_checks;
        body.push(no_match.clone());
        quote! { #(#body)* }
    } else {
        quote! {
            match raw_chars.nth(0) {
                #(#arms)*
                _ => {
                    #level_no_match
                }
            }
        }
    }
}
