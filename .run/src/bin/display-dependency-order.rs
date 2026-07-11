use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Parse Cargo.toml content and return dependency names that start with `prefix`.
fn parse_mingling_deps(content: &str, prefix: &str) -> Vec<String> {
    let value: toml::Value = match content.parse() {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    let mut names = Vec::new();

    // Check [dependencies]
    if let Some(deps) = value.get("dependencies").and_then(|d| d.as_table()) {
        for key in deps.keys() {
            if key.starts_with(prefix) {
                names.push(key.clone());
            }
        }
    }

    // Check [build-dependencies]
    if let Some(deps) = value.get("build-dependencies").and_then(|d| d.as_table()) {
        for key in deps.keys() {
            if key.starts_with(prefix) {
                names.push(key.clone());
            }
        }
    }

    names
}

/// Read workspace members from the root Cargo.toml.
fn get_workspace_members(workspace_root: &std::path::Path) -> Vec<String> {
    let cargo_path = workspace_root.join("Cargo.toml");
    let content = match std::fs::read_to_string(&cargo_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let value: toml::Value = match content.parse() {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    value
        .get("workspace")
        .and_then(|w| w.get("members"))
        .and_then(|m| m.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

/// Hierarchical topological sort (process layer by layer, sort siblings alphabetically).
///
/// `dep_map` maps each crate to the list of crates it depends on.
/// Returns the dependency order (dependent crates come first, dependents come later),
/// with crates at the same layer (which can be built in parallel) sorted alphabetically.
fn topological_sort(
    all_crates: &HashSet<String>,
    dep_map: &HashMap<String, Vec<String>>,
) -> Vec<String> {
    // in_degree[crate] = number of remaining mingling_* dependencies not yet processed
    let mut in_degree: HashMap<&str, usize> = HashMap::new();
    // reverse[dependency] = list of crates that depend on it
    let mut reverse: HashMap<&str, Vec<&str>> = HashMap::new();

    for name in all_crates {
        in_degree.entry(name.as_str()).or_insert(0);
        reverse.entry(name.as_str()).or_default();
    }

    for (crate_name, deps) in dep_map {
        for dep in deps {
            if all_crates.contains(dep.as_str()) {
                reverse
                    .get_mut(dep.as_str())
                    .unwrap()
                    .push(crate_name.as_str());
                *in_degree.get_mut(crate_name.as_str()).unwrap() += 1;
            }
        }
    }

    let mut result: Vec<String> = Vec::new();

    // Process layer by layer: all crates with in_degree == 0 in one batch form a layer
    loop {
        let mut current: Vec<&str> = all_crates
            .iter()
            .filter(|n| in_degree.get(n.as_str()).copied().unwrap_or(0) == 0)
            .filter(|n| !result.iter().any(|r| r.as_str() == n.as_str()))
            .map(|s| s.as_str())
            .collect();

        if current.is_empty() {
            break;
        }

        current.sort();
        result.extend(current.iter().map(|s| s.to_string()));

        for &node in &current {
            if let Some(dependents) = reverse.get(node) {
                for &dependent in dependents {
                    if let Some(degree) = in_degree.get_mut(dependent) {
                        *degree -= 1;
                    }
                }
            }
        }
    }

    result
}

/// Find the workspace root by looking for a Cargo.toml with `[workspace]` members.
/// Starts from `start` and walks up the directory tree.
fn find_workspace_root(start: &std::path::Path) -> Option<PathBuf> {
    let mut current = Some(std::fs::canonicalize(start).unwrap_or_else(|_| start.to_path_buf()));
    while let Some(dir) = current {
        let members = get_workspace_members(&dir);
        if !members.is_empty() {
            return Some(dir);
        }
        current = dir.parent().map(|p| p.to_path_buf());
    }
    None
}

/// Output all crate paths in dependency order
#[allow(unused)]
pub fn display_dependency_order() -> Vec<PathBuf> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    let workspace_root = match find_workspace_root(&cwd) {
        Some(root) => root,
        None => return Vec::new(),
    };

    // Read workspace members from root Cargo.toml
    let members = get_workspace_members(&workspace_root);

    // Filter to crates starting with "mingling"
    let mingling_crates: HashSet<String> = members
        .into_iter()
        .filter(|m| m.starts_with("mingling"))
        .collect();

    if mingling_crates.is_empty() {
        return Vec::new();
    }

    // Build dependency graph
    let mut dep_map: HashMap<String, Vec<String>> = HashMap::new();

    for crate_name in &mingling_crates {
        let cargo_path = workspace_root.join(crate_name).join("Cargo.toml");
        let content = match std::fs::read_to_string(&cargo_path) {
            Ok(c) => c,
            Err(_) => {
                dep_map.insert(crate_name.clone(), Vec::new());
                continue;
            }
        };
        let deps = parse_mingling_deps(&content, "mingling");
        // Only keep deps that are actually in our set
        let filtered: Vec<String> = deps
            .into_iter()
            .filter(|d| mingling_crates.contains(d.as_str()))
            .collect();
        dep_map.insert(crate_name.clone(), filtered);
    }

    let sorted = topological_sort(&mingling_crates, &dep_map);

    sorted.into_iter().map(PathBuf::from).collect()
}

fn main() {
    let order = display_dependency_order();
    if order.is_empty() {
        eprintln!("Error: could not find workspace root or mingling crates");
        std::process::exit(1);
    }
    for path in order {
        println!("{}", path.display());
    }
}
