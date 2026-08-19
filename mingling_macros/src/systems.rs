// Dispatch-strategy generators. Exactly one of {linear list, char trie,
// perfect-hash} is wired into `gen_program!` at a time (see
// `func/program_final_gen.rs`); with no dispatch feature enabled the "auto"
// strategy picks one from the table (see `dispatch_auto`). The
// workspace-internal `bench_support` feature compiles all three so the
// `bench_cell!` macro can generate the `dev/bench/dispatch` harness cells.

#[cfg(any(
    all(not(feature = "dispatch_phf"), not(feature = "dispatch_tree")),
    feature = "bench_support"
))]
pub(crate) mod dispatch_list_gen;

#[cfg(any(not(feature = "dispatch_phf"), feature = "bench_support"))]
pub(crate) mod dispatch_tree_gen;

#[cfg(any(
    all(not(feature = "dispatch_phf"), not(feature = "dispatch_tree")),
    feature = "dispatch_phf",
    feature = "bench_support"
))]
pub(crate) mod dispatch_phf_gen;

#[cfg(any(
    all(not(feature = "dispatch_phf"), not(feature = "dispatch_tree")),
    feature = "bench_support"
))]
pub(crate) mod dispatch_auto;

pub(crate) mod res_injection;

// TEMPORARY diagnostic probe — remove after bench matrix tuning.
#[cfg(test)]
mod size_probe {
    use crate::systems::dispatch_list_gen::gen_dispatch_args;
    use crate::systems::dispatch_phf_gen::gen_dispatch_args_phf;
    use crate::systems::dispatch_tree_gen::gen_dispatch_args_trie;

    fn entries_nested(count: usize, depth: usize) -> Vec<(String, String, String)> {
        let mut out = Vec::new();
        let groups = count.div_ceil(depth);
        'outer: for g in 0..groups {
            let base = format!("n{g:03}");
            for d in 0..depth {
                let mut name = base.clone();
                for k in 1..=d {
                    name.push_str(&format!(" w{k}"));
                }
                out.push((name, format!("D{}", out.len()), String::new()));
                if out.len() == count {
                    break 'outer;
                }
            }
        }
        out
    }

    fn entries_single(count: usize, len: usize) -> Vec<(String, String, String)> {
        (0..count)
            .map(|i| (format!("c{:0len$}", i), format!("D{i}"), String::new()))
            .collect()
    }

    #[test]
    fn probe() {
        for &(count, depth) in &[(128usize, 4usize), (1024, 4), (1024, 16)] {
            let e = entries_nested(count, depth);
            let t = gen_dispatch_args_trie(&e).0.to_string();
            let l = gen_dispatch_args(&e).to_string();
            let p = gen_dispatch_args_phf(&e).to_string();
            eprintln!(
                "nested count={count} depth={depth}: trie={}B lin={}B phf={}B",
                t.len(),
                l.len(),
                p.len()
            );
        }
        let e = entries_single(1024, 32);
        let t = gen_dispatch_args_trie(&e).0.to_string();
        let l = gen_dispatch_args(&e).to_string();
        let p = gen_dispatch_args_phf(&e).to_string();
        eprintln!(
            "single count=1024 len=32: trie={}B lin={}B phf={}B",
            t.len(),
            l.len(),
            p.len()
        );
    }
}
