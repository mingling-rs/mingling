use syn::Item;

use crate::pattern_analyzer::{AnalyzeItem, AnalyzePattern};

/// Matches the `dispatcher!` macro, extracts the entry type name.
///
/// Supported forms:
/// - `dispatcher!("greet", CMDGreet => EntryGreet)` — explicit
/// - `dispatcher!("greet")` — implicit, infers EntryType
/// - `dispatcher! { ... }` — with braces
pub struct DispatcherPattern;

impl AnalyzePattern for DispatcherPattern {
    fn contains(&self, content: &str) -> bool {
        content.contains("dispatcher!")
    }

    fn analyze(&self, content: &str) -> Vec<AnalyzeItem> {
        let Ok(syntax) = syn::parse_file(content) else {
            return Vec::new();
        };

        let mut items = Vec::new();

        for item in &syntax.items {
            match item {
                Item::Macro(m) => {
                    let macro_name = macro_simple_name(m);
                    if macro_name != "dispatcher" {
                        continue;
                    }
                    if let Some(entry_name) = extract_dispatcher_entry(&m.mac.tokens) {
                        items.push(AnalyzeItem {
                            module: String::new(),
                            item_name: entry_name,
                        });
                    }
                }
                Item::Mod(item_mod) => {
                    if let Some((_, nested)) = &item_mod.content {
                        for n in nested {
                            if let Item::Macro(m) = n {
                                if macro_simple_name(m) != "dispatcher" {
                                    continue;
                                }
                                if let Some(entry_name) = extract_dispatcher_entry(&m.mac.tokens) {
                                    items.push(AnalyzeItem {
                                        module: item_mod.ident.to_string(),
                                        item_name: entry_name,
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

fn macro_simple_name(m: &syn::ItemMacro) -> String {
    m.mac
        .path
        .segments
        .last()
        .map(|s| s.ident.to_string())
        .unwrap_or_default()
}

/// Extracts the entry type name from the `dispatcher!` macro arguments.
///
/// Input examples:
/// - `"greet", CMDGreet => EntryGreet` → `EntryGreet`
/// - `"remote.add"` → `EntryRemoteAdd` (implicit inference)
fn extract_dispatcher_entry(tokens: &proc_macro2::TokenStream) -> Option<String> {
    let stream = tokens.to_string();

    // Explicit form: look for `=>`
    if let Some(arrow_idx) = stream.find("=>") {
        let after_arrow = stream[arrow_idx + 2..].trim();
        let entry_name = after_arrow
            .split(|c: char| c.is_whitespace() || c == ',' || c == ')' || c == '}')
            .next()?;
        if !entry_name.is_empty() {
            return Some(entry_name.trim().to_string());
        }
    }

    // Implicit form: infer from command name
    let stream = stream.trim();
    if let Some(start) = stream.find('"') {
        let rest = &stream[start + 1..];
        let cmd_name = rest.split('"').next()?;
        let entry = format!("Entry{}", to_pascal_case(cmd_name));
        Some(entry)
    } else {
        None
    }
}

fn to_pascal_case(s: &str) -> String {
    s.split(['-', '_', '.'])
        .filter(|s| !s.is_empty())
        .map(|s| {
            let mut c = s.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect()
}
