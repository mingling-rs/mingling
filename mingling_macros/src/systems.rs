// Doc Not Optimize
#[cfg(not(feature = "dispatch_tree"))]
pub(crate) mod dispatch_list_gen;
#[cfg(feature = "dispatch_tree")]
pub(crate) mod dispatch_tree_gen;
pub(crate) mod res_injection;
#[cfg(feature = "structural_renderer")]
pub(crate) mod structural_data;
