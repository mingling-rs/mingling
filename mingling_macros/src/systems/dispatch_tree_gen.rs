//! Char-level trie dispatch generator (`dispatch_tree` feature).
//!
//! Builds a hardcoded match tree: at each depth, group nodes by character.
//! Single-node groups use `starts_with`; multi-node groups recurse with
//! `nth()` match.
//!
//! The "longest registered prefix" fallback (try the exact endpoint at this
//! node, then its parent, …) is **not** inlined into every arm. Instead each
//! trie node gets an id and every arm *calls* a single generic
//! `__trie_fallback<G>` method that runs that node's exact-endpoint checks
//! and tail-recurses to the parent, returning `None` when nothing in the
//! chain matches (the caller then produces the no-match result). This keeps
//! the generated code linear in the table size: inlining the whole fallback
//! chain per arm grew quadratically with nesting depth (a 1024×16 nested
//! table emitted ~13 MB of tokens).
//!
//! The generator returns two token streams: the `dispatch_args` method (for
//! the `ProgramCollect` trait impl) and the `__trie_fallback` method (for an
//! inherent impl of the program type). The fallback uses the concrete `Self`
//! (the program type implements `ProgramCollect` with `Enum = Self`, and the
//! bench harness's `BenchDispatch` mirrors that), so it lives outside the
//! trait.

use std::collections::BTreeMap;

use proc_macro2::TokenStream;
use quote::quote;

/// A trie node recorded for the shared longest-prefix fallback: its
/// exact-endpoint checks and its parent's node id.
struct FallbackNode {
    node_id: usize,
    parent: Option<usize>,
    exact_checks: Vec<TokenStream>,
}

/// Emit a `starts_with` dispatch arm.
///
/// `group_ty` is the type the dispatchers are generic over: `Self::Enum`
/// inside `dispatch_args` (trait impl) or `Self` inside the fallback
/// (inherent impl). `wrap_some` selects `return Some(match __cp …)`
/// (fallback) vs `return match __cp …` (dispatch).
fn make_starts_with_arm(
    name: &str,
    disp_type: &str,
    group_ty: &TokenStream,
    wrap_some: bool,
) -> TokenStream {
    let name_space = format!("{name} ");
    let name_lit = syn::LitStr::new(&name_space, proc_macro2::Span::call_site());
    let disp_ident = proc_macro2::Ident::new(disp_type, proc_macro2::Span::call_site());
    let prefix_word_count = name.split_whitespace().count();
    let ret = if wrap_some {
        quote! {
            return Some(match __cp {
                ::mingling::ChainProcess::Ok(any_output) => Ok(any_output.0),
                ::mingling::ChainProcess::Err(chain_process_error) => {
                    Err(chain_process_error.into())
                }
            });
        }
    } else {
        quote! {
            return match __cp {
                ::mingling::ChainProcess::Ok(any_output) => Ok(any_output.0),
                ::mingling::ChainProcess::Err(chain_process_error) => {
                    Err(chain_process_error.into())
                }
            };
        }
    };
    quote! {
        if raw_str.starts_with(#name_lit) {
            let prefix_len = #prefix_word_count;
            let trimmed_args: Vec<String> = raw.iter().skip(prefix_len).cloned().collect();
            let __cp = <#disp_ident as ::mingling::Dispatcher<#group_ty>>::begin(
                &#disp_ident::default(),
                trimmed_args,
            );
            #ret
        }
    }
}

/// Call site of the shared fallback from inside `dispatch_args`: if the
/// fallback chain resolved an exact endpoint, return it; otherwise fall
/// through (the terminal no-match result follows the root match).
fn fallback_call(node_id: usize) -> TokenStream {
    let id_lit = proc_macro2::Literal::usize_unsuffixed(node_id);
    quote! {
        if let Some(__r) = Self::__trie_fallback(raw, raw_str, #id_lit) {
            return __r;
        }
    }
}

/// Recursively build the trie match body.
///
/// `nodes`: slice of (`display_name`, `disp_type`) for commands that share the
/// same prefix so far. `depth`: the character index currently being matched.
/// `node_id` / `parent_id`: trie node identity used by the shared
/// longest-prefix fallback; `fallbacks` collects one entry per interior node
/// for the `__trie_fallback` method emitted by the caller.
///
/// Matching follows the same "longest registered prefix" rule used by the
/// other generators: a child (longer) path is preferred over an exact
/// endpoint at the same depth. Only when every descendant fails to match is
/// the exact endpoint here dispatched (via the fallback).
fn build_dispatch_body(
    nodes: &[(String, String)],
    depth: usize,
    node_id: usize,
    parent_id: Option<usize>,
    next_id: &mut usize,
    fallbacks: &mut Vec<FallbackNode>,
) -> TokenStream {
    if nodes.is_empty() {
        return parent_id.map_or_else(|| quote! {}, fallback_call);
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

    // Register this node in the fallback table (interior nodes only — leaf
    // exact checks run inline in the walk). The fallback's exact checks use
    // the concrete `Self` (see the module docs) and wrap results in `Some`.
    if !groups.is_empty() {
        fallbacks.push(FallbackNode {
            node_id,
            parent: parent_id,
            exact_checks: exact_nodes
                .iter()
                .map(|(name, disp_type)| make_starts_with_arm(name, disp_type, &quote!(Self), true))
                .collect(),
        });
    }

    let self_enum = quote!(Self::Enum);
    let mut arms = Vec::new();

    for (&ch, sub_nodes) in &groups {
        let ch_char = ch;

        if sub_nodes.len() == 1 {
            let (name, disp_type) = &sub_nodes[0];
            let arm = make_starts_with_arm(name, disp_type, &self_enum, false);
            let fb = fallback_call(node_id);
            // Try the child first; if it does not match, defer to this node's
            // longest-prefix fallback so the longer path wins when present.
            arms.push(quote! {
                Some(#ch_char) => {
                    #arm
                    #fb
                }
            });
        } else {
            let child_id = *next_id;
            *next_id += 1;
            let sub_body = build_dispatch_body(
                sub_nodes,
                depth + 1,
                child_id,
                Some(node_id),
                next_id,
                fallbacks,
            );
            arms.push(quote! {
                Some(#ch_char) => {
                    #sub_body
                }
            });
        }
    }

    if groups.is_empty() {
        // No children exist for this node; only the exact endpoint(s) apply,
        // then defer to the parent's fallback (longest-prefix semantics).
        let mut body = exact_nodes
            .iter()
            .map(|(name, disp_type)| make_starts_with_arm(name, disp_type, &self_enum, false))
            .collect::<Vec<TokenStream>>();
        if let Some(p) = parent_id {
            body.push(fallback_call(p));
        }
        quote! { #(#body)* }
    } else {
        let fb = fallback_call(node_id);
        quote! {
            match raw_chars.nth(0) {
                #(#arms)*
                _ => {
                    #fb
                }
            }
        }
    }
}

/// Generate the `dispatch_args()` method (for the `ProgramCollect` trait
/// impl) plus the `__trie_fallback` method (for an inherent impl of the
/// program type). Returns `(dispatch_method, extra_inherent_items)`.
pub(crate) fn gen_dispatch_args_trie(
    entries: &[(String, String, String)],
) -> (TokenStream, TokenStream) {
    let nodes: Vec<(String, String)> = entries
        .iter()
        .map(|(name, disp, _)| (name.replace('.', " "), disp.clone()))
        .collect();

    let mut next_id = 1usize;
    let mut fallbacks: Vec<FallbackNode> = Vec::new();
    let dispatch_body = build_dispatch_body(&nodes, 0, 0, None, &mut next_id, &mut fallbacks);

    let fallback_arms: Vec<TokenStream> = fallbacks
        .iter()
        .map(|fb| {
            let id_lit = proc_macro2::Literal::usize_unsuffixed(fb.node_id);
            let exact = &fb.exact_checks;
            let tail = fb.parent.map_or_else(
                || quote! { None },
                |p| {
                    let p_lit = proc_macro2::Literal::usize_unsuffixed(p);
                    quote! { Self::__trie_fallback(raw, raw_str, #p_lit) }
                },
            );
            quote! {
                #id_lit => {
                    #(#exact)*
                    #tail
                }
            }
        })
        .collect();

    let dispatch_fn = quote! {
        fn dispatch_args(
            raw: &[String],
        ) -> Result<::mingling::AnyOutput<Self::Enum>, ::mingling::error::ProgramInternalExecuteError>
        {
            let raw_string = format!("{} ", raw.join(" "));
            let raw_str = raw_string.as_str();
            let mut raw_chars = raw_str.chars();
            #dispatch_body
            Ok(Self::build_entry_fallback(raw.to_vec()))
        }
    };

    let fallback_fn = quote! {
        /// Longest-prefix fallback: run the exact-endpoint checks of trie
        /// node `__node`, then defer to its parent (tail-recursively), or
        /// return `None` at the root when nothing matched. Shared by all
        /// arms instead of being inlined into each one, keeping the generated
        /// code linear in the table size. Lives in an inherent impl, where
        /// `Self` is the program type (`ProgramCollect`'s `Enum = Self`).
        #[allow(dead_code)]
        #[inline(never)]
        fn __trie_fallback(
            raw: &[String],
            raw_str: &str,
            __node: usize,
        ) -> Option<Result<
            ::mingling::AnyOutput<Self>,
            ::mingling::error::ProgramInternalExecuteError,
        >> {
            match __node {
                #(#fallback_arms)*
                _ => None,
            }
        }
    };

    (dispatch_fn, fallback_fn)
}
