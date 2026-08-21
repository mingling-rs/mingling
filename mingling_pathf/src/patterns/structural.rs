// Doc Not Optimize
//! The `StructuralPattern` matches the `structural!` macro and
//! extracts the type name registered within it.
//! This is used to track structural-data types for code generation or analysis.

use std::collections::HashMap;

use syn::Item;
use syn::UseTree;

use crate::pattern_analyzer::{AnalyzeItem, AnalyzePattern};

/// Matches the `structural!` macro.
///
/// Covered forms:
/// - `structural!(TypeName)`
pub struct StructuralPattern;

impl AnalyzePattern for StructuralPattern {
    fn contains(&self, content: &str) -> bool {
        content.contains("structural!")
    }

    fn analyze(&self, content: &str) -> Vec<AnalyzeItem> {
        let Ok(syntax) = syn::parse_file(content) else {
            return Vec::new();
        };

        // Collect `use` imports at the file level
        let imports = collect_use_imports(&syntax.items);

        let mut items = Vec::new();

        for item in &syntax.items {
            match item {
                Item::Macro(m) => {
                    if is_structural_macro(&m.mac.path) {
                        if let Some(analyze_item) =
                            extract_structural_item(&m.mac.tokens, &imports, "")
                        {
                            items.push(analyze_item);
                        }
                    }
                }
                Item::Mod(item_mod) => {
                    if let Some((_, nested)) = &item_mod.content {
                        // Collect `use` imports inside this module
                        let inner_imports = collect_use_imports(nested);

                        for n in nested {
                            if let Item::Macro(m) = n
                                && is_structural_macro(&m.mac.path)
                                && let Some(analyze_item) = extract_structural_item(
                                    &m.mac.tokens,
                                    &inner_imports,
                                    &item_mod.ident.to_string(),
                                )
                            {
                                items.push(analyze_item);
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

/// Whether the macro path refers to `structural`.
fn is_structural_macro(path: &syn::Path) -> bool {
    path.segments
        .last()
        .is_some_and(|seg| seg.ident == "structural")
}

/// Extract an `AnalyzeItem` from the macro invocation tokens.
///
/// If the name matches a `use` import, resolves to the full foreign path.
fn extract_structural_item(
    tokens: &proc_macro2::TokenStream,
    imports: &HashMap<String, (String, String)>,
    current_mod: &str,
) -> Option<AnalyzeItem> {
    let name = extract_structural_name(tokens)?;

    if let Some((module, _)) = imports.get(&name) {
        // `structural!(TypeName)` where TypeName is imported via `use` — foreign
        Some(AnalyzeItem::foreign(module.clone(), name))
    } else {
        // `structural!(LocalType)` — local type
        Some(AnalyzeItem::local(current_mod.to_string(), name))
    }
}

/// Collect `use` imports from a list of top-level items.
///
/// Returns a map of `short_name → (module_path, short_name)`.
/// e.g. `use cargo_metadata::CompilerMessage;` → `"CompilerMessage" → ("cargo_metadata", "CompilerMessage")`
fn collect_use_imports(items: &[syn::Item]) -> HashMap<String, (String, String)> {
    let mut map = HashMap::new();
    for item in items {
        if let Item::Use(use_item) = item {
            collect_from_use_tree(&use_item.tree, "", &mut map);
        }
    }
    map
}

/// Recursively traverse a `UseTree` and collect named imports.
fn collect_from_use_tree(
    tree: &UseTree,
    prefix: &str,
    map: &mut HashMap<String, (String, String)>,
) {
    match tree {
        UseTree::Name(name) => {
            let module = prefix.to_string();
            let alias = name.ident.to_string();
            map.entry(alias)
                .or_insert_with(|| (module, name.ident.to_string()));
        }
        UseTree::Path(use_path) => {
            let new_prefix = if prefix.is_empty() {
                use_path.ident.to_string()
            } else {
                format!("{}::{}", prefix, use_path.ident)
            };
            collect_from_use_tree(&use_path.tree, &new_prefix, map);
        }
        UseTree::Rename(rename) => {
            let module = prefix.to_string();
            let alias = rename.ident.to_string();
            map.entry(alias)
                .or_insert_with(|| (module, rename.ident.to_string()));
        }
        UseTree::Glob(_) => {
            // `use path::*;` — skip glob imports
        }
        UseTree::Group(group) => {
            for item in &group.items {
                collect_from_use_tree(item, prefix, map);
            }
        }
    }
}

/// Extract the type name from the arguments of `structural!`.
///
/// - `structural!(ExternalType)` → `ExternalType`
fn extract_structural_name(tokens: &proc_macro2::TokenStream) -> Option<String> {
    let stream = tokens.clone();
    stream.into_iter().find_map(|token| match token {
        proc_macro2::TokenTree::Ident(ident) => Some(ident.to_string()),
        _ => None,
    })
}
