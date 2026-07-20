//! The `GroupPattern` matches the `group!` and `group_structural!` macros and
//! extracts the type name or alias defined within them.
//! This is used to track type groups for code generation or analysis.

use std::collections::HashMap;

use syn::Item;
use syn::UseTree;

use crate::pattern_analyzer::{AnalyzeItem, AnalyzePattern};

/// Matches the `group!` and `group_structural!` macros.
///
/// Covered forms:
/// - `group!(TypeName)`
/// - `group!(Alias = path::Type)`
/// - `group_structural!(TypeName)`
/// - `group_structural!(Alias = path::Type)`
pub struct GroupPattern;

impl AnalyzePattern for GroupPattern {
    fn contains(&self, content: &str) -> bool {
        content.contains("group!") || content.contains("group_structural!")
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
                    let Some(last) = m.mac.path.segments.last() else {
                        continue;
                    };
                    let macro_name = last.ident.to_string();
                    if macro_name != "group" && macro_name != "group_structural" {
                        continue;
                    }
                    if let Some(analyze_item) = extract_group_item(&m.mac.tokens, &imports, "") {
                        items.push(analyze_item);
                    }
                }
                Item::Mod(item_mod) => {
                    if let Some((_, nested)) = &item_mod.content {
                        // Collect `use` imports inside this module
                        let inner_imports = collect_use_imports(nested);

                        for n in nested {
                            if let Item::Macro(m) = n {
                                let Some(last) = m.mac.path.segments.last() else {
                                    continue;
                                };
                                let macro_name = last.ident.to_string();
                                if macro_name != "group" && macro_name != "group_structural" {
                                    continue;
                                }
                                if let Some(analyze_item) = extract_group_item(
                                    &m.mac.tokens,
                                    &inner_imports,
                                    &item_mod.ident.to_string(),
                                ) {
                                    items.push(analyze_item);
                                }
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

/// Extract an `AnalyzeItem` from the macro invocation tokens.
///
/// If the name matches a `use` import, resolves to the full foreign path.
/// For the `Alias = path::Type` form, returns a local item (the alias exists in-crate).
fn extract_group_item(
    tokens: &proc_macro2::TokenStream,
    imports: &HashMap<String, (String, String)>,
    current_mod: &str,
) -> Option<AnalyzeItem> {
    let name = extract_group_name(tokens)?;
    let is_aliased = has_equals_sign(tokens);

    if is_aliased {
        // `group!(Alias = path::Type)` — alias lives in-crate
        Some(AnalyzeItem::local(current_mod.to_string(), name))
    } else if let Some((module, _)) = imports.get(&name) {
        // `group!(TypeName)` where TypeName is imported via `use` — foreign
        Some(AnalyzeItem::foreign(module.clone(), name))
    } else {
        // `group!(LocalType)` — local type
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
            // Only handle non-`pub use` (regular imports)
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
            map.entry(alias).or_insert((module, name.ident.to_string()));
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
                .or_insert((module, rename.ident.to_string()));
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

/// Check whether the macro tokens contain `=`, indicating aliased form.
fn has_equals_sign(tokens: &proc_macro2::TokenStream) -> bool {
    let stream = tokens.clone();
    for token in stream {
        if let proc_macro2::TokenTree::Punct(p) = token
            && p.as_char() == '='
        {
            return true;
        }
    }
    false
}

/// Extract the alias / type name from the arguments of `group!`.
///
/// - `group!(ParseIntError)` → `ParseIntError`
/// - `group!(ErrorIo = std::io::Error)` → `ErrorIo`
fn extract_group_name(tokens: &proc_macro2::TokenStream) -> Option<String> {
    let stream = tokens.clone();
    let mut iter = stream.into_iter();

    loop {
        match iter.next()? {
            proc_macro2::TokenTree::Ident(ident) => {
                let name = ident.to_string();

                // Check if there is a `=` following
                let next = iter.next();
                match next {
                    Some(proc_macro2::TokenTree::Punct(p)) if p.as_char() == '=' => {
                        // group!(Alias = path::Type)
                        return Some(name);
                    }
                    _ => {
                        // group!(TypeName)
                        return Some(name);
                    }
                }
            }
            _ => continue,
        }
    }
}
