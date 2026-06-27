use std::collections::HashSet;
use std::path::Path;

use crate::error::MinglingPathfinderError;

/// Top-level convenience function: creates a default PatternAnalyzer with all built-in patterns pre-registered
pub fn init() -> PatternAnalyzer {
    let mut analyzer = PatternAnalyzer::new();
    macro_rules! __register {
        ( $($pattern:ident),* $(,)? ) => {
            $(
                analyzer.add_pattern(crate::patterns::$pattern);
            )*
        };
    }

    __register![
        BasicStructPattern,
    ];

    analyzer
}

/// A single analysis item representing a parseable importable/referenceable item from code
#[derive(Debug, Clone)]
pub struct AnalyzeItem {
    /// The module path to which the item belongs, e.g. `"std::collections"`; empty string `""` if the item is in the root module
    pub module: String,
    /// The name of the item itself, e.g. `"HashMap"`, `"AnalyzeResult"`, etc.
    pub item_name: String,
}

/// Collection of analysis results
#[derive(Debug)]
pub struct AnalyzeResult {
    items: Vec<AnalyzeItem>,
}

impl AnalyzeResult {
    /// Creates an empty `AnalyzeResult` instance
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Formats all items into a set of strings in the format `"::{module_path}::{item_name}"`
    pub fn into_formatted(self) -> HashSet<String> {
        self.items
            .into_iter()
            .map(|item| {
                if item.module.is_empty() {
                    format!("::{}", item.item_name)
                } else {
                    format!("::{}::{}", item.module, item.item_name)
                }
            })
            .collect()
    }
}

impl Default for AnalyzeResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Extension point trait — one independent implementation per syntax kind
pub trait AnalyzePattern {
    /// Quickly determine whether the file content contains an analyzable item
    fn contains(&self, content: &str) -> bool;

    /// Analyze the content and return all found AnalyzeItems
    fn analyze(&self, content: &str) -> Vec<AnalyzeItem>;
}

/// A pattern analyzer that registers and runs multiple `AnalyzePattern` instances to parse
/// referenceable items from code.
///
/// Internally maintains a `Vec<Box<dyn AnalyzePattern>>` for dynamic dispatch, supporting
/// runtime registration of custom patterns. The `Default` derive provides an empty analyzer
/// instance.
///
/// # Fields
/// - `patterns`: A list of registered analysis patterns, each responsible for detecting and
///   extracting a specific type of analyzable item (e.g., structs, functions).
#[derive(Default)]
pub struct PatternAnalyzer {
    /// A list of registered analysis patterns, each responsible for detecting and extracting
    /// a specific type of analyzable item.
    patterns: Vec<Box<dyn AnalyzePattern>>,
}

impl PatternAnalyzer {
    /// Creates a new `PatternAnalyzer` instance.
    ///
    /// Internally calls `Default::default()` directly, initially containing no registered patterns.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers an analysis pattern with the current analyzer.
    ///
    /// The pattern is wrapped in a `Box` and stored in the internal pattern list,
    /// and will be used to scan and analyze file content in subsequent calls to `analyze_file`.
    ///
    /// # Parameters
    /// - `pattern` —— A type instance implementing the `AnalyzePattern` trait,
    ///   responsible for detecting and extracting a specific syntactic structure (e.g., structs, functions).
    pub fn add_pattern(&mut self, pattern: impl AnalyzePattern + 'static) {
        self.patterns.push(Box::new(pattern));
    }

    /// Analyzes a single file and returns a formatted set of strings.
    ///
    /// This method reads the content of the file at the specified path, then uses each registered
    /// pattern to analyze the content in turn. Only patterns whose `contains` method returns `true`
    /// will trigger the subsequent `analyze` call. All extracted `AnalyzeItem`s are merged and
    /// converted into a formatted `HashSet<String>`.
    ///
    /// # Parameters
    /// - `path` —— The path to the file to analyze, supporting any type that implements `AsRef<Path>`.
    ///
    /// # Returns
    /// - `Ok(HashSet<String>)` —— On success, returns a formatted set of strings, each in the form `"::module_path::item_name"`.
    /// - `Err(MinglingPathfinderError)` —— If the file cannot be read, returns the corresponding I/O error wrapper.
    pub fn analyze_file(&self, path: impl AsRef<Path>) -> Result<HashSet<String>, MinglingPathfinderError> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)?;

        let mut all_items = Vec::new();
        for pattern in &self.patterns {
            if pattern.contains(&content) {
                let items = pattern.analyze(&content);
                all_items.extend(items);
            }
        }

        let result = AnalyzeResult { items: all_items };
        Ok(result.into_formatted())
    }
}
