use syn::Item;

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
                    if let Some(name) = extract_group_name(&m.mac.tokens) {
                        items.push(AnalyzeItem {
                            module: String::new(),
                            item_name: name,
                        });
                    }
                }
                Item::Mod(item_mod) => {
                    if let Some((_, nested)) = &item_mod.content {
                        for n in nested {
                            if let Item::Macro(m) = n {
                                let Some(last) = m.mac.path.segments.last() else {
                                    continue;
                                };
                                let macro_name = last.ident.to_string();
                                if macro_name != "group" && macro_name != "group_structural" {
                                    continue;
                                }
                                if let Some(name) = extract_group_name(&m.mac.tokens) {
                                    items.push(AnalyzeItem {
                                        module: item_mod.ident.to_string(),
                                        item_name: name,
                                    });
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
