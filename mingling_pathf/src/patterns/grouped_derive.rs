//! The `GroupedDerivePattern` matches structs, enums, and unions annotated with
//! `#[derive(Grouped)]` or `#[derive(GroupedSerialize)]` (or any combination
//! with other derives). It also recurses into `mod` items to find nested types.
//! This is used to track grouped items for code generation or analysis.

use syn::Item;

use crate::pattern_analyzer::{AnalyzeItem, AnalyzePattern};

/// Matches `#[derive(Grouped)]` and `#[derive(GroupedSerialize)]`.
///
/// Covers the forms:
/// - `#[derive(Grouped)] struct T { ... }`
/// - `#[derive(Grouped, Serialize, ...)] struct T { ... }`
/// - `#[derive(GroupedSerialize)] struct T { ... }`
pub struct GroupedDerivePattern;

impl AnalyzePattern for GroupedDerivePattern {
    fn contains(&self, content: &str) -> bool {
        content.contains("Grouped")
    }

    fn analyze(&self, content: &str) -> Vec<AnalyzeItem> {
        let Ok(syntax) = syn::parse_file(content) else {
            return Vec::new();
        };

        let mut items = Vec::new();

        for item in &syntax.items {
            match item {
                Item::Struct(s) if has_grouped_derive(&s.attrs) => {
                    items.push(AnalyzeItem {
                        module: String::new(),
                        item_name: s.ident.to_string(),
                    });
                }
                Item::Enum(e) if has_grouped_derive(&e.attrs) => {
                    items.push(AnalyzeItem {
                        module: String::new(),
                        item_name: e.ident.to_string(),
                    });
                }
                Item::Union(u) if has_grouped_derive(&u.attrs) => {
                    items.push(AnalyzeItem {
                        module: String::new(),
                        item_name: u.ident.to_string(),
                    });
                }
                Item::Mod(item_mod) => {
                    if let Some((_, nested)) = &item_mod.content {
                        for n in nested {
                            match n {
                                Item::Struct(s) if has_grouped_derive(&s.attrs) => {
                                    items.push(AnalyzeItem {
                                        module: item_mod.ident.to_string(),
                                        item_name: s.ident.to_string(),
                                    });
                                }
                                Item::Enum(e) if has_grouped_derive(&e.attrs) => {
                                    items.push(AnalyzeItem {
                                        module: item_mod.ident.to_string(),
                                        item_name: e.ident.to_string(),
                                    });
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
                    name == "Grouped" || name == "GroupedSerialize"
                }))
            })
            .unwrap_or(false)
        } else {
            false
        }
    })
}
