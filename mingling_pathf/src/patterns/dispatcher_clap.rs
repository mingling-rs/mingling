//! The `DispatcherClapPattern` matches structs annotated with `#[dispatcher_clap(...)]` and
//! extracts key items for code generation or analysis:
//! - The entry struct name (always)
//! - The dispatcher command struct (`CMD*`, always)
//! - The error type, if `error = ErrorType` is specified
//! - The help internal struct, if `help = true` is specified
//! - The `__internal_dispatcher_*` dispatch tree static, if `use_dispatch_tree` is enabled
//!
//! Supported forms:
//! - `#[dispatcher_clap("greet", CMDGreet)] struct EntryGreet { ... }`
//! - `#[dispatcher_clap("greet", CMDGreet, error = ErrorGreet)] struct EntryGreet { ... }`
//! - `#[dispatcher_clap("greet", CMDGreet, help = true)] struct EntryGreet { ... }`
//! - `#[dispatcher_clap("greet", CMDGreet, error = ErrorGreet, help = true)] struct EntryGreet { ... }`

use syn::Item;

use crate::pattern_analyzer::{AnalyzeItem, AnalyzePattern};

/// Match structs annotated with `#[dispatcher_clap(...)]`, extracting:
/// - The entry type (struct name, always)
/// - The dispatcher struct (`CMD*`, always)
/// - The error type, if `error = ErrorType` is specified
/// - The help internal struct, if `help = true` is specified
/// - `__internal_dispatcher_*` — dispatch tree static (when `use_dispatch_tree` is true)
///
/// Covers forms:
/// - `#[dispatcher_clap("greet", CMDGreet)] struct EntryGreet { ... }`
/// - `#[dispatcher_clap("greet", CMDGreet, error = ErrorGreet)] struct EntryGreet { ... }`
/// - `#[dispatcher_clap("greet", CMDGreet, help = true)] struct EntryGreet { ... }`
/// - `#[dispatcher_clap("greet", CMDGreet, error = ErrorGreet, help = true)] struct EntryGreet { ... }`
pub struct DispatcherClapPattern {
    /// Whether to include the `__internal_dispatcher_*` dispatch tree static in the analysis.
    pub use_dispatch_tree: bool,
}

impl DispatcherClapPattern {
    /// Creates a new `DispatcherClapPattern` with the given configuration.
    ///
    /// # Parameters
    /// - `use_dispatch_tree`: When `true`, enables analysis of the `__internal_dispatcher_*`
    ///   static dispatch tree for each matched command.
    pub fn new(use_dispatch_tree: bool) -> Self {
        Self { use_dispatch_tree }
    }
}

impl AnalyzePattern for DispatcherClapPattern {
    fn contains(&self, content: &str) -> bool {
        content.contains("dispatcher_clap(")
    }

    fn analyze(&self, content: &str) -> Vec<AnalyzeItem> {
        let Ok(syntax) = syn::parse_file(content) else {
            return Vec::new();
        };

        let mut items = Vec::new();

        for item in &syntax.items {
            match item {
                Item::Struct(s) if has_attr(&s.attrs, "dispatcher_clap") => {
                    // Entry type (struct name) — always
                    let entry_name = s.ident.to_string();
                    items.push(AnalyzeItem {
                        module: String::new(),
                        item_name: entry_name.clone(),
                    });

                    // Parse the attribute to extract CMD, error, and help info
                    if let Some(attr) = s.attrs.iter().find(|a| {
                        a.path()
                            .segments
                            .last()
                            .is_some_and(|seg| seg.ident == "dispatcher_clap")
                    }) {
                        let args = attr.meta.require_list().ok();
                        let args_str = args.map(|l| l.tokens.to_string()).unwrap_or_default();
                        let parsed = parse_dispatcher_clap_args(&args_str);

                        // CMD type — always
                        if let Some(ref cmd) = parsed.cmd_type {
                            items.push(AnalyzeItem {
                                module: String::new(),
                                item_name: cmd.clone(),
                            });
                        }

                        // Error type — if error = TypeName
                        if let Some(ref err) = parsed.error_type {
                            items.push(AnalyzeItem {
                                module: String::new(),
                                item_name: err.clone(),
                            });
                        }

                        // Help internal struct — if help = true
                        if parsed.help_enabled
                            && let Some(ref cmd) = parsed.cmd_type
                        {
                            let help_fn = format!("__{}_help", just_fmt::snake_case!(cmd));
                            let help_struct =
                                format!("__internal_help_{}", just_fmt::snake_case!(&help_fn));
                            items.push(AnalyzeItem {
                                module: String::new(),
                                item_name: help_struct,
                            });
                        }

                        // __internal_dispatcher_* — when configured
                        if self.use_dispatch_tree
                            && let Some(ref cmd_name) = parsed.cmd_name
                        {
                            let internal_name = format!(
                                "__internal_dispatcher_{}",
                                just_fmt::snake_case!(cmd_name)
                            );
                            items.push(AnalyzeItem {
                                module: String::new(),
                                item_name: internal_name,
                            });
                        }
                    }
                }
                Item::Mod(item_mod) => {
                    if let Some((_, nested)) = &item_mod.content {
                        for n in nested {
                            if let Item::Struct(s) = n
                                && has_attr(&s.attrs, "dispatcher_clap")
                            {
                                let entry_name = s.ident.to_string();
                                items.push(AnalyzeItem {
                                    module: item_mod.ident.to_string(),
                                    item_name: entry_name.clone(),
                                });

                                if let Some(attr) = s.attrs.iter().find(|a| {
                                    a.path()
                                        .segments
                                        .last()
                                        .is_some_and(|seg| seg.ident == "dispatcher_clap")
                                }) {
                                    let args = attr.meta.require_list().ok();
                                    let args_str =
                                        args.map(|l| l.tokens.to_string()).unwrap_or_default();
                                    let parsed = parse_dispatcher_clap_args(&args_str);

                                    if let Some(ref cmd) = parsed.cmd_type {
                                        items.push(AnalyzeItem {
                                            module: item_mod.ident.to_string(),
                                            item_name: cmd.clone(),
                                        });
                                    }

                                    if let Some(ref err) = parsed.error_type {
                                        items.push(AnalyzeItem {
                                            module: item_mod.ident.to_string(),
                                            item_name: err.clone(),
                                        });
                                    }

                                    // Help internal struct — same naming rule as root level
                                    if parsed.help_enabled
                                        && let Some(ref cmd) = parsed.cmd_type
                                    {
                                        let help_fn =
                                            format!("__{}_help", just_fmt::snake_case!(cmd));
                                        let help_struct = format!(
                                            "__internal_help_{}",
                                            just_fmt::snake_case!(&help_fn)
                                        );
                                        items.push(AnalyzeItem {
                                            module: item_mod.ident.to_string(),
                                            item_name: help_struct,
                                        });
                                    }

                                    // __internal_dispatcher_* — when configured
                                    if self.use_dispatch_tree
                                        && let Some(ref cmd_name) = parsed.cmd_name
                                    {
                                        let internal_name = format!(
                                            "__internal_dispatcher_{}",
                                            just_fmt::snake_case!(cmd_name)
                                        );
                                        items.push(AnalyzeItem {
                                            module: item_mod.ident.to_string(),
                                            item_name: internal_name,
                                        });
                                    }
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

struct ParsedClapArgs {
    cmd_name: Option<String>,
    cmd_type: Option<String>,
    error_type: Option<String>,
    help_enabled: bool,
}

/// Parse `#[dispatcher_clap("cmd", CMDType, error = ErrorType, help = true)]` arguments.
fn parse_dispatcher_clap_args(args: &str) -> ParsedClapArgs {
    let mut cmd_name = None;
    let mut cmd_type = None;
    let mut error_type = None;
    let mut help_enabled = false;

    let args = args.trim();

    // Extract the first quoted string (the command name)
    let after_cmd = if let Some(start) = args.find('"') {
        let after_open = &args[start + 1..];
        if let Some(end) = after_open.find('"') {
            cmd_name = Some(after_open[..end].to_string());
            after_open[end + 1..].trim()
        } else {
            args
        }
    } else {
        args
    };

    // Split by commas and parse each part
    for part in after_cmd.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        // Skip the command name (first quoted string should already be removed)
        if part.starts_with('"') {
            continue;
        }

        // Check for key = value
        if let Some(eq_idx) = part.find('=') {
            let key = part[..eq_idx].trim();
            let value = part[eq_idx + 1..].trim();
            let value = value.trim_end_matches([')', ']']).trim();

            match key {
                "error" => {
                    error_type = Some(value.to_string());
                }
                "help" => {
                    help_enabled = value == "true";
                }
                _ => {}
            }
        } else {
            // Bare ident — the CMD type
            let clean = part.trim_end_matches([')', ']']).trim();
            if !clean.is_empty() && cmd_type.is_none() {
                cmd_type = Some(clean.to_string());
            }
        }
    }

    ParsedClapArgs {
        cmd_name,
        cmd_type,
        error_type,
        help_enabled,
    }
}

fn has_attr(attrs: &[syn::Attribute], name: &str) -> bool {
    attrs
        .iter()
        .any(|a| a.path().segments.last().is_some_and(|s| s.ident == name))
}
