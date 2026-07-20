//! The `HelpPattern` matches functions annotated with `#[help]` and
//! extracts the generated internal struct name (e.g., `__internal_help_<fn_name>`).
//! This is used to track help functions for code generation or analysis.

use syn::Item;

use crate::pattern_analyzer::{AnalyzeItem, AnalyzePattern};

/// Matches `#[help]` functions, extracting the generated internal struct name.
///
/// `#[help] fn help_my_entry(...)` → `__internal_help_help_my_entry`
pub struct HelpPattern;

impl AnalyzePattern for HelpPattern {
    fn contains(&self, content: &str) -> bool {
        content.contains("[help") || content.contains("help]")
    }

    fn analyze(&self, content: &str) -> Vec<AnalyzeItem> {
        let Ok(syntax) = syn::parse_file(content) else {
            return Vec::new();
        };

        let mut items = Vec::new();
        for item in &syntax.items {
            collect_from_item(item, "", &mut items);
        }
        items
    }
}

fn internal_name(fn_name: &str) -> String {
    format!("__internal_help_{fn_name}")
}

fn collect_from_item(item: &Item, current_mod: &str, items: &mut Vec<AnalyzeItem>) {
    match item {
        Item::Fn(f) if has_attr(&f.attrs, "help") => {
            let fn_name = f.sig.ident.to_string();
            items.push(AnalyzeItem::local(
                current_mod.to_string(),
                internal_name(&fn_name),
            ));
        }
        Item::Mod(item_mod) => {
            if let Some((_, nested)) = &item_mod.content {
                let mod_name = &item_mod.ident.to_string();
                let nested_mod = if current_mod.is_empty() {
                    mod_name.clone()
                } else {
                    format!("{current_mod}::{mod_name}")
                };
                for n in nested {
                    collect_from_item(n, &nested_mod, items);
                }
            }
        }
        _ => {}
    }
}

fn has_attr(attrs: &[syn::Attribute], name: &str) -> bool {
    attrs
        .iter()
        .any(|a| a.path().segments.last().is_some_and(|s| s.ident == name))
}
