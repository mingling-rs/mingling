#[doc(hidden)]
#[cfg(feature = "comp")]
pub mod comp;

#[cfg(all(feature = "builds", feature = "pathf"))]
pub use mingling_pathf::analyze_and_build_type_mapping;

#[cfg(all(feature = "builds", feature = "pathf"))]
pub use mingling_pathf::analyze_and_build_type_mapping_for;
