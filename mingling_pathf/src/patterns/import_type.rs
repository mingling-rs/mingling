// Doc Not Optimize
//! The `ImportTypePattern` matches `import_type!` invocations and extracts the
//! generated `__mingling_import_*` namespace module.

use syn::Item;

use crate::pattern_analyzer::{AnalyzeItem, AnalyzePattern};

/// Matches `import_type!`.
///
/// Covered forms:
/// - `import_type!(std::io::Error)`
/// - `import_type!(ErrorIo = std::io::Error)`
pub struct ImportTypePattern;

impl AnalyzePattern for ImportTypePattern {
    fn contains(&self, content: &str) -> bool {
        content.contains("import_type!")
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

fn collect_from_item(item: &Item, current_mod: &str, items: &mut Vec<AnalyzeItem>) {
    match item {
        Item::Macro(m) => {
            let Some(last) = m.mac.path.segments.last() else {
                return;
            };
            if last.ident != "import_type" {
                return;
            }
            if let Some(module_name) = module_name_from_tokens(&m.mac.tokens) {
                items.push(AnalyzeItem::local_module(
                    current_mod.to_string(),
                    module_name,
                ));
            }
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

/// Computes the `__mingling_import_*` module name from an `import_type!` token
/// stream. The target path is taken after `=` for the aliased form; otherwise
/// the whole token stream is the fully-qualified path.
fn module_name_from_tokens(tokens: &proc_macro2::TokenStream) -> Option<String> {
    let stream = tokens.to_string();

    let type_path = stream.find('=').map_or_else(
        || {
            stream
                .trim()
                .trim_end_matches([')', '}'])
                .trim()
                .to_string()
        },
        |eq| {
            let after = stream[eq + 1..].trim();
            after.trim_end_matches([')', '}']).trim().to_string()
        },
    );

    let segments: Vec<&str> = type_path.split("::").collect();
    if segments.len() < 2 {
        return None;
    }

    let joined = segments
        .iter()
        .map(|s| s.trim().to_lowercase())
        .collect::<Vec<_>>()
        .join("_");

    Some(format!("__mingling_import_{joined}"))
}
