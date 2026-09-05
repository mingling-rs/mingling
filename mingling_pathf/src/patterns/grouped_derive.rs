// Doc Not Optimize
//! The `GroupedDerivePattern` matches structs, enums, and unions annotated with
//! `#[derive(Grouped)]`, `#[derive(GroupedSerialize)]`, or `#[derive(StructuralData)]`
//! (or any combination with other derives). It also recurses into `mod` items to
//! find nested types. This is used to track grouped items for code generation or
//! analysis.

use syn::Item;

use crate::pattern_analyzer::{AnalyzeItem, AnalyzePattern};

/// Matches `#[derive(Grouped)]`, `#[derive(GroupedSerialize)]`, and
/// `#[derive(StructuralData)]`.
///
/// Covers the forms:
/// - `#[derive(Grouped)] struct T { ... }`
/// - `#[derive(Grouped, Serialize, ...)] struct T { ... }`
/// - `#[derive(GroupedSerialize)] struct T { ... }`
/// - `#[derive(StructuralData)] struct T { ... }`
pub struct GroupedDerivePattern;

impl AnalyzePattern for GroupedDerivePattern {
    fn contains(&self, content: &str) -> bool {
        content.contains("Grouped") || content.contains("StructuralData")
    }

    fn analyze(&self, content: &str) -> Vec<AnalyzeItem> {
        let Ok(syntax) = syn::parse_file(content) else {
            return Vec::new();
        };

        let mut items = Vec::new();

        for item in &syntax.items {
            match item {
                Item::Struct(s) if has_grouped_derive(&s.attrs) => {
                    items.push(type_module(String::new(), &s.ident.to_string()));
                }
                Item::Enum(e) if has_grouped_derive(&e.attrs) => {
                    items.push(type_module(String::new(), &e.ident.to_string()));
                }
                Item::Union(u) if has_grouped_derive(&u.attrs) => {
                    items.push(type_module(String::new(), &u.ident.to_string()));
                }
                Item::Mod(item_mod) => {
                    if let Some((_, nested)) = &item_mod.content {
                        for n in nested {
                            match n {
                                Item::Struct(s) if has_grouped_derive(&s.attrs) => {
                                    items.push(type_module(
                                        item_mod.ident.to_string(),
                                        &s.ident.to_string(),
                                    ));
                                }
                                Item::Enum(e) if has_grouped_derive(&e.attrs) => {
                                    items.push(type_module(
                                        item_mod.ident.to_string(),
                                        &e.ident.to_string(),
                                    ));
                                }
                                _ => {}
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        items
    }
}

fn type_module(module: String, type_name: &str) -> AnalyzeItem {
    AnalyzeItem::local_module(
        module,
        format!("__mingling_type_{}", just_fmt::snake_case!(type_name)),
    )
}

fn has_grouped_derive(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if attr.path().is_ident("derive") {
            // Correctly parse comma-separated paths in #[derive(Grouped, Debug, ...)]
            attr.parse_args_with(|input: syn::parse::ParseStream| {
                let paths =
                    syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated(
                        input,
                    )?;
                Ok(paths.iter().any(|p| {
                    let name = p.segments.last().unwrap().ident.to_string();
                    name == "Grouped" || name == "GroupedSerialize" || name == "StructuralData"
                }))
            })
            .unwrap_or(false)
        } else {
            false
        }
    })
}
