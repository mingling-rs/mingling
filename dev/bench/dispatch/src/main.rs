//! Dispatch-strategy matrix benchmark: linear list vs char trie vs perfect
//! hash, plus the "auto" strategy (which picks one of the three from the
//! command table — the summary checks it against the per-cell minimum).
//!
//! Build & run (workspace-internal, `-O`):
//! ```bash
//! cargo run --release --manifest-path dev/bench/dispatch/Cargo.toml
//! ```

#![allow(dead_code, clippy::all)]

use prettytable::format::Alignment;
use prettytable::{Cell, Row, Table};

/// Minimal dispatch surface implemented by every generated cell so the four
/// strategies can be benchmarked through a uniform call.
pub trait BenchDispatch {
    type Enum;
    fn dispatch_args(
        raw: &[String],
    ) -> Result<::mingling::AnyOutput<Self::Enum>, ::mingling::error::ProgramInternalExecuteError>;
    fn build_entry_fallback(args: Vec<String>) -> ::mingling::AnyOutput<Self::Enum>;
}

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

const N_STRATS: usize = 4; // linear, trie, phf, auto

fn right(v: f64) -> Cell {
    Cell::new_align(&format!("{v:.1}"), Alignment::RIGHT)
}

fn right2(v: f64) -> Cell {
    Cell::new_align(&format!("{v:.2}"), Alignment::RIGHT)
}

fn left(s: &str) -> Cell {
    Cell::new(s)
}

fn main() {
    println!("dispatch matrix — ns/op (lower is better), best of 5×50k, rustc -O");
    println!();

    let results: Vec<(String, String, f64, f64)> = CELL_META
        .iter()
        .enumerate()
        .map(|(id, (label, strat, names))| {
            let hits: Vec<Vec<String>> = names
                .iter()
                .map(|name| {
                    let mut v: Vec<String> = name.split(' ').map(str::to_string).collect();
                    v.push("argA".into());
                    v.push("argB".into());
                    v
                })
                .collect();
            let misses: Vec<Vec<String>> = vec![
                vec!["qqq".into(), "zzz".into()],
                vec!["qqq".into(), "zzz".into(), "argA".into()],
                vec!["qqq".into()],
            ];
            let (h, m) = run_cell(id, &hits, &misses);
            (label.to_string(), strat.to_string(), h, m)
        })
        .collect();

    let mut table = Table::new();
    table.set_format(*prettytable::format::consts::FORMAT_BOX_CHARS);
    table.set_titles(Row::new(vec![
        left("cell"),
        left("── hit ──"),
        left(""),
        left(""),
        left(""),
        left("── miss ──"),
        left(""),
        left(""),
        left(""),
    ]));
    table.add_row(Row::new(vec![
        left(""),
        left("lin"),
        left("trie"),
        left("phf"),
        left("auto"),
        left("lin"),
        left("trie"),
        left("phf"),
        left("auto"),
    ]));

    let mut geo_hit = [1.0f64; N_STRATS];
    let mut geo_miss = [1.0f64; N_STRATS];
    let mut wins = [0usize; N_STRATS];
    let mut auto_optimal = 0usize;
    let mut auto_within5 = 0usize;
    let mut cells = 0usize;
    let mut auto_picks = [0usize; N_STRATS]; // auto's own selection counts

    for (chunk_idx, chunk) in results.chunks(N_STRATS).enumerate() {
        let label = &chunk[0].0;
        let hit: Vec<f64> = chunk.iter().map(|c| c.2).collect();
        let miss: Vec<f64> = chunk.iter().map(|c| c.3).collect();
        table.add_row(Row::new(vec![
            left(label),
            right(hit[0]),
            right(hit[1]),
            right(hit[2]),
            right(hit[3]),
            right(miss[0]),
            right(miss[1]),
            right(miss[2]),
            right(miss[3]),
        ]));
        for s in 0..N_STRATS {
            geo_hit[s] *= hit[s];
            geo_miss[s] *= miss[s];
        }
        // Per-cell winner by hit×miss geomean among the three explicit
        // strategies.
        let gm: Vec<f64> = (0..N_STRATS).map(|s| (hit[s] * miss[s]).sqrt()).collect();
        let mut best_idx = 0usize;
        for s in 1..3 {
            if gm[s] < gm[best_idx] {
                best_idx = s;
            }
        }
        let best_idx_gm = gm[best_idx];
        wins[best_idx] += 1;
        // The auto cell (last of the four) exposes its chosen strategy; check
        // it against the measured minimum.
        let auto_id = chunk_idx * N_STRATS + (N_STRATS - 1);
        let pick_idx = match run_cell_pick(auto_id) {
            "dispatch_linear" => 0,
            "dispatch_tree" => 1,
            _ => 2,
        };
        auto_picks[pick_idx] += 1;
        if pick_idx == best_idx {
            auto_optimal += 1;
        }
        // Identical generated code measures up to ~10 ns apart across cells,
        // so also count selections within 5% of the measured minimum.
        if gm[3] <= best_idx_gm * 1.05 {
            auto_within5 += 1;
        }
        cells += 1;
    }

    let n = cells as f64;
    table.add_row(Row::new(vec![
        left("geomean"),
        right2(geo_hit[0].powf(1.0 / n)),
        right2(geo_hit[1].powf(1.0 / n)),
        right2(geo_hit[2].powf(1.0 / n)),
        right2(geo_hit[3].powf(1.0 / n)),
        right2(geo_miss[0].powf(1.0 / n)),
        right2(geo_miss[1].powf(1.0 / n)),
        right2(geo_miss[2].powf(1.0 / n)),
        right2(geo_miss[3].powf(1.0 / n)),
    ]));
    table.add_row(Row::new(vec![
        left("wins (hit×miss geomean)"),
        left(&wins[0].to_string()),
        left(&wins[1].to_string()),
        left(&wins[2].to_string()),
    ]));
    table.add_row(Row::new(vec![
        left("auto picks"),
        left(&auto_picks[0].to_string()),
        left(&auto_picks[1].to_string()),
        left(&auto_picks[2].to_string()),
    ]));
    table.add_row(Row::new(vec![left(
        format!("auto == per-cell best: {auto_optimal}/{cells}").as_str(),
    )]));
    table.add_row(Row::new(vec![left(
        format!("auto within 5% of best:  {auto_within5}/{cells}").as_str(),
    )]));

    table.printstd();
}
