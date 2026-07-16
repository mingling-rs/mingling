//! A module for mapping Rust module paths to source files.
//!
//! This module provides functionality to analyze the module structure of a Rust crate
//! and determine the effective module path for each source file, taking into account
//! `pub use` re-exports that can cause modules to be hoisted to parent paths.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use syn::{Item, UseTree};

use crate::error::MinglingPathfinderError;

/// Represents a mapping from a module path to a source file.
///
/// Each instance represents a source file and its corresponding
/// effective module path (e.g., `crate::foo::bar`).
#[derive(Debug, Clone)]
pub struct MappingItem {
    /// The path of the source file (relative to the crate root, with `./` prefix).
    file_path: PathBuf,

    /// The effective module path corresponding to this file (e.g., `"crate::foo::bar"`).
    module_path: String,
}

impl MappingItem {
    /// Returns the path of the source file (relative to the crate root, with `./` prefix).
    pub fn file_path(&self) -> &Path {
        &self.file_path
    }

    /// Returns the effective module path corresponding to this file (e.g., `"crate::foo::bar"`).
    pub fn module_path(&self) -> &str {
        &self.module_path
    }
}

/// A mapping from module paths to source files.
///
/// Built by the [`analyze`] function. Records the effective module path
/// for each source file in a crate, taking into account `pub use` re-exports.
///
/// Can be iterated over via [`IntoIterator`] to get each [`MappingItem`].
#[derive(Debug)]
pub struct ModulePathMapping {
    /// A list of mappings from source files to their effective module paths.
    items: Vec<MappingItem>,
}

/// Analyzes the module structure of a crate and returns the effective module path for each source file.
///
/// `crate_dir` is the crate root directory (i.e., the directory containing `Cargo.toml`).
pub fn analyze(crate_dir: &Path) -> Result<ModulePathMapping, MinglingPathfinderError> {
    let src_dir = crate_dir.join("src");
    if !src_dir.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("src/ directory not found at {}", src_dir.display()),
        )
        .into());
    }

    let entry = find_entry_point(&src_dir)?;
    let mut ctx = Context::new(crate_dir);

    // Phase 1: Traverse the module tree, recording direct paths and re-export relationships
    let root_path = "crate".to_string();
    build_direct_paths(&entry, &root_path, &mut ctx)?;

    // Phase 2: Propagate effective paths top-down
    // The effective path of the root file is "crate"
    ctx.effective_paths
        .insert(entry.clone(), "crate".to_string());
    propagate_children(&entry, &mut ctx);

    // Assemble the results
    let items = ctx
        .effective_paths
        .iter()
        .map(|(file, module_path)| MappingItem {
            file_path: ctx.relative_path(file),
            module_path: module_path.clone(),
        })
        .collect();

    Ok(ModulePathMapping { items })
}

/// Internal context used during analysis to maintain module paths, child module relationships, and re-export information.
struct Context {
    /// The crate root directory (i.e., the directory containing Cargo.toml)
    crate_dir: PathBuf,
    /// Mapping from source files to their direct module paths (e.g., `"crate::foo::bar"`)
    direct_paths: HashMap<PathBuf, String>,
    /// Mapping from source files to their effective module paths (after considering `pub use` re-exports)
    effective_paths: HashMap<PathBuf, String>,
    /// Mapping from source files to their child module lists
    children: HashMap<PathBuf, Vec<ChildModule>>,
    /// Mapping from source files to the list of module names re-exported via `pub use`
    reexports: HashMap<PathBuf, Vec<String>>,
    /// Set of source files already visited, used to prevent cycles
    visited: std::collections::HashSet<PathBuf>,
}

#[derive(Clone)]
struct ChildModule {
    name: String,
    file: PathBuf,
}

impl Context {
    fn new(crate_dir: &Path) -> Self {
        Self {
            crate_dir: crate_dir.to_path_buf(),
            direct_paths: HashMap::new(),
            effective_paths: HashMap::new(),
            children: HashMap::new(),
            reexports: HashMap::new(),
            visited: std::collections::HashSet::new(),
        }
    }

    fn relative_path(&self, abs: &Path) -> PathBuf {
        if let Ok(rel) = abs.strip_prefix(&self.crate_dir) {
            PathBuf::from("./").join(rel)
        } else {
            abs.to_path_buf()
        }
    }
}

/// Finds the entry point file of a crate.
///
/// The resolution order is:
/// 1. First, look for `src/main.rs` or `src/lib.rs`.
/// 2. If neither exists, look for any `.rs` file in the `src/bin/` directory (binary entry points).
///
/// # Arguments
/// - `src_dir`: The path to the crate's `src/` directory.
///
/// # Returns
/// Returns the absolute path to the first entry point file found.
///
/// # Errors
/// Returns [`MinglingPathfinderError::NoEntryPointFound`] if no entry point file is found.
fn find_entry_point(src_dir: &Path) -> Result<PathBuf, MinglingPathfinderError> {
    // First, look for src/main.rs or src/lib.rs
    for name in &["main.rs", "lib.rs"] {
        let path = src_dir.join(name);
        if path.is_file() {
            return Ok(path);
        }
    }

    // Next, look for .rs files in src/bin/
    let bin_dir = src_dir.join("bin");
    if bin_dir.is_dir() {
        for entry in std::fs::read_dir(&bin_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "rs") {
                return Ok(path);
            }
        }
    }

    // No entry point found, return an error
    Err(MinglingPathfinderError::NoEntryPointFound)
}

/// Recursively builds direct module paths for source files, recording child modules and `pub use` re-exports.
///
/// This is the first phase of the analysis, responsible for:
/// - Marking files as visited (to prevent circular dependencies)
/// - Recording the direct module path of each file (e.g., `crate::foo::bar`)
/// - Parsing file contents to extract:
///   - Child modules declared via `mod xxx;` (including their visibility)
///   - Module names re-exported via `pub use`
/// - Recursively processing each child module file to build the complete module tree path
///
/// # Arguments
/// - `file`: The absolute path of the source file currently being processed
/// - `module_path`: The direct module path corresponding to the current file (e.g., `"crate::foo"`)
/// - `ctx`: The context used to store paths, child modules, and re-export information
fn build_direct_paths(
    file: &Path,
    module_path: &str,
    ctx: &mut Context,
) -> Result<(), MinglingPathfinderError> {
    // If the current file has already been visited, skip it (prevent infinite recursion due to circular references)
    if !ctx.visited.insert(file.to_path_buf()) {
        return Ok(());
    }

    // Record the direct module path for the current file
    ctx.direct_paths
        .insert(file.to_path_buf(), module_path.to_string());

    // Read and parse the source file into a syntax tree
    let content = std::fs::read_to_string(file)?;
    let syntax = syn::parse_file(&content).map_err(|e| MinglingPathfinderError::SynError {
        path: file.to_path_buf(),
        message: e.to_string(),
    })?;

    // Store child module information: `(module_name, is_public)`
    let mut sub_modules: Vec<(String, bool)> = Vec::new();
    // Store module names re-exported via `pub use`
    let mut reexports: Vec<String> = Vec::new();

    // Iterate over top-level items in the syntax tree
    for item in &syntax.items {
        match item {
            // Handle `mod xxx;` (non-inline modules, i.e., those corresponding to external files)
            Item::Mod(item_mod) if item_mod.semi.is_some() => {
                let is_pub = matches!(item_mod.vis, syn::Visibility::Public(_));
                sub_modules.push((item_mod.ident.to_string(), is_pub));
            }
            // Handle `pub use xxx;` re-exports
            Item::Use(item_use) => {
                if matches!(item_use.vis, syn::Visibility::Public(_)) {
                    collect_reexports(&item_use.tree, &mut reexports);
                }
            }
            _ => {}
        }
    }

    // If there are re-exports, record them in the context
    if !reexports.is_empty() {
        ctx.reexports.insert(file.to_path_buf(), reexports);
    }

    let mut children = Vec::new();

    // Recursively process each child module
    for (name, _is_pub) in &sub_modules {
        // Resolve the file path corresponding to the child module
        let child_path = resolve_module_file(file, name)?;
        // Construct the child module's direct path: `parent_path::child_module_name`
        let child_direct = format!("{module_path}::{name}");
        // Recursively build paths for the child module
        build_direct_paths(&child_path, &child_direct, ctx)?;
        children.push(ChildModule {
            name: name.clone(),
            file: child_path,
        });
    }

    // If there are children, record them in the context
    if !children.is_empty() {
        ctx.children.insert(file.to_path_buf(), children);
    }

    Ok(())
}

/// Collects re-exported module names from a `use` syntax tree (i.e., `pub use`).
///
/// Only recognizes `pub use X::*;` and `pub use X;` forms (which hoist the entire module).
fn collect_reexports(tree: &UseTree, results: &mut Vec<String>) {
    match tree {
        UseTree::Path(use_path) => {
            if matches!(use_path.tree.as_ref(), UseTree::Glob(_)) {
                results.push(use_path.ident.to_string());
            }
        }
        UseTree::Name(use_name) => {
            results.push(use_name.ident.to_string());
        }
        UseTree::Group(group) => {
            for item in &group.items {
                collect_reexports(item, results);
            }
        }
        UseTree::Rename(rename) => {
            results.push(rename.ident.to_string());
        }
        _ => {}
    }
}

/// Resolves the source file path for a child module based on the parent file's path and the child module's name.
///
/// # Resolution Rules
///
/// The Rust module resolution rules are as follows:
/// - If the parent file is `mod.rs`, the module base directory is the parent file's directory.
/// - If the parent file is `main.rs` or `lib.rs` (i.e., the crate root entry point), the module base directory is also the parent file's directory,
///   because the crate root directly corresponds to the `src/` directory.
/// - For other files (e.g., `aaa.rs`), the module base directory is the directory with the same name as the file (without extension)
///   located under the parent file's directory (e.g., `aaa/`).
///
/// After determining the module base directory based on the rules above, the following candidate paths are tried in order:
/// 1. `{module_base}/{module_name}.rs` (single-file module)
/// 2. `{module_base}/{module_name}/mod.rs` (directory module)
///
/// # Arguments
/// - `parent_file`: The absolute path of the parent module source file.
/// - `module_name`: The name of the child module.
///
/// # Returns
/// Returns the absolute path to the child module's source file.
///
/// # Errors
/// Returns [`MinglingPathfinderError::ModuleNotFound`] if none of the candidate paths exist.
fn resolve_module_file(
    parent_file: &Path,
    module_name: &str,
) -> Result<PathBuf, MinglingPathfinderError> {
    let parent_dir = parent_file.parent().unwrap();
    let file_stem = parent_file
        .file_stem()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or("");

    // Rust module resolution rules:
    // - `mod.rs` → module base is the parent directory
    // - `main.rs` / `lib.rs` → crate root, module base is directly src/
    // - `aaa.rs` → module base is `src/aaa/`
    let module_base = if file_stem == "mod" || file_stem == "main" || file_stem == "lib" {
        parent_dir.to_path_buf()
    } else {
        parent_dir.join(file_stem)
    };

    let candidates = [
        module_base.join(format!("{module_name}.rs")),
        module_base.join(module_name).join("mod.rs"),
    ];

    for path in &candidates {
        if path.is_file() {
            return Ok(path.clone());
        }
    }

    Err(MinglingPathfinderError::ModuleNotFound {
        parent: parent_file.to_path_buf(),
        module_name: module_name.to_string(),
    })
}

/// Starting from the parent file, recursively compute the effective paths of all child files.
///
/// Core rules:
/// - The effective path of a child file = parent effective path + "::" + child module name
/// - However, if the parent file re-exports the child module via `pub use`,
///   then the effective path of the child file = parent effective path (the module name is not appended, i.e., it is hoisted to the parent level)
fn propagate_children(parent_file: &Path, ctx: &mut Context) {
    let parent_effective = ctx
        .effective_paths
        .get(parent_file)
        .cloned()
        .unwrap_or_else(|| "crate".to_string());

    let reexported = ctx.reexports.get(parent_file).cloned().unwrap_or_default();

    let Some(children) = ctx.children.get(parent_file).cloned() else {
        return;
    };

    for child in &children {
        let effective = if reexported.contains(&child.name) {
            // Re-exported: hoist to parent level, do not append module name
            parent_effective.clone()
        } else {
            format!("{}::{}", parent_effective, child.name)
        };

        ctx.effective_paths.insert(child.file.clone(), effective);
        propagate_children(&child.file, ctx);
    }
}

impl IntoIterator for ModulePathMapping {
    type Item = MappingItem;
    type IntoIter = std::vec::IntoIter<MappingItem>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}
