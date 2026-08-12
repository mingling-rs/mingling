use std::collections::BTreeMap;
use std::fmt::Write;
use std::path::{Path, PathBuf};

use tools::println_cargo_style;

const SIDEBAR_HEAD: &str = "- [Welcome!](README)\n";

fn main() {
    println_cargo_style!("Refresh: _sidebar.md");
    gen_all_sidebars();
}

/// Find all README.md under docs/, treat each as a site, and generate _sidebar.md for it.
fn gen_all_sidebars() {
    let repo_root = find_git_repo().unwrap();
    let docs_root = repo_root.join("docs");

    let readme_paths = find_all_readmes(&docs_root);

    for readme_path in &readme_paths {
        let site_root = readme_path.parent().unwrap();

        let content_dir = find_content_dir(site_root);

        if let Some(content_dir) = content_dir {
            let lines = build_sidebar_content(site_root, &content_dir, SIDEBAR_HEAD);

            let sidebar_path = site_root.join("_sidebar.md");
            std::fs::write(&sidebar_path, lines).unwrap();
            println_cargo_style!("Generated: {}", sidebar_path.display());
        }
    }
}

/// Recursively find all README.md files under a directory.
fn find_all_readmes(dir: &Path) -> Vec<PathBuf> {
    let mut results = Vec::new();
    if let Ok(read_dir) = std::fs::read_dir(dir) {
        let mut entries: Vec<_> = read_dir.flatten().collect();
        entries.sort_by_key(|e| e.path());
        for entry in entries {
            let path = entry.path();
            if path.is_dir() {
                results.extend(find_all_readmes(&path));
            } else if path.file_name().is_some_and(|n| n == "README.md") {
                results.push(path);
            }
        }
    }
    results
}

/// Find the content directory for a site:
/// 1. Prefer `pages/` if it exists (backward compatible)
/// 2. Fall back to the first subdirectory that contains .md files
fn find_content_dir(site_root: &Path) -> Option<PathBuf> {
    // Try pages/ first
    let pages_dir = site_root.join("pages");
    if pages_dir.exists() && pages_dir.is_dir() {
        return Some(pages_dir);
    }

    // Fall back to any subdirectory containing .md files
    if let Ok(read_dir) = std::fs::read_dir(site_root) {
        let mut entries: Vec<_> = read_dir.flatten().collect();
        entries.sort_by_key(|e| e.path());
        for entry in entries {
            let path = entry.path();
            if path.is_dir() && has_markdown_files(&path) {
                return Some(path);
            }
        }
    }

    None
}

/// Check if a directory (recursively) contains any .md files.
fn has_markdown_files(dir: &Path) -> bool {
    if let Ok(read_dir) = std::fs::read_dir(dir) {
        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if has_markdown_files(&path) {
                    return true;
                }
            } else if path.extension().is_some_and(|ext| ext == "md") {
                return true;
            }
        }
    }
    false
}

/// Build sidebar content: scan .md files in `pages_dir` and return a formatted sidebar string
fn build_sidebar_content(base_dir: &Path, pages_dir: &Path, sidebar_head: &str) -> String {
    let mut lines = String::from(sidebar_head);

    // Collect and sort entries at root level first
    let mut root_files: Vec<SidebarEntry> = Vec::new();
    // Subdirectory name -> its files
    let mut sub_dirs: BTreeMap<String, Vec<SidebarEntry>> = BTreeMap::new();

    if let Ok(read_dir) = std::fs::read_dir(pages_dir) {
        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let dir_name = entry.file_name().to_string_lossy().to_string();
                let entries = collect_markdown_files(&path, base_dir);
                if !entries.is_empty() {
                    // Check for .name file to override directory display name
                    let display_name = get_directory_display_name(&path, &dir_name);
                    sub_dirs.insert(display_name, entries);
                }
            } else if path.extension().is_some_and(|ext| ext == "md") {
                let title = extract_title(&path);
                let relative = path
                    .strip_prefix(base_dir)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                let link = relative
                    .strip_suffix(".md")
                    .unwrap_or(&relative)
                    .to_string();
                root_files.push(SidebarEntry { title, link });
            }
        }
    }

    // Sort root files — natural order (1, 2, ..., 10, 11)
    root_files.sort_by(|a, b| natural_cmp(&a.link, &b.link));

    // Append root-level files
    for f in &root_files {
        let _ = writeln!(lines, "* [{}]({})", f.title, f.link);
    }

    // Append subdirectory groups
    for (dir_name, entries) in &sub_dirs {
        let mut sorted_entries = entries.clone();
        sorted_entries.sort_by(|a, b| natural_cmp(&a.link, &b.link));

        // Directory header with 2-space indent
        let _ = writeln!(lines, "* {dir_name}");
        for f in &sorted_entries {
            let _ = writeln!(lines, "  * [{}]({})", f.title, f.link);
        }
    }

    lines
}

#[derive(Clone)]
struct SidebarEntry {
    title: String,
    link: String,
}

/// Collect all `.md` files directly under `dir`
fn collect_markdown_files(dir: &Path, base_dir: &Path) -> Vec<SidebarEntry> {
    let mut entries = Vec::new();

    if let Ok(read_dir) = std::fs::read_dir(dir) {
        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "md") {
                let title = extract_title(&path);
                let relative = path
                    .strip_prefix(base_dir)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                let link = relative
                    .strip_suffix(".md")
                    .unwrap_or(&relative)
                    .to_string();
                entries.push(SidebarEntry { title, link });
            }
        }
    }

    entries
}

/// Extract title from the first line `<h1 align="center">TITLE</h1>`.
/// Fallback to filename stem.
fn extract_title(path: &Path) -> String {
    let content = std::fs::read_to_string(path).unwrap_or_default();
    if let Some(first_line) = content.lines().next() {
        let trimmed = first_line.trim();
        // Find `>TITLE<` between `<h1 align="center">` and `</h1>`
        if let Some(start) = trimmed.find('>') {
            let after_start = &trimmed[start + 1..];
            if let Some(end) = after_start.find('<') {
                return after_start[..end].to_string();
            }
        }
    }
    // Fallback: use file stem
    path.file_stem().map_or_else(
        || "Untitled".to_string(),
        |s| s.to_string_lossy().to_string(),
    )
}

/// Read `.name` file inside a directory to get its display name for the sidebar.
/// Falls back to the directory name itself if no `.name` file exists.
fn get_directory_display_name(dir_path: &std::path::Path, fallback: &str) -> String {
    let name_file = dir_path.join(".name");
    if name_file.exists() && name_file.is_file() {
        std::fs::read_to_string(&name_file)
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| fallback.to_string())
    } else {
        fallback.to_string()
    }
}

fn find_git_repo() -> Option<std::path::PathBuf> {
    let mut current_dir = std::env::current_dir().ok()?;

    loop {
        let git_dir = current_dir.join(".git");
        if git_dir.exists() && git_dir.is_dir() {
            return Some(current_dir);
        }

        if !current_dir.pop() {
            break;
        }
    }

    None
}

/// Natural (numeric-aware) comparison for sidebar links.
///
/// Files prefixed with a number (e.g. `1-getting-started`) are sorted by that number;
/// files without a numeric prefix fall back to lexicographic order (after numbers).
fn natural_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    let num_a = extract_leading_number(a);
    let num_b = extract_leading_number(b);
    num_a.cmp(&num_b).then_with(|| a.cmp(b))
}

/// Extract the leading numeric prefix from a sidebar link path.
///
/// Looks at the filename stem (after the last `/`) for a number before the first `-`.
/// Returns `usize::MAX` for entries without a numeric prefix.
fn extract_leading_number(link: &str) -> usize {
    if let Some(file_stem) = link.rsplit('/').next()
        && let Some(num_end) = file_stem.find('-')
        && let Ok(num) = file_stem[..num_end].parse::<usize>()
    {
        return num;
    }
    usize::MAX
}
