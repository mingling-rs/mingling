use syn::Item;

use crate::pattern_analyzer::{AnalyzeItem, AnalyzePattern};

/// Match structs annotated with `#[dispatcher_clap]`, extracting the entry type name (i.e., the struct name).
///
/// Covers the following forms:
/// - `#[dispatcher_clap] struct EntryGreet { ... }`
/// - `#[dispatcher_clap] #[command(...)] struct EntryGreet { ... }`
pub struct DispatcherClapPattern;

impl AnalyzePattern for DispatcherClapPattern {
    fn contains(&self, content: &str) -> bool {
        content.contains("dispatcher_clap(")
    }

    fn analyze(&self, content: &str) -> Vec<AnalyzeItem> {
        let Ok(syntax) = syn::parse_file(content) else {
            return Vec::new();
        };

        let mut items = Vec::new();

        for item in &syntax.items {
            match item {
                Item::Struct(s) if has_attr(&s.attrs, "dispatcher_clap") => {
                    items.push(AnalyzeItem {
                        module: String::new(),
                        item_name: s.ident.to_string(),
                    });
                }
                Item::Mod(item_mod) => {
                    if let Some((_, nested)) = &item_mod.content {
                        for n in nested {
                            if let Item::Struct(s) = n
                                && has_attr(&s.attrs, "dispatcher_clap") {
                                    items.push(AnalyzeItem {
                                        module: item_mod.ident.to_string(),
                                        item_name: s.ident.to_string(),
                                    });
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

fn has_attr(attrs: &[syn::Attribute], name: &str) -> bool {
    attrs.iter().any(|a| {
        a.path()
            .segments
            .last()
            .is_some_and(|s| s.ident == name)
    })
}
