//! The `RendererPattern` matches functions annotated with `#[renderer]` and
//! extracts the generated internal struct name (e.g., `__internal_renderer_<fn_name>`).
//! This is used to track rendering functions for code generation or analysis.

use syn::Item;

use crate::pattern_analyzer::{AnalyzeItem, AnalyzePattern};

/// Match `#[renderer]` functions, extract the generated internal struct name.
///
/// `#[renderer] fn render_name(...)` → `__internal_renderer_render_name`
pub struct RendererPattern;

impl AnalyzePattern for RendererPattern {
    fn contains(&self, content: &str) -> bool {
        content.contains("[renderer") || content.contains("renderer]")
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
    format!("__internal_renderer_{fn_name}")
}

fn collect_from_item(item: &Item, current_mod: &str, items: &mut Vec<AnalyzeItem>) {
    match item {
        Item::Fn(f) if has_attr(&f.attrs, "renderer") => {
            let fn_name = f.sig.ident.to_string();
            items.push(AnalyzeItem {
                module: current_mod.to_string(),
                item_name: internal_name(&fn_name),
            });
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
