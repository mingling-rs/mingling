use syn::Item;

use crate::pattern_analyzer::{AnalyzeItem, AnalyzePattern};

/// Matches `#[derive(Groupped)]` and `#[derive(GrouppedSerialize)]`.
///
/// Covers the forms:
/// - `#[derive(Groupped)] struct T { ... }`
/// - `#[derive(Groupped, Serialize, ...)] struct T { ... }`
/// - `#[derive(GrouppedSerialize)] struct T { ... }`
pub struct GrouppedDerivePattern;

impl AnalyzePattern for GrouppedDerivePattern {
    fn contains(&self, content: &str) -> bool {
        content.contains("Groupped")
    }

    fn analyze(&self, content: &str) -> Vec<AnalyzeItem> {
        let Ok(syntax) = syn::parse_file(content) else {
            return Vec::new();
        };

        let mut items = Vec::new();

        for item in &syntax.items {
            match item {
                Item::Struct(s) => {
                    if has_groupped_derive(&s.attrs) {
                        items.push(AnalyzeItem {
                            module: String::new(),
                            item_name: s.ident.to_string(),
                        });
                    }
                }
                Item::Enum(e) => {
                    if has_groupped_derive(&e.attrs) {
                        items.push(AnalyzeItem {
                            module: String::new(),
                            item_name: e.ident.to_string(),
                        });
                    }
                }
                Item::Union(u) => {
                    if has_groupped_derive(&u.attrs) {
                        items.push(AnalyzeItem {
                            module: String::new(),
                            item_name: u.ident.to_string(),
                        });
                    }
                }
                Item::Mod(item_mod) => {
                    if let Some((_, nested)) = &item_mod.content {
                        for n in nested {
                            match n {
                                Item::Struct(s) if has_groupped_derive(&s.attrs) => {
                                    items.push(AnalyzeItem {
                                        module: item_mod.ident.to_string(),
                                        item_name: s.ident.to_string(),
                                    });
                                }
                                Item::Enum(e) if has_groupped_derive(&e.attrs) => {
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

fn has_groupped_derive(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if attr.path().is_ident("derive") {
            attr.parse_args::<syn::MetaList>().ok().is_some_and(|meta| {
                meta.path.segments.iter().any(|seg| {
                    let name = seg.ident.to_string();
                    name == "Groupped" || name == "GrouppedSerialize"
                })
            })
        } else {
            false
        }
    })
}
