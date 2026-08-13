// Doc Not Optimize
//! The `CommandPattern` matches functions annotated with `#[command]` and
//! extracts the generated hidden module name (`__command_<fn>_module`).
//!
//! Supported forms:
//! ```rust,ignore
//! #[command]
//! fn greet(args: Vec<String>) -> Next { ... }
//!
//! #[command(node = "foo.foo-bar")]
//! fn greet(args: Vec<String>) -> Next { ... }
//!
//! #[command(name = CMDGreet, entry = EntryGreet)]
//! fn greet(args: Vec<String>) -> Next { ... }
//!
//! #[command(node = "greet", name = CMDGreet, entry = EntryGreet, buffer)]
//! fn greet(args: Vec<String>) { ... }
//! ```

use syn::Item;

use crate::pattern_analyzer::{AnalyzeItem, AnalyzePattern};

/// Match `#[command]` functions.
///
/// The `#[command]` macro generates a hidden `__command_<fn>_module` that
/// re-exports all internal types (`Entry*`, `CMD*`, chain struct, dispatcher
/// static). This pattern tracks that module; the build system generates a
/// glob re-export `use path::__command_<fn>_module::*;` to bring everything
/// into scope.
pub struct CommandPattern;

impl AnalyzePattern for CommandPattern {
    fn contains(&self, content: &str) -> bool {
        content.contains("command")
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
        Item::Fn(f) if has_command_attr(&f.attrs) => {
            let fn_name = f.sig.ident.to_string();
            let mod_name = format!("__command_{}_module", &fn_name);
            items.push(AnalyzeItem::local_module(current_mod.to_string(), mod_name));
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

fn has_command_attr(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|a| {
        a.path()
            .segments
            .last()
            .is_some_and(|s| s.ident == "command")
    })
}
