// Doc Not Optimize
use proc_macro::TokenStream;
use quote::quote;

use crate::CHAINS;
use crate::CHAINS_EXIST;
use crate::COMPILE_TIME_DISPATCHERS;
#[cfg(feature = "comp")]
use crate::COMPLETIONS;
use crate::HELP_REQUESTS;
use crate::METADATA;
use crate::PACKED_TYPES;
use crate::RENDERERS;
use crate::RENDERERS_EXIST;
#[cfg(feature = "structural_renderer")]
use crate::STRUCTURAL_RENDERERS;
use crate::get_global_set;
#[cfg(not(feature = "dispatch_tree"))]
use crate::systems::dispatch_list_gen;
#[cfg(feature = "dispatch_tree")]
use crate::systems::dispatch_tree_gen;

#[cfg(feature = "async")]
const ASYNC_ENABLED: bool = true;
#[cfg(not(feature = "async"))]
const ASYNC_ENABLED: bool = false;

/// Generate the `get_nodes()` function body for a `ProgramCollect` impl.
///
/// Shared by both dispatch strategies (trie and linear list); it only depends
/// on the compile-time-collected `__internal_dispatcher_*` statics.
fn gen_get_nodes(entries: &[(String, String, String)]) -> proc_macro2::TokenStream {
    let mut node_entries = Vec::new();

    for (node_name, _disp_type, _entry_name) in entries {
        let static_name_str = format!("__internal_dispatcher_{}", just_fmt::snake_case!(node_name));
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

/// Parses an entry of the format `StructName => EnumVariant,` into a pair of idents.
fn parse_entry_pair(entry: &proc_macro2::TokenStream) -> (proc_macro2::Ident, proc_macro2::Ident) {
    let s = entry.to_string();
    let arrow_idx = s
        .find("=>")
        .unwrap_or_else(|| panic!("Entry missing '=>': {s}"));
    let struct_str = s[..arrow_idx].trim();
    let variant_str = s[arrow_idx + 2..].trim().trim_end_matches(',').trim();
    let struct_ident = proc_macro2::Ident::new(struct_str, proc_macro2::Span::call_site());
    let variant_ident = proc_macro2::Ident::new(variant_str, proc_macro2::Span::call_site());
    (struct_ident, variant_ident)
}

/// Helper: convert a string ident into a token stream for the generated code.
/// Types are expected to be in scope (e.g. via pathf glob re-exports), so bare
/// idents suffice.
fn ident_tokens(name: &str) -> proc_macro2::TokenStream {
    let ident = proc_macro2::Ident::new(name, proc_macro2::Span::call_site());
    quote! { #ident }
}

#[allow(clippy::too_many_lines)]
#[allow(clippy::similar_names)] // You're being quite picky.
pub(crate) fn program_final_gen_impl(_input: TokenStream) -> TokenStream {
    let name = syn::Ident::new("ThisProgram", proc_macro2::Span::call_site());

    let packed_types = get_global_set(&PACKED_TYPES).lock().unwrap().clone();

    let renderers = get_global_set(&RENDERERS).lock().unwrap().clone();
    let chains = get_global_set(&CHAINS).lock().unwrap().clone();
    let renderer_exist = get_global_set(&RENDERERS_EXIST).lock().unwrap().clone();
    let chain_exist = get_global_set(&CHAINS_EXIST).lock().unwrap().clone();

    #[cfg(feature = "structural_renderer")]
    let structural_renderers = get_global_set(&STRUCTURAL_RENDERERS)
        .lock()
        .unwrap()
        .clone();

    #[cfg(feature = "comp")]
    let completions = get_global_set(&COMPLETIONS).lock().unwrap().clone();

    let packed_types: Vec<proc_macro2::TokenStream> = packed_types
        .iter()
        .map(|s| syn::parse_str::<proc_macro2::TokenStream>(s).unwrap())
        .collect();

    let renderer_tokens: Vec<proc_macro2::TokenStream> = renderers
        .iter()
        .map(|s| syn::parse_str::<proc_macro2::TokenStream>(s).unwrap())
        .collect();

    let chain_tokens: Vec<proc_macro2::TokenStream> = chains
        .iter()
        .map(|s| syn::parse_str::<proc_macro2::TokenStream>(s).unwrap())
        .collect();

    let renderer_exist_tokens: Vec<proc_macro2::TokenStream> = renderer_exist
        .iter()
        .map(|s| syn::parse_str::<proc_macro2::TokenStream>(s).unwrap())
        .collect();

    let chain_exist_tokens: Vec<proc_macro2::TokenStream> = chain_exist
        .iter()
        .map(|s| syn::parse_str::<proc_macro2::TokenStream>(s).unwrap())
        .collect();

    #[cfg(feature = "structural_renderer")]
    let structural_renderer_tokens: Vec<proc_macro2::TokenStream> = structural_renderers
        .iter()
        .map(|s| syn::parse_str::<proc_macro2::TokenStream>(s).unwrap())
        .collect();

    #[cfg(feature = "structural_renderer")]
    let structural_render = quote! {
        fn structural_render(
            any: ::mingling::AnyOutput<Self::Enum>,
            setting: &::mingling::config::StructuralRendererSetting,
        ) -> Result<::mingling::RenderResult, ::mingling::error::StructuralRendererSerializeError> {
            match any.member_id() {
                #(#structural_renderer_tokens)*
                _ => {
                    let mut r = ::mingling::RenderResult::default();
                    ::mingling::StructuralRenderer::render(&ResultEmpty, setting, &mut r)?;
                    Ok(r)
                }
            }
        }
    };

    #[cfg(not(feature = "structural_renderer"))]
    let structural_render = quote! {};

    let compile_time_dispatchers: Vec<String> = get_global_set(&COMPILE_TIME_DISPATCHERS)
        .lock()
        .unwrap()
        .clone()
        .iter()
        .cloned()
        .collect();

    let entries: Vec<(String, String, String)> = compile_time_dispatchers
        .iter()
        .filter_map(|entry| {
            let parts: Vec<&str> = entry.split(':').collect();
            if parts.len() == 3 {
                Some((
                    parts[0].to_string(),
                    parts[1].to_string(),
                    parts[2].to_string(),
                ))
            } else {
                None
            }
        })
        .collect();

    // The `dispatch_tree` feature only selects the internal matching strategy:
    // a char-level trie when enabled, a linear longest-prefix list otherwise.
    #[cfg(feature = "dispatch_tree")]
    let dispatch_gen = {
        let get_nodes_fn = gen_get_nodes(&entries);
        let dispatch_fn = dispatch_tree_gen::gen_dispatch_args_trie(&entries);

        quote! {
            #get_nodes_fn
            #dispatch_fn
        }
    };

    #[cfg(not(feature = "dispatch_tree"))]
    let dispatch_gen = {
        let get_nodes_fn = gen_get_nodes(&entries);
        let dispatch_fn = dispatch_list_gen::gen_dispatch_args(&entries);

        quote! {
            #get_nodes_fn
            #dispatch_fn
        }
    };

    #[cfg(feature = "comp")]
    let completion_tokens: Vec<proc_macro2::TokenStream> = completions
        .iter()
        .map(|s| syn::parse_str::<proc_macro2::TokenStream>(s).unwrap())
        .collect();

    #[cfg(feature = "comp")]
    let comp = quote! {
        fn do_comp(any: &::mingling::AnyOutput<Self::Enum>, ctx: &::mingling::ShellContext) -> ::mingling::Suggest {
            match any.member_id() {
                #(#completion_tokens)*
                _ => ::mingling::Suggest::FileCompletion,
            }
        }
    };

    #[cfg(not(feature = "comp"))]
    let comp = quote! {};

    // Build render function arms from stored entries
    let render_fn =
        if renderer_tokens.is_empty() {
            quote! {
                fn render(_any: ::mingling::AnyOutput<Self::Enum>) -> ::mingling::RenderResult {
                    ::mingling::RenderResult::default()
                }
            }
        } else {
            let render_arms: Vec<_> = renderer_tokens.iter().map(|entry| {
            let (struct_ident, variant_ident) = parse_entry_pair(entry);
            let downcast_ty = ident_tokens(&variant_ident.to_string());
            let resolved_struct = ident_tokens(&struct_ident.to_string());
            quote! {
                Self::#variant_ident => {
                    let value = unsafe { any.downcast::<#downcast_ty>().unwrap_unchecked() };
                    <#resolved_struct as ::mingling::Renderer>::render(value)
                }
            }
        }).collect();
            quote! {
                fn render(any: ::mingling::AnyOutput<Self::Enum>) -> ::mingling::RenderResult {
                    match any.member_id() {
                        #(#render_arms)*
                        _ => ::mingling::RenderResult::default(),
                    }
                }
            }
        };

    // Build do_chain function (async and sync versions)
    let chain_arms_async: Vec<_> = chain_tokens.iter().map(|entry| {
        let (struct_ident, variant_ident) = parse_entry_pair(entry);
        let downcast_ty = ident_tokens(&variant_ident.to_string());
        let resolved_struct = ident_tokens(&struct_ident.to_string());
        quote! {
            Self::#variant_ident => {
                let value = unsafe { any.downcast::<#downcast_ty>().unwrap_unchecked() };
                let fut = async { <#resolved_struct as ::mingling::Chain<Self::Enum>>::proc(value).await };
                ::std::boxed::Box::pin(fut)
            }
        }
    }).collect();

    let chain_arms_sync: Vec<_> = chain_tokens
        .iter()
        .map(|entry| {
            let (struct_ident, variant_ident) = parse_entry_pair(entry);
            let downcast_ty = ident_tokens(&variant_ident.to_string());
            let resolved_struct = ident_tokens(&struct_ident.to_string());
            quote! {
                Self::#variant_ident => {
                    let value = unsafe { any.downcast::<#downcast_ty>().unwrap_unchecked() };
                    <#resolved_struct as ::mingling::Chain<Self::Enum>>::proc(value)
                }
            }
        })
        .collect();

    let do_chain_fn = if chain_tokens.is_empty() {
        // An empty chain list is still valid, but the synthesized `do_chain`
        // must match the trait signature for the enabled mode. The sync
        // branch used unconditionally breaks the `async` feature (E0053),
        // so dispatch on `ASYNC_ENABLED` here as well.
        if ASYNC_ENABLED {
            quote! {
                fn do_chain(
                    _any: ::mingling::AnyOutput<Self::Enum>,
                ) -> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ::mingling::ChainProcess<Self::Enum>> + ::std::marker::Send>> {
                    ::std::boxed::Box::pin(async {
                        ::core::panic!("No chain found for type id")
                    })
                }
            }
        } else {
            quote! {
                fn do_chain(_any: ::mingling::AnyOutput<Self::Enum>) -> ::mingling::ChainProcess<Self::Enum> {
                    ::core::panic!("No chain found for type id")
                }
            }
        }
    } else if ASYNC_ENABLED {
        quote! {
            fn do_chain(
                any: ::mingling::AnyOutput<Self::Enum>,
            ) -> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ::mingling::ChainProcess<Self::Enum>> + ::std::marker::Send>> {
                match any.member_id() {
                    #(#chain_arms_async)*
                    _ => ::core::panic!("No chain found for type id: {:?}", any.type_id()),
                }
            }
        }
    } else {
        quote! {
            fn do_chain(
                any: ::mingling::AnyOutput<Self::Enum>,
            ) -> ::mingling::ChainProcess<Self::Enum> {
                match any.member_id() {
                    #(#chain_arms_sync)*
                    _ => ::core::panic!("No chain found for type id: {:?}", any.type_id()),
                }
            }
        }
    };

    let help_tokens: Vec<proc_macro2::TokenStream> = get_global_set(&HELP_REQUESTS)
        .lock()
        .unwrap()
        .clone()
        .iter()
        .map(|s| syn::parse_str::<proc_macro2::TokenStream>(s).unwrap())
        .collect();

    let metadata_tokens: Vec<proc_macro2::TokenStream> = get_global_set(&METADATA)
        .lock()
        .unwrap()
        .clone()
        .iter()
        .map(|s| syn::parse_str::<proc_macro2::TokenStream>(s).unwrap())
        .collect();

    let get_metadata_fn = if metadata_tokens.is_empty() {
        quote! {
            fn get_metadata<T: 'static>(_member_id: Self::Enum) -> Option<T> {
                None
            }
        }
    } else {
        let metadata_arms = metadata_tokens.iter().map(|entry| {
            quote! {
                #entry
            }
        });
        quote! {
            fn get_metadata<T: 'static>(member_id: Self::Enum) -> Option<T> {
                let type_id = ::std::any::TypeId::of::<T>();
                let any = match member_id {
                    #(#metadata_arms)*
                    _ => None,
                };
                any.and_then(|b| b.downcast::<T>().ok().map(|b| *b))
            }
        }
    };

    let num_variants = packed_types.len();
    let repr_type = if u8::try_from(num_variants).is_ok() {
        quote! { u8 }
    } else if u16::try_from(num_variants).is_ok() {
        quote! { u16 }
    } else if u32::try_from(num_variants).is_ok() {
        quote! { u32 }
    } else {
        quote! { u128 }
    };

    let expanded = quote! {
        #[derive(Debug, PartialEq, Eq, Clone, Copy)]
        #[repr(#repr_type)]
        #[allow(nonstandard_style)]
        pub enum #name {
            #(#packed_types),*
        }

        impl ::std::fmt::Display for #name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                match self {
                    #(#name::#packed_types => write!(f, stringify!(#packed_types)),)*
                }
            }
        }

        impl ::mingling::ProgramCollect for #name {
            type Enum = #name;
            type EntryFallback = EntryFallback;
            type ErrorRendererNotFound = ErrorRendererNotFound;
            type ResultEmpty = ResultEmpty;

            fn build_renderer_not_found(member_id: Self::Enum) -> ::mingling::AnyOutput<Self::Enum> {
                ::mingling::AnyOutput::new(ErrorRendererNotFound(member_id.to_string()))
            }
            fn build_entry_fallback(args: Vec<String>) -> ::mingling::AnyOutput<Self::Enum> {
                ::mingling::AnyOutput::new(EntryFallback(args))
            }
            fn build_empty_result() -> ::mingling::AnyOutput<Self::Enum> {
                ::mingling::AnyOutput::new(ResultEmpty)
            }
            #render_fn
            #do_chain_fn
            #get_metadata_fn
            fn render_help(any: ::mingling::AnyOutput<Self::Enum>) -> ::mingling::RenderResult {
                match any.member_id() {
                    #(#help_tokens)*
                    _ => ::mingling::RenderResult::default(),
                }
            }
            fn has_renderer(any: &::mingling::AnyOutput<Self::Enum>) -> bool {
                match any.member_id() {
                    #(#renderer_exist_tokens)*
                    _ => false
                }
            }
            fn has_chain(any: &::mingling::AnyOutput<Self::Enum>) -> bool {
                match any.member_id() {
                    #(#chain_exist_tokens)*
                    _ => false
                }
            }
            #dispatch_gen
            #structural_render
            #comp
        }

        impl #name {
            /// Creates a new `Program<#name>` instance with default configuration.
            pub fn new() -> ::mingling::Program<#name> {
                ::mingling::Program::new()
            }

            /// Returns a static reference to the global `Program<#name>` singleton.
            pub fn this() -> &'static ::mingling::Program<#name> {
                &::mingling::this::<#name>()
            }
        }
    };

    // Clear all global registries to prevent stale state in Rust Analyzer
    get_global_set(&PACKED_TYPES).lock().unwrap().clear();
    get_global_set(&CHAINS).lock().unwrap().clear();
    get_global_set(&CHAINS_EXIST).lock().unwrap().clear();
    get_global_set(&RENDERERS).lock().unwrap().clear();
    get_global_set(&RENDERERS_EXIST).lock().unwrap().clear();
    get_global_set(&HELP_REQUESTS).lock().unwrap().clear();
    get_global_set(&METADATA).lock().unwrap().clear();
    #[cfg(feature = "comp")]
    get_global_set(&COMPLETIONS).lock().unwrap().clear();
    get_global_set(&COMPILE_TIME_DISPATCHERS)
        .lock()
        .unwrap()
        .clear();
    #[cfg(feature = "structural_renderer")]
    get_global_set(&STRUCTURAL_RENDERERS)
        .lock()
        .unwrap()
        .clear();

    TokenStream::from(expanded)
}
