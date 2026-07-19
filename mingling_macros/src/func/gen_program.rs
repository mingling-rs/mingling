use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

use crate::CHAINS;
use crate::CHAINS_EXIST;
#[cfg(feature = "dispatch_tree")]
use crate::COMPILE_TIME_DISPATCHERS;
#[cfg(feature = "comp")]
use crate::COMPLETIONS;
use crate::HELP_REQUESTS;
use crate::PACKED_TYPES;
use crate::RENDERERS;
use crate::RENDERERS_EXIST;
#[cfg(feature = "structural_renderer")]
use crate::STRUCTURAL_RENDERERS;
use crate::attr::{chain, renderer};
use crate::get_global_set;
#[cfg(feature = "dispatch_tree")]
use crate::systems::dispatch_tree_gen;

#[cfg(feature = "async")]
const ASYNC_ENABLED: bool = true;
#[cfg(not(feature = "async"))]
const ASYNC_ENABLED: bool = false;

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

/// Loads the pathf type mapping from `$OUT_DIR/{crate}/type_using.rs`.
/// Always compiled; returns empty map when pathf feature is not enabled.
fn load_pathf_map() -> std::collections::HashMap<String, String> {
    if !cfg!(feature = "pathf") {
        return std::collections::HashMap::new();
    }
    let out_dir = std::env::var("OUT_DIR").ok();
    let crate_name = std::env::var("CARGO_PKG_NAME").ok();
    match (out_dir, crate_name) {
        (Some(dir), Some(name)) => {
            let path = std::path::Path::new(&dir).join(&name).join("type_using.rs");
            match std::fs::read_to_string(&path) {
                Ok(content) => content
                    .lines()
                    .filter_map(|line| {
                        let line = line.trim();
                        if let Some(rest) = line.strip_prefix("use ") {
                            let path = rest.strip_suffix(';').unwrap_or(rest);
                            if let Some((_mod, type_name)) = path.rsplit_once("::") {
                                return Some((type_name.to_string(), path.to_string()));
                            }
                        }
                        None
                    })
                    .collect(),
                Err(_) => std::collections::HashMap::new(),
            }
        }
        _ => std::collections::HashMap::new(),
    }
}

/// Resolves a type name to its full path token stream using the pathf mapping.
pub(crate) fn resolve_type(
    name: &str,
    map: &std::collections::HashMap<String, String>,
) -> proc_macro2::TokenStream {
    if let Some(full_path) = map.get(name) {
        syn::parse_str::<proc_macro2::TokenStream>(full_path).unwrap_or_else(|_| {
            let ident = proc_macro2::Ident::new(name, proc_macro2::Span::call_site());
            quote! { #ident }
        })
    } else {
        let ident = proc_macro2::Ident::new(name, proc_macro2::Span::call_site());
        quote! { #ident }
    }
}

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
                    crate::CompletionSuggest::new((ctx, suggest)).to_render()
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
                    crate::CompletionSuggest::new((ctx, suggest)).to_render()
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

pub(crate) fn register_type_impl(input: TokenStream) -> TokenStream {
    let type_ident = parse_macro_input!(input as syn::Ident);
    let entry_str = type_ident.to_string();

    get_global_set(&PACKED_TYPES)
        .lock()
        .unwrap()
        .insert(entry_str);

    TokenStream::new()
}

pub(crate) fn register_chain_impl(input: TokenStream) -> TokenStream {
    chain::register_chain(input)
}

pub(crate) fn register_renderer_impl(input: TokenStream) -> TokenStream {
    renderer::register_renderer(input)
}

pub(crate) fn program_fallback_gen_impl(_input: TokenStream) -> TokenStream {
    #[cfg(feature = "structural_renderer")]
    let pack_empty = quote! {
        #[derive(::serde::Serialize, ::mingling::StructuralData, ::mingling::Grouped, Default)]
        pub struct ResultEmpty;
    };

    #[cfg(not(feature = "structural_renderer"))]
    let pack_empty = quote! {
        #[derive(::mingling::Grouped, Default)]
        pub struct ResultEmpty;
    };

    let expanded = quote! {
        ::mingling::macros::pack!(ErrorRendererNotFound = String);
        ::mingling::macros::pack!(ErrorDispatcherNotFound = Vec<String>);
        #pack_empty
    };
    TokenStream::from(expanded)
}

#[allow(clippy::too_many_lines)]
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

    let pathf_map: std::collections::HashMap<String, String> = if cfg!(feature = "pathf") {
        load_pathf_map()
    } else {
        std::collections::HashMap::new()
    };

    let pathf_uses: Vec<proc_macro2::TokenStream> = if cfg!(feature = "pathf") {
        pathf_map
            .values()
            .map(|path| format!("use {};", path).parse().unwrap_or_default())
            .collect()
    } else {
        Vec::new()
    };

    #[cfg(feature = "structural_renderer")]
    let structural_renderer_tokens: Vec<proc_macro2::TokenStream> = structural_renderers
        .iter()
        .map(|s| syn::parse_str::<proc_macro2::TokenStream>(s).unwrap())
        .collect();

    #[cfg(feature = "structural_renderer")]
    let structural_render = quote! {
        fn structural_render(
            any: ::mingling::AnyOutput<Self::Enum>,
            setting: &::mingling::StructuralRendererSetting,
        ) -> Result<::mingling::RenderResult, ::mingling::error::StructuralRendererSerializeError> {
            #[allow(unused_imports)]
            #(#pathf_uses)*
            match any.member_id {
                #(#structural_renderer_tokens)*
                _ => {
                    // Non-structural types: render ResultEmpty (which implements
                    // StructuralData + Serialize) instead of producing nothing.
                    let mut r = ::mingling::RenderResult::default();
                    ::mingling::StructuralRenderer::render(&ResultEmpty, setting, &mut r)?;
                    Ok(r)
                }
            }
        }
    };

    #[cfg(not(feature = "structural_renderer"))]
    let structural_render = quote! {};

    #[cfg(feature = "dispatch_tree")]
    let compile_time_dispatchers: Vec<String> = get_global_set(&COMPILE_TIME_DISPATCHERS)
        .lock()
        .unwrap()
        .clone()
        .iter()
        .cloned()
        .collect();

    #[cfg(feature = "dispatch_tree")]
    let dispatch_tree_nodes = {
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

        let get_nodes_fn = dispatch_tree_gen::gen_get_nodes(&entries, &pathf_map);
        let dispatch_trie_fn = dispatch_tree_gen::gen_dispatch_args_trie(&entries, &pathf_map);

        quote! {
            #get_nodes_fn
            #dispatch_trie_fn
        }
    };

    #[cfg(not(feature = "dispatch_tree"))]
    let dispatch_tree_nodes = quote! {};

    #[cfg(feature = "comp")]
    let completion_tokens: Vec<proc_macro2::TokenStream> = completions
        .iter()
        .map(|s| syn::parse_str::<proc_macro2::TokenStream>(s).unwrap())
        .collect();

    #[cfg(feature = "comp")]
    let comp = quote! {
        fn do_comp(any: &::mingling::AnyOutput<Self::Enum>, ctx: &::mingling::ShellContext) -> ::mingling::Suggest {
            #[allow(unused_imports)]
            #(#pathf_uses)*
            match any.member_id {
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
            let downcast_ty = resolve_type(&variant_ident.to_string(), &pathf_map);
            let resolved_struct = resolve_type(&struct_ident.to_string(), &pathf_map);
            quote! {
                Self::#variant_ident => {
                    // SAFETY: The `type_id` check ensures that `any` contains a value of type `#variant_ident`,
                    // so downcasting to `#variant_ident` is safe.
                    let value = unsafe { any.downcast::<#downcast_ty>().unwrap_unchecked() };
                    <#resolved_struct as ::mingling::Renderer>::render(value)
                }
            }
        }).collect();
            quote! {
                fn render(any: ::mingling::AnyOutput<Self::Enum>) -> ::mingling::RenderResult {
                    match any.member_id {
                        #(#render_arms)*
                        _ => ::mingling::RenderResult::default(),
                    }
                }
            }
        };

    // Build do_chain function (async and sync versions)
    let chain_arms_async: Vec<_> = chain_tokens.iter().map(|entry| {
        let (struct_ident, variant_ident) = parse_entry_pair(entry);
        let downcast_ty = resolve_type(&variant_ident.to_string(), &pathf_map);
        let resolved_struct = resolve_type(&struct_ident.to_string(), &pathf_map);
        quote! {
            Self::#variant_ident => {
                // SAFETY: The `type_id` check ensures that `any` contains a value of type `#variant_ident`,
                // so downcasting to `#variant_ident` is safe.
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
            let downcast_ty = resolve_type(&variant_ident.to_string(), &pathf_map);
            let resolved_struct = resolve_type(&struct_ident.to_string(), &pathf_map);
            quote! {
                Self::#variant_ident => {
                    // SAFETY: The `type_id` check ensures that `any` contains a value of type `#variant_ident`,
                    // so downcasting to `#variant_ident` is safe.
                    let value = unsafe { any.downcast::<#downcast_ty>().unwrap_unchecked() };
                    <#resolved_struct as ::mingling::Chain<Self::Enum>>::proc(value)
                }
            }
        })
        .collect();

    let do_chain_fn = if chain_tokens.is_empty() {
        quote! {
            fn do_chain(_any: ::mingling::AnyOutput<Self::Enum>) -> ::mingling::ChainProcess<Self::Enum> {
                ::core::panic!("No chain found for type id")
            }
        }
    } else if ASYNC_ENABLED {
        quote! {
            fn do_chain(
                any: ::mingling::AnyOutput<Self::Enum>,
            ) -> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ::mingling::ChainProcess<Self::Enum>> + ::std::marker::Send>> {
                match any.member_id {
                    #(#chain_arms_async)*
                    _ => ::core::panic!("No chain found for type id: {:?}", any.type_id),
                }
            }
        }
    } else {
        quote! {
            fn do_chain(
                any: ::mingling::AnyOutput<Self::Enum>,
            ) -> ::mingling::ChainProcess<Self::Enum> {
                match any.member_id {
                    #(#chain_arms_sync)*
                    _ => ::core::panic!("No chain found for type id: {:?}", any.type_id),
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
        #[derive(Debug, PartialEq, Eq, Clone)]
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
            type ErrorDispatcherNotFound = ErrorDispatcherNotFound;
            type ErrorRendererNotFound = ErrorRendererNotFound;
            type ResultEmpty = ResultEmpty;
            fn build_renderer_not_found(member_id: Self::Enum) -> ::mingling::AnyOutput<Self::Enum> {
                ::mingling::AnyOutput::new(ErrorRendererNotFound::new(member_id.to_string()))
            }
            fn build_dispatcher_not_found(args: Vec<String>) -> ::mingling::AnyOutput<Self::Enum> {
                ::mingling::AnyOutput::new(ErrorDispatcherNotFound::new(args))
            }
            fn build_empty_result() -> ::mingling::AnyOutput<Self::Enum> {
                ::mingling::AnyOutput::new(ResultEmpty)
            }
            #render_fn
            #do_chain_fn
            fn render_help(any: ::mingling::AnyOutput<Self::Enum>) -> ::mingling::RenderResult {
                #[allow(unused_imports)]
                #(#pathf_uses)*
                match any.member_id {
                    #(#help_tokens)*
                    _ => ::mingling::RenderResult::default(),
                }
            }
            fn has_renderer(any: &::mingling::AnyOutput<Self::Enum>) -> bool {
                match any.member_id {
                    #(#renderer_exist_tokens)*
                    _ => false
                }
            }
            fn has_chain(any: &::mingling::AnyOutput<Self::Enum>) -> bool {
                match any.member_id {
                    #(#chain_exist_tokens)*
                    _ => false
                }
            }
            #dispatch_tree_nodes
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
    #[cfg(feature = "comp")]
    get_global_set(&COMPLETIONS).lock().unwrap().clear();
    #[cfg(feature = "dispatch_tree")]
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
