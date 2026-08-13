// Doc Not Optimize
//! The `CompletionPattern` matches functions annotated with `#[completion(T)]` and
//! extracts the generated internal struct name (e.g., `__internal_completion_<fn_name>`).
//! This is used to track completion handler functions for code generation or analysis.

use syn::Item;

use crate::pattern_analyzer::{AnalyzeItem, AnalyzePattern};

/// Matches `#[completion(T)]` functions, extracts the generated inner struct name.
///
/// `#[completion(EntryGreet)] fn complete_greet_entry(...)` → `__internal_completion_complete_greet_entry`
pub struct CompletionPattern;

impl AnalyzePattern for CompletionPattern {
    fn contains(&self, content: &str) -> bool {
        content.contains("completion(") || content.contains("[completion")
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
    format!("__internal_completion_{fn_name}")
}

fn collect_from_item(item: &Item, current_mod: &str, items: &mut Vec<AnalyzeItem>) {
    match item {
        Item::Fn(f) if has_attr(&f.attrs, "completion") => {
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
