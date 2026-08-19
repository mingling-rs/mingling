// Doc Not Optimize
//! Perfect-hash dispatch generator (`dispatch_phf` feature).
//!
//! Generates a `dispatch_args()` body using a **CHD minimal perfect hash**
//! (Belazzougui, Botelho, Dietzfelbinger) computed at macro-expansion time
//! over the normalized command names.
//!
//! Semantics match `dispatch_list_gen` / `dispatch_tree_gen` exactly:
//! - the longest registered name that is a word-aligned prefix of the input
//!   wins (scan the input word-by-word from the longest possible prefix down
//!   to the first word);
//! - every hash hit is verified with an exact byte equality against the
//!   stored key (the hash alone is not a proof of membership);
//! - duplicate normalized names are dropped, keeping the first registration
//!   ("first wins", same as the other generators).
//!
//! Runtime cost per lookup: one byte scan over the first `max_words` words
//! plus at most `max_words` (typically 1–3) double-hash + verify attempts.
//! Code size is O(1) in the number of commands.

use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::quote;

/// Hash used both at macro time (seed search) and inside the generated code.
/// FNV-1a over the bytes followed by a splitmix64 finalizer for good
/// avalanche; fully deterministic and platform independent.
fn phf_hash(s: &str, seed: u64) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64 ^ seed;
    for &b in s.as_bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h ^= h >> 33;
    h = h.wrapping_mul(0xff51_afd7_ed55_8ccd);
    h ^= h >> 33;
    h = h.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    h ^= h >> 33;
    h
}

/// CHD construction: bucket the keys by `h1`, then per bucket find a
/// displacement `d` such that `(h0(key) + d) % n` is globally collision free.
/// Returns `(seed0, seed1, bucket_count, displacements)`.
///
/// Note: this function performs `u64` → `usize` casts that may truncate on
/// 32-bit platforms. This is intentional and acceptable here because the
/// number of keys is derived from the macro input (command names), which is
/// far smaller than `u32::MAX` in practice on all supported targets.
#[allow(clippy::cast_possible_truncation)]
fn build_chd(keys: &[String]) -> (u64, u64, usize, Vec<u64>) {
    let n = keys.len();
    debug_assert!(n > 0);
    let m = n as u64;
    let b = n.div_ceil(2).max(1);

    for attempt in 0..10_000u64 {
        // Fixed magic constants used as seed bases; the `^ attempt * prime`
        // pattern produces a fresh seed pair on each attempt.
        #[allow(clippy::unreadable_literal)]
        let seed0 = 0x9e37_79b9_7f4a_7c15u64 ^ attempt.wrapping_mul(0xbf58_476d_1ce4_e5b9);
        #[allow(clippy::unreadable_literal)]
        let seed1 = 0x94d0_49bb_1331_11ebu64 ^ attempt.wrapping_mul(0xd6e8_feb8_6659_fd93);

        let mut buckets: Vec<Vec<usize>> = vec![Vec::new(); b];
        for (i, key) in keys.iter().enumerate() {
            let h1 = (phf_hash(key, seed1) % b as u64) as usize;
            buckets[h1].push(i);
        }

        // Place the largest buckets first; they have the least freedom.
        let mut order: Vec<usize> = (0..b).collect();
        order.sort_by_key(|&j| std::cmp::Reverse(buckets[j].len()));

        let mut disp = vec![0u64; b];
        let mut occupied = vec![false; m as usize];
        let mut ok = true;

        'attempt: for &j in &order {
            if buckets[j].is_empty() {
                continue;
            }
            for d in 0..m {
                let mut seen: HashSet<usize> = HashSet::with_capacity(buckets[j].len());
                for &i in &buckets[j] {
                    let h0 = (phf_hash(&keys[i], seed0) % m) as usize;
                    let slot = (h0 + d as usize) % m as usize;
                    if occupied[slot] || !seen.insert(slot) {
                        break;
                    }
                }
                if seen.len() == buckets[j].len() {
                    for &i in &buckets[j] {
                        let h0 = (phf_hash(&keys[i], seed0) % m) as usize;
                        let slot = (h0 + d as usize) % m as usize;
                        occupied[slot] = true;
                    }
                    disp[j] = d;
                    continue 'attempt;
                }
            }
            ok = false;
            break;
        }

        if ok {
            return (seed0, seed1, b, disp);
        }
    }

    panic!("dispatch_phf: failed to construct a perfect hash for {n} keys");
}

// This function is long because it's generating a large body of code we
// don't want to split; keeping it as one codegen function is far easier to
// maintain than splitting it into many small pieces.
#[allow(clippy::too_many_lines)]
/// Generate the `dispatch_args()` function body for a `ProgramCollect` impl.
pub(crate) fn gen_dispatch_args_phf(entries: &[(String, String, String)]) -> TokenStream {
    // Normalize names (dots become spaces), dropping duplicate names while
    // keeping the first registration ("first wins").
    let mut seen = HashSet::new();
    let mut nodes: Vec<(String, String)> = Vec::new();
    for (name, disp, _) in entries {
        let name = name.replace('.', " ");
        if seen.insert(name.clone()) {
            nodes.push((name, disp.clone()));
        }
    }

    let fallback_body = quote! {
        fn dispatch_args(
            raw: &[String],
        ) -> Result<::mingling::AnyOutput<Self::Enum>, ::mingling::error::ProgramInternalExecuteError>
        {
            Ok(Self::build_entry_fallback(raw.to_vec()))
        }
    };
    if nodes.is_empty() {
        return fallback_body;
    }

    let max_words = nodes
        .iter()
        .map(|(name, _)| name.split_whitespace().count())
        .max()
        .unwrap();
    let keys: Vec<String> = nodes.iter().map(|(name, _)| name.clone()).collect();
    let (seed0, seed1, b, disp) = build_chd(&keys);

    let name_lits: Vec<proc_macro2::Literal> = nodes
        .iter()
        .map(|(name, _)| proc_macro2::Literal::string(name))
        .collect();
    let disp_lits: Vec<proc_macro2::Literal> = disp
        .iter()
        .map(|d| proc_macro2::Literal::u64_unsuffixed(*d))
        .collect();

    let arms: Vec<TokenStream> = nodes
        .iter()
        .enumerate()
        .map(|(idx, (_, disp_type))| {
            let idx_lit = proc_macro2::Literal::usize_unsuffixed(idx);
            let disp_ident = proc_macro2::Ident::new(disp_type, proc_macro2::Span::call_site());
            quote! {
                #idx_lit => <#disp_ident as ::mingling::Dispatcher<Self::Enum>>::begin(
                    &#disp_ident::default(),
                    __args,
                ),
            }
        })
        .collect();

    let max_words_lit = proc_macro2::Literal::usize_unsuffixed(max_words);
    let seed0_lit = proc_macro2::Literal::u64_unsuffixed(seed0);
    let seed1_lit = proc_macro2::Literal::u64_unsuffixed(seed1);
    let b_lit = proc_macro2::Literal::usize_unsuffixed(b);
    let m_lit = proc_macro2::Literal::usize_unsuffixed(keys.len());

    quote! {
        fn dispatch_args(
            raw: &[String],
        ) -> Result<::mingling::AnyOutput<Self::Enum>, ::mingling::error::ProgramInternalExecuteError>
        {
            const __DISPATCH_PHF_MAX_WORDS: usize = #max_words_lit;
            const __DISPATCH_PHF_SEED0: u64 = #seed0_lit;
            const __DISPATCH_PHF_SEED1: u64 = #seed1_lit;
            const __DISPATCH_PHF_BUCKETS: usize = #b_lit;
            const __DISPATCH_PHF_MOD: usize = #m_lit;
            static __DISPATCH_PHF_KEYS: &[&str] = &[#(#name_lits),*];
            static __DISPATCH_PHF_DISP: &[u64] = &[#(#disp_lits),*];

            #[inline(always)]
            fn __dispatch_phf_hash(s: &str, seed: u64) -> u64 {
                let mut h = 0xcbf29ce484222325u64 ^ seed;
                for &b in s.as_bytes() {
                    h ^= b as u64;
                    h = h.wrapping_mul(0x100000001b3);
                }
                h ^= h >> 33;
                h = h.wrapping_mul(0xff51afd7ed558ccd);
                h ^= h >> 33;
                h = h.wrapping_mul(0xc4ceb9fe1a85ec53);
                h ^= h >> 33;
                h
            }

            let raw_string = format!("{} ", raw.join(" "));
            let raw_str = raw_string.as_str();
            let bytes = raw_str.as_bytes();

            // Record the byte offset of the space that terminates each word
            // (`raw_str` always ends with a trailing space).
            let mut __ends = [0usize; #max_words_lit];
            let mut __w = 0usize;
            let mut __i = 0usize;
            while __w < __DISPATCH_PHF_MAX_WORDS && __i < bytes.len() {
                if bytes[__i] == b' ' {
                    __ends[__w] = __i;
                    __w += 1;
                }
                __i += 1;
            }

            // Scan the input from the longest possible prefix down to the
            // first word, so the longest registered prefix wins (mirroring
            // the linear list and trie generators).
            let mut __k = __w;
            while __k > 0 {
                let __cand = &raw_str[..__ends[__k - 1]];
                let __h1 = (__dispatch_phf_hash(__cand, __DISPATCH_PHF_SEED1)
                    % __DISPATCH_PHF_BUCKETS as u64) as usize;
                let __d = __DISPATCH_PHF_DISP[__h1];
                let __idx = ((__dispatch_phf_hash(__cand, __DISPATCH_PHF_SEED0)
                    % __DISPATCH_PHF_MOD as u64)
                    + __d)
                    % __DISPATCH_PHF_MOD as u64;
                let __idx = __idx as usize;
                if __DISPATCH_PHF_KEYS[__idx].as_bytes() == __cand.as_bytes() {
                    let __args: Vec<String> = raw.iter().skip(__k).cloned().collect();
                    let __cp = match __idx {
                        #(#arms)*
                        _ => unreachable!("dispatch_phf: perfect-hash index out of range"),
                    };
                    return match __cp {
                        ::mingling::ChainProcess::Ok(any_output) => Ok(any_output.0),
                        ::mingling::ChainProcess::Err(chain_process_error) => {
                            Err(chain_process_error.into())
                        }
                    };
                }
                __k -= 1;
            }

            Ok(Self::build_entry_fallback(raw.to_vec()))
        }
    }
}
