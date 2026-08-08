fn main() {
??? >>> COMPLETION
    build_scripts();
??? <<<
??? >>> PATHF
    build_pathf_mapping();
??? <<<
}
??? >>> COMPLETION

/// Generate completion scripts
fn build_scripts() {
    // `env!("CARGO_PKG_NAME")` equals the crate name, which matches the binary name.
    // If your binary name differs from the crate name, specify it explicitly.
    mingling::build::build_comp_scripts(
        // Your binary name:
        env!("CARGO_PKG_NAME"),
    )
    .unwrap();
}
??? <<<
??? >>> PATHF

/// Use `pathf` to analyze the workspace's mingling macro calls
/// and generate type path mappings for them
fn build_pathf_mapping() {
    // Build pathf type mapping to ensure that the enabled `pathf` feature
    // can correctly scan macros in the project
    mingling::build::analyze_and_build_type_mapping().unwrap();
}
??? <<<
