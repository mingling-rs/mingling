//! Template Linter
//!
//! ## Summary
//!
//! This is a template Linter that introduces how to add a Lint for Mingling.
//! You can write an introduction for this Linter in the Summary section, for example:
//!
//! - Trigger conditions
//! - Why is it necessary?
//!
//! ## Metadata
//!
//! > This section is the **Metadata** section, which needs to be filled in correctly.
//! > These contents will eventually be compiled as the Linter's behavior.
//!
//! Author: `Your-Name`
//! Default: `allow`
//            ^^^^^ Supported parameters: `warn`, `allow`, `deny`

// --- ABOUT AUTO IDENTIFICATION RULES ---
//
// The compiler will treat code with the following structure as a Linter entry point:
//  |
// --> your_linter_module.rs
//  |
//  | pub fn linter(ast: syn::ItemX) -> Vec<MlintReport> {
//  |     /* ... */      ^^^^^^^^^^ Your linter scope
//  | }
//  |
//  = note: Please ensure your function is `pub`, named `linter`, and returns `Vec<MlintReport>`
//
// --- ABOUT AUTO IDENTIFICATION RULES ---

use crate::linter::mlint_report::MlintReport;

pub fn linter(_ast: syn::ItemFn, _source: &str) -> Vec<MlintReport> {
    //              ^^^^^^^^^^^ Supported parameters:
    //                               | syn::File
    //                               | syn::ItemImpl
    //                               | syn::ItemStruct
    //                               | syn::ItemEnum
    //                               | syn::ItemTrait
    //                               | syn::ItemFn
    //                               | syn::ItemMacro
    //                               | syn::ItemMod
    //                               | syn::ItemUnion
    vec![]
}

#[cfg(test)]
mod lint_test {
    use crate::{assert_detected, assert_not_detected};

    #[test]
    fn test() {}
}
