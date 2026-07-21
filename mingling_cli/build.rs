use mingling::build::analyze_and_build_type_mapping;

pub mod pre;

fn main() {
    // Perform path analysis and build type mapping table
    analyze_and_build_type_mapping().ok();

    // Generate lint registry
    pre::gen_mod_file().unwrap();
    pre::gen_lint_registry().unwrap();
}
