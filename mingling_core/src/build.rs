#[doc(hidden)]
#[cfg(feature = "comp")]
mod comp;

#[cfg(feature = "comp")]
mod comp_re_export {
    pub use super::comp::build_comp_script;
    pub use super::comp::build_comp_script_to;
    pub use super::comp::build_comp_script_to_file;
    pub use super::comp::build_comp_scripts;
}

#[cfg(feature = "comp")]
pub use comp_re_export::*;

#[doc(hidden)]
#[cfg(feature = "pathf")]
mod pathf;

#[cfg(feature = "pathf")]
mod pathf_re_export {
    pub use super::pathf::analyze;
    pub use super::pathf::analyze_and_build_type_mapping;
    pub use super::pathf::analyze_and_build_type_mapping_for;
}

#[cfg(feature = "pathf")]
pub use pathf_re_export::*;
