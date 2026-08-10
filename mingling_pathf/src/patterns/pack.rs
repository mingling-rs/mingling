//! The `PackPattern` matches types defined by `pack!`, `pack_err!`, `pack_structural!`, and `pack_err_structural!` macros.
//! It extracts the registered type name (e.g., `TypeName` from `pack!(TypeName = InnerType)`).
//! This is used to track packed type definitions for code generation or analysis.

use syn::Item;

use crate::pattern_analyzer::{AnalyzeItem, AnalyzePattern};

/// Matches types defined by `pack!`, `pack_err!`, `pack_structural!`, `pack_err_structural!` macros.
///
/// Covered forms:
/// - `pack!(TypeName = InnerType)`
/// - `pack! { TypeName = InnerType }`
/// - `pack_err!(TypeName)`
/// - `pack_err!(TypeName = InnerType)`
/// - `pack_structural!` series same as above
pub struct PackPattern;

impl AnalyzePattern for PackPattern {
    fn contains(&self, content: &str) -> bool {
        content.contains("pack!")
            || content.contains("pack_err!")
            || content.contains("pack_structural!")
            || content.contains("pack_err_structural!")
    }

    fn analyze(&self, content: &str) -> Vec<AnalyzeItem> {
        let Ok(syntax) = syn::parse_file(content) else {
            return Vec::new();
        };

        let mut items = Vec::new();

        for item in &syntax.items {
            match item {
                // Top-level macro calls
                Item::Macro(m) => {
                    if let Some(name) = try_extract_pack_name(m) {
                        items.push(AnalyzeItem::local(String::new(), name));
                    }
                }
                // Macro calls inside inline modules
                Item::Mod(item_mod) => {
                    if let Some((_, nested)) = &item_mod.content {
                        for n in nested {
                            if let Item::Macro(m) = n
                                && let Some(name) = try_extract_pack_name(m)
                            {
                                items.push(AnalyzeItem::local(item_mod.ident.to_string(), name));
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

/// If the macro call is `pack!` / `pack_err!` / etc., extract the registered type name.
fn try_extract_pack_name(m: &syn::ItemMacro) -> Option<String> {
    let macro_name = m.mac.path.segments.last()?.ident.to_string();

    match macro_name.as_str() {
        "pack" | "pack_err" | "pack_structural" | "pack_err_structural" => {}
        _ => return None,
    }

    let tokens = &m.mac.tokens;

    // `pack!(T)` or `pack!(T = U)` — the first ident is the type name
    // Parse simply with syn
    if let Ok(ident) = syn::parse2::<syn::Ident>(tokens.clone()) {
        // pack!(TypeName) — just a single ident
        return Some(ident.to_string());
    }

    // Try to parse `Ident = Type`
    // Clone tokens first to avoid partial consumption
    let stream = tokens.clone();
    let mut iter = stream.into_iter();

    // Skip leading attributes/doc comments
    loop {
        if let proc_macro2::TokenTree::Ident(ident) = iter.next()? {
            // Found the first ident, this is the type name
            let type_name = ident.to_string();

            // Check if `=` follows
            if let Some(proc_macro2::TokenTree::Punct(p)) = iter.next()
                && p.as_char() == '='
            {
                // pack!(TypeName = InnerType)
                return Some(type_name);
            }

            // pack_err!(TypeName) — only a single ident
            return Some(type_name);
        }
    }
}
