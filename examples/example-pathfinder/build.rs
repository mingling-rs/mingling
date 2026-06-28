use mingling::builds::analyze_and_build_type_mapping;

fn main() {
    // --------- IMPORTANT ---------
    // Use this method in build.rs,
    //   to analyze the project structure at build time,
    //   and automatically introduce members from other modules into gen_program!()
    analyze_and_build_type_mapping().unwrap();
    // --------- IMPORTANT ---------
}
