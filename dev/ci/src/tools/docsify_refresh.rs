//! Docsify maintenance: fix code-box blank lines and regenerate `_sidebar.md`
//! files under `docs/`.

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use mingling::{
    Grouped, RenderResult, Routable,
    macros::{buffer, command, r_println, renderer},
};

use crate::Next;
use crate::res::{CargoError, MessagePrinter};

const DOCS_DIR: &str = "./docs";
const SIDEBAR_HEAD: &str = "- [Welcome!](README)\n";

#[command(node = "docsify-refresh")]
pub fn docsify_refresh() -> Next {
    match refresh_all() {
        Ok(written) => ResultDocsifyRefresh { written }.to_chain(),
        Err(e) => ErrorDocsifyRefresh(e).to_chain(),
    }
}

fn refresh_all() -> Result<Vec<String>, String> {
    let mut written = Vec::new();
    written.extend(fix_code_boxes());
    written.extend(gen_sidebars()?);
    Ok(written)
}

/// Part 1: docsify renders code blocks poorly when the blank lines around
/// them are completely empty — replace them with a single space.
fn fix_code_boxes() -> Vec<String> {
    let mut file_count = 0;
    let mut fixed_count = 0;
    let mut written = Vec::new();

    collect_md_files(Path::new(DOCS_DIR), &mut |path| {
        if path
            .file_name()
            .is_some_and(|n| n.to_string_lossy().to_lowercase() == "_sidebar.md")
        {
            return;
        }
        let content = fs::read_to_string(path).unwrap_or_default();
        if content.is_empty() {
            return;
        }
        let new_content = fix_code_box_empty_lines(&content);
        if new_content != content {
            fs::write(path, &new_content).unwrap();
            written.push(format!("fixed: {}", path.display()));
            fixed_count += 1;
        }
        file_count += 1;
    });

    written.push(format!("scanned {file_count} files, fixed {fixed_count}"));
    written
}

/// Replaces completely empty lines adjacent to fenced code blocks with lines
/// containing a single space.
fn fix_code_box_empty_lines(content: &str) -> String {
    let mut result = String::new();
    let lines: Vec<&str> = content.lines().collect();
    let len = lines.len();

    let mut i = 0;
    while i < len {
        let line = lines[i];
        result.push_str(line);
        result.push('\n');
        i += 1;

        if !line.trim_start().starts_with("```") {
            continue;
        }

        // In a code block: find the closing fence.
        let code_start = i;
        let mut code_end = len;
        let mut found_end = false;
        while i < len {
            let cline = lines[i];
            if cline.trim_start().starts_with("```") && !cline.trim().is_empty() {
                code_end = i;
                found_end = true;
                break;
            }
            i += 1;
        }

        ensure_space_before_code_block(&mut result);

        for code_line in lines.iter().take(code_end).skip(code_start) {
            if code_line.is_empty() {
                result.push(' ');
            } else {
                result.push_str(code_line);
            }
            result.push('\n');
        }

        if found_end {
            result.push_str(lines[code_end]);
            result.push('\n');
            i += 1;

            if i < len && lines[i].trim().is_empty() && lines[i].is_empty() {
                result.push(' ');
                result.push('\n');
                i += 1;
            }
        }
    }

    while result.ends_with('\n') {
        result.pop();
    }
    result.push('\n');
    result
}

/// Turns a trailing `\n\n` before a code block into `\n \n`.
fn ensure_space_before_code_block(result: &mut String) {
    let len = result.len();
    if len >= 2 && &result[len - 2..] == "\n\n" {
        result.insert(len - 1, ' ');
    }
}

/// Part 2: find every README.md under `docs/` (each is a site root) and
/// regenerate its `_sidebar.md`.
fn gen_sidebars() -> Result<Vec<String>, String> {
    let mut written = Vec::new();
    for readme_path in find_all_readmes(Path::new(DOCS_DIR)) {
        let site_root = readme_path
            .parent()
            .ok_or_else(|| format!("{} has no parent", readme_path.display()))?;
        if let Some(content_dir) = find_content_dir(site_root) {
            let lines = build_sidebar_content(site_root, &content_dir, SIDEBAR_HEAD);
            let sidebar_path = site_root.join("_sidebar.md");
            fs::write(&sidebar_path, lines)
                .map_err(|e| format!("failed to write {}: {e}", sidebar_path.display()))?;
            written.push(format!("generated: {}", sidebar_path.display()));
        }
    }
    Ok(written)
}

/// Recursively finds all README.md files under a directory.
fn find_all_readmes(dir: &Path) -> Vec<PathBuf> {
    let mut results = Vec::new();
    if let Ok(read_dir) = fs::read_dir(dir) {
        let mut entries: Vec<_> = read_dir.flatten().collect();
        entries.sort_by_key(std::fs::DirEntry::path);
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

/// The content directory of a site: `pages/` if present, else the first
/// subdirectory containing markdown files.
fn find_content_dir(site_root: &Path) -> Option<PathBuf> {
    let pages_dir = site_root.join("pages");
    if pages_dir.is_dir() {
        return Some(pages_dir);
    }
    if let Ok(read_dir) = fs::read_dir(site_root) {
        let mut entries: Vec<_> = read_dir.flatten().collect();
        entries.sort_by_key(std::fs::DirEntry::path);
        for entry in entries {
            let path = entry.path();
            if path.is_dir() && has_markdown_files(&path) {
                return Some(path);
            }
        }
    }
    None
}

fn has_markdown_files(dir: &Path) -> bool {
    if let Ok(read_dir) = fs::read_dir(dir) {
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

struct SidebarEntry {
    title: String,
    link: String,
}

enum SidebarNode {
    File(SidebarEntry),
    Dir {
        name: String,
        display_name: String,
        path: PathBuf,
    },
}

impl SidebarNode {
    /// Sort key used to interleave files and directories by their real
    /// file/directory name (not by display name or item type).
    fn sort_key(&self) -> &str {
        match self {
            Self::File(entry) => entry.link.rsplit('/').next().unwrap_or(&entry.link),
            Self::Dir { name, .. } => name,
        }
    }
}

/// Builds the sidebar content from the markdown files under `pages_dir`.
fn build_sidebar_content(base_dir: &Path, pages_dir: &Path, sidebar_head: &str) -> String {
    let mut lines = String::from(sidebar_head);
    append_dir_entries(&mut lines, pages_dir, base_dir, 0);
    lines
}

/// Recursively appends markdown files and subdirectories under `dir` to the
/// sidebar, using deeper indentation for each nesting level. Files and
/// directories are interleaved and sorted together by their real names.
fn append_dir_entries(lines: &mut String, dir: &Path, base_dir: &Path, depth: usize) {
    let mut nodes: Vec<SidebarNode> = Vec::new();

    if let Ok(read_dir) = fs::read_dir(dir) {
        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let dir_name = entry.file_name().to_string_lossy().into_owned();
                let display_name = get_directory_display_name(&path, &dir_name);
                nodes.push(SidebarNode::Dir {
                    name: dir_name,
                    display_name,
                    path,
                });
            } else if path.extension().is_some_and(|ext| ext == "md") {
                nodes.push(SidebarNode::File(SidebarEntry {
                    title: extract_title(&path),
                    link: relative_link(&path, base_dir),
                }));
            }
        }
    }

    nodes.sort_by(|a, b| natural_cmp(a.sort_key(), b.sort_key()));
    let indent = "  ".repeat(depth);
    for node in &nodes {
        match node {
            SidebarNode::File(entry) => {
                let _ = writeln!(lines, "{indent}* [{}]({})", entry.title, entry.link);
            }
            SidebarNode::Dir {
                display_name, path, ..
            } => {
                if !has_markdown_files(path) {
                    continue;
                }
                let _ = writeln!(lines, "{indent}* {display_name}");
                append_dir_entries(lines, path, base_dir, depth + 1);
            }
        }
    }
}

/// The link of a file relative to `base_dir`, without the `.md` suffix.
fn relative_link(path: &Path, base_dir: &Path) -> String {
    path.strip_prefix(base_dir)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
        .strip_suffix(".md")
        .unwrap_or_default()
        .to_string()
}

/// Extracts the title from the first line `<h1 align="center">TITLE</h1>`,
/// falling back to the file stem.
fn extract_title(path: &Path) -> String {
    let content = fs::read_to_string(path).unwrap_or_default();
    if let Some(first_line) = content.lines().next() {
        let trimmed = first_line.trim();
        if let Some(start) = trimmed.find('>') {
            let after_start = &trimmed[start + 1..];
            if let Some(end) = after_start.find('<') {
                return after_start[..end].to_string();
            }
        }
    }
    path.file_stem().map_or_else(
        || "Untitled".to_string(),
        |s| s.to_string_lossy().into_owned(),
    )
}

/// Reads a directory's `.name` file to override its sidebar display name.
fn get_directory_display_name(dir_path: &Path, fallback: &str) -> String {
    let name_file = dir_path.join(".name");
    if name_file.is_file() {
        fs::read_to_string(&name_file)
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| fallback.to_string())
    } else {
        fallback.to_string()
    }
}

/// Numeric-aware comparison: `1-x` sorts before `10-x`, unnumbered last.
fn natural_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    extract_leading_number(a)
        .cmp(&extract_leading_number(b))
        .then_with(|| a.cmp(b))
}

/// The leading numeric prefix of a link's file stem, `usize::MAX` if absent.
fn extract_leading_number(link: &str) -> usize {
    if let Some(file_stem) = link.rsplit('/').next()
        && let Some(num_end) = file_stem.find('-')
        && let Ok(num) = file_stem[..num_end].parse::<usize>()
    {
        return num;
    }
    usize::MAX
}

/// Recursively collects all `.md` files under a directory.
fn collect_md_files(dir: &Path, callback: &mut dyn FnMut(&Path)) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_md_files(&path, callback);
            } else if path.extension().is_some_and(|ext| ext == "md") {
                callback(&path);
            }
        }
    }
}

/// Files written by `docsify-refresh`.
#[derive(Grouped)]
pub struct ResultDocsifyRefresh {
    pub written: Vec<String>,
}

#[derive(Grouped, Default)]
pub struct ErrorDocsifyRefresh(pub String);

#[renderer(buffer)]
pub fn render_docsify_refresh(r: ResultDocsifyRefresh) {
    for item in r.written {
        r_println!("{item}");
    }
}

#[renderer]
pub fn render_error_docsify_refresh(e: ErrorDocsifyRefresh, error: &CargoError) -> RenderResult {
    let render_result = RenderResult::new();
    error.println(vec![e.0]);
    render_result
}
