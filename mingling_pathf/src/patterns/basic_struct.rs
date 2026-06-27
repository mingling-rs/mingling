use syn::Item;

use crate::pattern_analyzer::{AnalyzeItem, AnalyzePattern};

/// Basic struct pattern analyzer.
///
/// Used to identify and analyze struct definitions (`struct`) in Rust source code.
/// Supports analyzing root-level structs as well as structs within inline modules.
pub struct BasicStructPattern;

impl AnalyzePattern for BasicStructPattern {
    fn contains(&self, content: &str) -> bool {
        content.contains("struct")
    }

    fn analyze(&self, content: &str) -> Vec<AnalyzeItem> {
        let Ok(syntax) = syn::parse_file(content) else {
            return Vec::new();
        };

        let mut items = Vec::new();

        for item in &syntax.items {
            match item {
                // Root-level struct
                Item::Struct(s) => {
                    items.push(AnalyzeItem {
                        module: String::new(),
                        item_name: s.ident.to_string(),
                    });
                }
                // Struct within inline modules
                Item::Mod(item_mod) => {
                    if let Some((_, nested)) = &item_mod.content {
                        for n in nested {
                            if let syn::Item::Struct(s) = n {
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
