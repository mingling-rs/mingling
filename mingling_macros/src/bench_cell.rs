//! Workspace-internal `bench_cell!` implementation for the `mingling_bench`
//! harness (compiled only with the `bench_support` feature).

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, LitStr, Result as SynResult, Token, bracketed};

use crate::systems::dispatch_auto;
use crate::systems::dispatch_list_gen::gen_dispatch_args;
use crate::systems::dispatch_phf_gen::gen_dispatch_args_phf;
use crate::systems::dispatch_tree_gen::gen_dispatch_args_trie;

struct BenchCellInput {
    strategy: Ident,
    module: Ident,
    entries: Vec<(LitStr, Ident)>,
}

impl Parse for BenchCellInput {
    fn parse(input: ParseStream<'_>) -> SynResult<Self> {
        let strategy = input.parse()?;
        input.parse::<Token![,]>()?;
        let module = input.parse()?;
        input.parse::<Token![,]>()?;
        let content;
        bracketed!(content in input);
        let mut entries = Vec::new();
        while !content.is_empty() {
            let name = content.parse()?;
            content.parse::<Token![=>]>()?;
            let disp = content.parse()?;
            entries.push((name, disp));
            if content.is_empty() {
                break;
            }
            content.parse::<Token![,]>()?;
        }
        Ok(Self {
            strategy,
            module,
            entries,
        })
    }
}

pub(crate) fn bench_cell_impl(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as BenchCellInput);

    let entries: Vec<(String, String, String)> = input
        .entries
        .iter()
        .map(|(name, disp)| (name.value(), disp.to_string(), String::new()))
        .collect();

    let (dispatch, extra, chosen) = match input.strategy.to_string().as_str() {
        "dispatch_linear" => (gen_dispatch_args(&entries), quote! {}, "dispatch_linear"),
        "dispatch_tree" => {
            let (d, e) = gen_dispatch_args_trie(&entries);
            (d, e, "dispatch_tree")
        }
        "dispatch_phf" => (gen_dispatch_args_phf(&entries), quote! {}, "dispatch_phf"),
        "dispatch_auto" => match dispatch_auto::select_strategy(&entries) {
            dispatch_auto::DispatchStrategy::Linear => {
                (gen_dispatch_args(&entries), quote! {}, "dispatch_linear")
            }
            dispatch_auto::DispatchStrategy::Trie => {
                let (d, e) = gen_dispatch_args_trie(&entries);
                (d, e, "dispatch_tree")
            }
            dispatch_auto::DispatchStrategy::Phf => {
                (gen_dispatch_args_phf(&entries), quote! {}, "dispatch_phf")
            }
        },
        other => panic!("bench_cell: unknown strategy `{other}`"),
    };

    // Keep the generated dispatch out of the harness loop's inline budget:
    // the loop calls it once per iteration, and large tables (hundreds of
    // arms) would otherwise make LLVM attempt to inline a huge function into
    // the 50k-iteration loop, exploding compile time and memory. Production
    // codegen would not inline such functions either. The trie generator
    // also emits its shared `__trie_fallback` method via `extra`, so parse
    // the whole item list (a `syn::File`) and annotate the first function
    // (dispatch_args).
    let mut file: syn::File = syn::parse2(dispatch)
        .unwrap_or_else(|e| panic!("bench_cell: generated dispatch_args failed to parse: {e}"));
    if let Some(syn::Item::Fn(dispatch_fn)) = file.items.first_mut() {
        dispatch_fn.attrs.push(syn::parse_quote!(#[inline(never)]));
    }
    let dispatch = quote!(#file);

    // The trie fallback is a generic method that must live in an inherent
    // impl of the cell type (it references no `Self` associated types).
    let extra_impl = if extra.is_empty() {
        quote! {}
    } else {
        quote! {
            impl X {
                #extra
            }
        }
    };

    let module = &input.module;
    let name_lits: Vec<proc_macro2::Literal> = input
        .entries
        .iter()
        .map(|(name, _)| proc_macro2::Literal::string(&name.value()))
        .collect();

    let mut dispatchers = proc_macro2::TokenStream::new();
    for (_, disp) in &input.entries {
        dispatchers.extend(quote! {
            #[derive(Default)]
            pub struct #disp;
            impl Dispatcher<X> for #disp {
                fn begin(&self, _args: Vec<String>) -> ChainProcess<X> {
                    ChainProcess::Ok((AnyOutput::new(Dummy), NextProcess::Chain))
                }
            }
        });
    }

    quote! {
        pub mod #module {
            use ::mingling::{AnyOutput, ChainProcess, Dispatcher, Grouped, NextProcess};

            #dispatchers

            #[derive(Clone, Copy)]
            pub struct X;
            unsafe impl Grouped<X> for X {
                fn member_id() -> X {
                    X
                }
            }

            pub struct Dummy;
            unsafe impl Grouped<X> for Dummy {
                fn member_id() -> X {
                    X
                }
            }

            pub struct EntryFallback(pub Vec<String>);
            unsafe impl Grouped<X> for EntryFallback {
                fn member_id() -> X {
                    X
                }
            }

            impl crate::BenchDispatch for X {
                type Enum = X;
                fn build_entry_fallback(args: Vec<String>) -> AnyOutput<Self::Enum> {
                    AnyOutput::new(EntryFallback(args))
                }
                #dispatch
            }

            #extra_impl

            pub static NAMES: &[&str] = &[#(#name_lits),*];

            /// The strategy this cell was generated with (for `dispatch_auto`
            /// cells this is the auto-selected strategy).
            pub static STRATEGY: &str = #chosen;

            /// Time dispatch over a corpus; returns (hit_ns, miss_ns), best of
            /// 5 rounds of 50k dispatches.
            pub fn measure(hits: &[Vec<String>], misses: &[Vec<String>]) -> (f64, f64) {
                let mut acc = 0usize;
                for i in 0..2_000 {
                    let r = <X as crate::BenchDispatch>::dispatch_args(&hits[i % hits.len()]);
                    acc = acc.wrapping_add(match r {
                        Ok(_) => 1,
                        Err(_) => 2,
                    });
                }
                std::hint::black_box(acc);

                let time = |corpus: &[Vec<String>]| -> f64 {
                    let mut best = f64::MAX;
                    for _ in 0..5 {
                        let start = std::time::Instant::now();
                        let mut acc = 0usize;
                        for i in 0..50_000 {
                            let r = <X as crate::BenchDispatch>::dispatch_args(
                                &corpus[i % corpus.len()],
                            );
                            acc = acc.wrapping_add(match r {
                                Ok(_) => 1,
                                Err(_) => 2,
                            });
                        }
                        let el = start.elapsed().as_nanos() as f64 / 50_000.0;
                        std::hint::black_box(acc);
                        if el < best {
                            best = el;
                        }
                    }
                    best
                };

                (time(hits), time(misses))
            }
        }
    }
    .into()
}
