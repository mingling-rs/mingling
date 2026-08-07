use mingling::build::{analyze_and_build_type_mapping, build_comp_scripts};

pub mod pre;

fn main() {
    // Perform path analysis and build type mapping table
    analyze_and_build_type_mapping().ok();

    // Generate Mingling CLI Completion Scripts
    build_comp_scripts("mling").unwrap();

    // Generate lint registry
    pre::gen_mod_file().unwrap();
    pre::gen_lint_registry().unwrap();
}
