// Doc Not Optimize
//! Auto dispatch-strategy selection (no dispatch feature enabled).
//!
//! Picks the matching strategy at macro-expansion time from the normalized
//! command table. The rules are calibrated against the `dev/bench/dispatch`
//! matrix (len 4/8/16/32 × count 128/256 × single/multi/nested4/nested10),
//! where "optimal" is the per-cell minimum of hit×miss geomean:
//!
//! - deep nested chains at modest sizes (`max_words ≥ 8`, `n ≤ 128`) →
//!   **linear list**: a few short memcmps beat the trie's per-level `nth(0)`
//!   walk plus the fallback call on non-leaf hits;
//! - single-word tables with long names → **perfect hash**: one hash beats
//!   the trie's char walk once names grow past ~16 chars;
//! - small tables (`n ≤ 64`) → linear vs trie by a cost model (linear wins
//!   on short names, loses once `count × length` grows);
//! - everything else → **char trie** (O(depth) hit cost independent of table
//!   size, best miss path, linear code size after the fallback-chain fix).

/// Linear per-check memcmp cost: `LIN_A + LIN_B × len` ns, plus a miss term
/// that shrinks as names grow (long names are rejected by the length
/// precheck on the miss path).
const LIN_A: f64 = 0.5;
const LIN_B: f64 = 0.04;
const LIN_MISS_A: f64 = 0.18;
const LIN_MISS_B: f64 = 0.005;

/// Trie per-command overhead: `TRIE_FIXED + TRIE_PER_CHAR × max_len` plus an
/// extra term when exact endpoints exist at multiple depths (nested chains
/// force a fallback call on non-leaf hits).
const TRIE_FIXED: f64 = 25.0;
const TRIE_PER_CHAR: f64 = 1.5;
const TRIE_NESTED: f64 = 25.0;
const TRIE_MISS: f64 = 3.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum DispatchStrategy {
    Linear,
    Trie,
    Phf,
}

/// Pick the dispatch strategy for a normalized entry table.
///
/// Casting `usize` counts to `f64` is intentional: these are cost-model
/// heuristics, not exact numeric computations. Any rounding error is tiny
/// relative to the measured performance margins between strategies, so
/// precision loss here is harmless.
#[allow(clippy::cast_precision_loss)]
pub(crate) fn select_strategy(entries: &[(String, String, String)]) -> DispatchStrategy {
    if entries.is_empty() {
        return DispatchStrategy::Linear;
    }

    let mut names: Vec<String> = Vec::with_capacity(entries.len());
    for (name, _, _) in entries {
        names.push(name.replace('.', " "));
    }

    let n = names.len() as f64;
    let avg_len = names.iter().map(|s| s.chars().count()).sum::<usize>() as f64 / n;
    let max_len = names.iter().map(|s| s.chars().count()).max().unwrap_or(0) as f64;
    let max_words = names
        .iter()
        .map(|s| s.split_whitespace().count())
        .max()
        .unwrap_or(0);

    // Nested prefix chains (a command that is a strict prefix of another)
    // make the trie fall back on non-leaf hits.
    let mut sorted: Vec<&String> = names.iter().collect();
    sorted.sort();
    let nested = sorted.windows(2).any(|w| w[1].starts_with(w[0].as_str()));

    // Nested tables at modest sizes: decide linear vs trie by the cost
    // model. The linear list wins on short names (its few memcmps beat the
    // trie's char walk plus fallback calls) and loses once names grow.
    if nested && n <= 128.0 {
        let miss_linear = n * LIN_MISS_B.mul_add(-avg_len, LIN_MISS_A).max(0.0);
        let cost_linear = (n / 2.0).mul_add(LIN_B.mul_add(avg_len, LIN_A), miss_linear);
        let cost_trie = TRIE_PER_CHAR.mul_add(max_len, TRIE_FIXED) + TRIE_NESTED + TRIE_MISS;
        return if cost_linear < cost_trie {
            DispatchStrategy::Linear
        } else {
            DispatchStrategy::Trie
        };
    }

    // Single-word tables with long names: one hash beats the char walk once
    // names grow past ~16 chars (measured on single-word len16/32).
    if max_words == 1 && (avg_len >= 24.0 || (n <= 128.0 && avg_len >= 16.0)) {
        return DispatchStrategy::Phf;
    }

    // Small tables: linear vs trie by the cost model. The linear list wins
    // on short names but collapses as `count × length` grows.
    if n <= 64.0 {
        let miss_linear = n * LIN_MISS_B.mul_add(-avg_len, LIN_MISS_A).max(0.0);
        let cost_linear = (n / 2.0).mul_add(LIN_B.mul_add(avg_len, LIN_A), miss_linear);
        let cost_trie = TRIE_PER_CHAR.mul_add(max_len, TRIE_FIXED)
            + if nested { TRIE_NESTED } else { 0.0 }
            + TRIE_MISS;
        return if cost_linear < cost_trie {
            DispatchStrategy::Linear
        } else {
            DispatchStrategy::Trie
        };
    }

    DispatchStrategy::Trie
}
