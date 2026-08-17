pub mod pre;

fn main() {
    // Generate lint registry
    pre::gen_mod_file().unwrap();
    pre::gen_lint_registry().unwrap();
}
