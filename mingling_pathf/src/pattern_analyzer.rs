// Doc Not Optimize
//! This module defines the core pattern analysis system used to parse and extract
//! importable/referenceable items (like structs, enums, functions, etc.) from Rust source files.
//!
//! It provides a pluggable architecture via the `AnalyzePattern` trait, allowing different
//! syntactic patterns to be registered and applied. Built-in patterns cover common structures
//! such as basic structs, groups, derives, chains, renderers, help, completion, and
//! dispatch patterns (both standard and clap-based).
//!
//! The entry points are:
//! - [`init()`] — creates a default `PatternAnalyzer` with all built-in patterns.
//! - [`PatternAnalyzer::analyze_file()`] / [`PatternAnalyzer::analyze_file_items()`] — run
//!   analysis on a single file.

use std::collections::HashSet;
use std::path::Path;

use crate::error::MinglingPathfinderError;
use crate::patterns::{
    ChainPattern, CommandPattern, CompletionPattern, DispatcherClapPattern, DispatcherPattern,
    GroupedDerivePattern, HelpPattern, ImportTypePattern, MetadataPattern, RendererPattern,
    StructuralPattern,
};

/// Creates a default `PatternAnalyzer` with all built-in patterns pre-registered.
#[must_use]
pub fn init() -> PatternAnalyzer {
    let mut analyzer = PatternAnalyzer::new();
    analyzer.add_pattern(ImportTypePattern);
    analyzer.add_pattern(GroupedDerivePattern);
    analyzer.add_pattern(StructuralPattern);
    analyzer.add_pattern(ChainPattern);
    analyzer.add_pattern(CommandPattern);
    analyzer.add_pattern(RendererPattern);
    analyzer.add_pattern(HelpPattern);
    analyzer.add_pattern(MetadataPattern);
    analyzer.add_pattern(CompletionPattern);
    analyzer.add_pattern(DispatcherPattern::new());
    analyzer.add_pattern(DispatcherClapPattern::new());
    analyzer
}

/// A single analysis item representing a parseable importable/referenceable item from code
#[derive(Debug, Clone)]
pub struct AnalyzeItem {
    /// The module path to which the item belongs, e.g. `"std::collections"`; empty string `""` if the item is in the root module
    pub module: String,
    /// The name of the item itself, e.g. `"HashMap"`, `"AnalyzeResult"`, etc.
    pub item_name: String,
    /// Whether the item is from an external crate (resolved via `use`), bypassing the file's own module path.
    pub is_foreign: bool,
    /// When `true`, this item is a module whose contents should be glob-imported (`::*`).
    pub is_module: bool,
}

impl AnalyzeItem {
    /// Creates a local `AnalyzeItem` (not foreign, will be prefixed with the file's module path).
    #[must_use]
    pub const fn local(module: String, item_name: String) -> Self {
        Self {
            module,
            item_name,
            is_foreign: false,
            is_module: false,
        }
    }

    /// Creates a local module item — generates a `use path::item_name::*;` glob import.
    #[must_use]
    pub const fn local_module(module: String, item_name: String) -> Self {
        Self {
            module,
            item_name,
            is_foreign: false,
            is_module: true,
        }
    }

    /// Creates a foreign `AnalyzeItem` (resolved via `use`, the `module` field is the full import path).
    #[must_use]
    pub const fn foreign(module: String, item_name: String) -> Self {
        Self {
            module,
            item_name,
            is_foreign: true,
            is_module: false,
        }
    }
}

/// Collection of analysis results
#[derive(Debug)]
pub struct AnalyzeResult {
    items: Vec<AnalyzeItem>,
}

impl AnalyzeResult {
    /// Creates an empty `AnalyzeResult` instance
    #[must_use]
    pub const fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Formats all items into a set of strings in the format `"::{module_path}::{item_name}"`
    #[must_use]
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

    /// Analyze the content and return all found `AnalyzeItem`s
    fn analyze(&self, content: &str) -> Vec<AnalyzeItem>;
}

/// A pattern analyzer that registers and runs multiple `AnalyzePattern` instances to parse
/// referenceable items from code.
#[derive(Default)]
pub struct PatternAnalyzer {
    patterns: Vec<Box<dyn AnalyzePattern>>,
}

impl PatternAnalyzer {
    /// Creates a new empty `PatternAnalyzer`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a new pattern for analysis.
    pub fn add_pattern(&mut self, pattern: impl AnalyzePattern + 'static) {
        self.patterns.push(Box::new(pattern));
    }

    /// Analyzes a single file and returns a formatted set of strings.
    ///
    /// # Errors
    ///
    /// Returns a [`MinglingPathfinderError`] if the file cannot be read.
    pub fn analyze_file(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<HashSet<String>, MinglingPathfinderError> {
        self.collect_items(path)
            .map(|items| AnalyzeResult { items }.into_formatted())
    }

    /// Analyzes a single file and returns the raw `Vec<AnalyzeItem>`.
    ///
    /// # Errors
    ///
    /// Returns a [`MinglingPathfinderError`] if the file cannot be read.
    pub fn analyze_file_items(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<Vec<AnalyzeItem>, MinglingPathfinderError> {
        self.collect_items(path)
    }

    fn collect_items(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<Vec<AnalyzeItem>, MinglingPathfinderError> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)?;

        let mut all_items = Vec::new();
        for pattern in &self.patterns {
            if pattern.contains(&content) {
                let items = pattern.analyze(&content);
                all_items.extend(items);
            }
        }

        Ok(all_items)
    }
}
