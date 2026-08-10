use std::{
    fs,
    path::{Path, PathBuf},
};

use just_fmt::{camel_case, kebab_case, pascal_case, snake_case};
use just_template::Template;
use mingling::{
    Grouped, RenderResult, Routable, ShellContext, Suggest, SuggestItem,
    macros::{arg, chain, command, completion, metadata, pack, pack_err, renderer, routeify},
    metadata::Description,
    picker::EntryPicker,
    res::ResCurrentDir,
};
use toml_edit::DocumentMut;

use crate::{Next, eprintln_cargo, println_cargo};

/// A `[[classes]]` entry in `.mling/classes.toml`.
#[derive(Debug, Clone)]
pub struct ClassEntry {
    /// Class type name, e.g. `subcommand`.
    pub name: String,
    /// Template path relative to `.mling/`, e.g. `classes/subcommand.rs`.
    pub template: String,
    /// Output directory relative to the project root, e.g. `src/command/`.
    pub output_dir: String,
    /// Short description shown in completions, may be empty.
    pub description: String,
}

pack!(StateClassAdd = (String, String));

/// Result of adding a class: the generated file path.
#[derive(Debug, Default, Grouped)]
pub struct ResultClassAdd {
    pub output: PathBuf,
}

pack_err!(ErrorClassNameRequired = ());
pack_err!(ErrorClassConfigMissing = String);
pack_err!(ErrorClassNotFound = String);
pack_err!(ErrorClassTemplateMissing = String);
pack_err!(ErrorClassWriteFailed = String);

#[command(node = "class-add", routeify)]
pub fn class_add(args: EntryClassAdd) -> Next {
    let (class_name, name) = args
        .pick_or_route(&arg![String], || ErrorClassNameRequired::new(()).to_chain())
        .pick_or_route(&arg![String], || ErrorClassNameRequired::new(()).to_chain())
        .to_result()?;
    StateClassAdd::new((class_name, name)).to_chain()
}

/// Walk upward from `start` to find the first directory containing `.mling`.
fn find_project_root(start: &Path) -> Option<PathBuf> {
    let mut dir = deverbatim(&fs::canonicalize(start).ok()?);
    loop {
        if dir.join(".mling").is_dir() {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}

/// Deverbatim a Windows path: strip the `\\?\` prefix that `fs::canonicalize`
/// may add. On non-Windows platforms this is a no-op.
///
/// On Windows, canonical paths sometimes carry a verbatim prefix like
/// `\\?\C:\...`. This function removes that prefix. For UNC paths such as
/// `\\?\UNC\server\share`, the prefix is converted back to the conventional
/// `\\server\share` form.
fn deverbatim(path: &Path) -> PathBuf {
    if !cfg!(windows) {
        return path.to_path_buf();
    }
    let as_string = path.to_string_lossy();
    if let Some(rest) = as_string.strip_prefix(r"\\?\") {
        // Turns `\\?\UNC\server\share` back into `\\server\share`.
        if let Some(share) = rest.strip_prefix("UNC\\") {
            return PathBuf::from(format!(r"\\{share}"));
        }
        return PathBuf::from(rest.to_owned());
    }
    path.to_path_buf()
}

/// Read `.mling/classes.toml`, find the class template and render it with the
/// name-derived parameters into `<output-dir>/<snake_case>.rs`.
#[chain(routeify)]
pub fn handle_state_class_add(state: StateClassAdd, cwd: &ResCurrentDir) -> Next {
    let (class_name, name) = state.inner;

    // Resolve the project root: the nearest ancestor directory with `.mling`.
    let Some(project_root) = find_project_root(cwd) else {
        return ErrorClassConfigMissing::new(format!(
            "no `.mling` directory found from {} upward; run this inside a mingling project",
            cwd.display()
        ))
        .to_chain();
    };

    // Read `.mling/classes.toml` (the class registry).
    let classes_path = project_root.join(".mling").join("classes.toml");
    let content = fs::read_to_string(&classes_path).map_err(|e| {
        ErrorClassConfigMissing::new(format!("failed to read {}: {e}", classes_path.display()))
    })?;
    let classes = parse_classes(&content)
        .map_err(|e| ErrorClassConfigMissing::new(format!("invalid classes.toml: {e}")))?;

    // Find the requested class type.
    let Some(entry) = classes.iter().find(|c| c.name == class_name) else {
        return ErrorClassNotFound::new(format!(
            "class `{class_name}` not found in {}",
            classes_path.display()
        ))
        .to_chain();
    };

    // Read the class template (relative to `.mling/`).
    let template_path = project_root.join(".mling").join(&entry.template);
    let template_content = fs::read_to_string(&template_path).map_err(|e| {
        ErrorClassTemplateMissing::new(format!("failed to read {}: {e}", template_path.display()))
    })?;

    // Derive the name variants used by the template placeholders.
    let snake = snake_case!(name.as_str());
    let pascal = pascal_case!(name.as_str());
    let kebab = kebab_case!(name.as_str());
    let upper_snake = snake.to_uppercase();
    let camel = camel_case!(name.as_str());

    let mut tmpl = Template::from(template_content);
    tmpl.insert_param("snake_case".to_string(), snake.clone());
    tmpl.insert_param("pascal_case".to_string(), pascal);
    tmpl.insert_param("kebab_case".to_string(), kebab);
    tmpl.insert_param("upper_snake_case".to_string(), upper_snake);
    tmpl.insert_param("camel_case".to_string(), camel);
    let expanded = tmpl.expand().ok_or_else(|| {
        ErrorClassWriteFailed::new(format!(
            "failed to expand class template: {}",
            template_path.display()
        ))
    })?;

    // Write to `<output-dir>/<snake_case>.rs`.
    let output_dir = project_root.join(&entry.output_dir);
    let output = output_dir.join(format!("{snake}.rs"));
    fs::create_dir_all(&output_dir).map_err(|e| {
        ErrorClassWriteFailed::new(format!("failed to create {}: {e}", output_dir.display()))
    })?;
    fs::write(&output, expanded).map_err(|e| {
        ErrorClassWriteFailed::new(format!("failed to write {}: {e}", output.display()))
    })?;

    ResultClassAdd { output }.to_chain()
}

/// Parse `.mling/classes.toml` into class entries.
fn parse_classes(content: &str) -> Result<Vec<ClassEntry>, String> {
    let doc = content.parse::<DocumentMut>().map_err(|e| e.to_string())?;
    let mut entries = Vec::new();
    if let Some(tables) = doc
        .get("classes")
        .and_then(|item| item.as_array_of_tables())
    {
        for table in tables {
            let name = table
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or("[[classes]] entry is missing `name`")?
                .to_string();
            let template = table
                .get("template")
                .and_then(|v| v.as_str())
                .ok_or("[[classes]] entry is missing `template`")?
                .to_string();
            let output_dir = table
                .get("output-dir")
                .and_then(|v| v.as_str())
                .ok_or("[[classes]] entry is missing `output-dir`")?
                .to_string();
            let description = table
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            entries.push(ClassEntry {
                name,
                template,
                output_dir,
                description,
            });
        }
    }
    Ok(entries)
}

#[renderer]
pub fn render_result_class_add(result: ResultClassAdd) -> RenderResult {
    let mut r = RenderResult::new();
    println_cargo!(r, "Class added: {}", result.output.display());
    r
}

#[renderer]
pub fn render_error_class_name_required(_err: ErrorClassNameRequired) -> RenderResult {
    let mut r = RenderResult::new();
    eprintln_cargo!(r, "usage: mling class-add <class> <name>");
    r
}

#[renderer]
pub fn render_error_class_config_missing(err: ErrorClassConfigMissing) -> RenderResult {
    let mut r = RenderResult::new();
    eprintln_cargo!(r, "{}", err.info);
    r
}

#[renderer]
pub fn render_error_class_not_found(err: ErrorClassNotFound) -> RenderResult {
    let mut r = RenderResult::new();
    eprintln_cargo!(r, "{}", err.info);
    r
}

#[renderer]
pub fn render_error_class_template_missing(err: ErrorClassTemplateMissing) -> RenderResult {
    let mut r = RenderResult::new();
    eprintln_cargo!(r, "{}", err.info);
    r
}

#[renderer]
pub fn render_error_class_write_failed(err: ErrorClassWriteFailed) -> RenderResult {
    let mut r = RenderResult::new();
    eprintln_cargo!(r, "{}", err.info);
    r
}

#[completion(EntryClassAdd)]
pub fn complete_class_add(ctx: &ShellContext, cwd: &ResCurrentDir) -> Suggest {
    if ctx.previous_word != "class-add" {
        return Suggest::file_comp();
    }
    let mut suggest = Suggest::new();
    let Some(project_root) = find_project_root(cwd) else {
        return suggest;
    };
    let classes_path = project_root.join(".mling").join("classes.toml");
    let Ok(content) = fs::read_to_string(&classes_path) else {
        return suggest;
    };
    let Ok(classes) = parse_classes(&content) else {
        return suggest;
    };
    for entry in classes {
        if entry.description.is_empty() {
            suggest.insert(SuggestItem::new(entry.name));
        } else {
            suggest.insert(SuggestItem::new_with_desc(entry.name, entry.description));
        }
    }
    suggest
}

#[metadata(EntryClassAdd)]
pub fn desc_class_add() -> Description {
    "Add a class instance to the project from a registered template".into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_project_root_walks_upward() {
        let tmp = std::env::temp_dir().join(format!("mling-class-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(tmp.join("project/src/deep/nested")).unwrap();
        fs::create_dir_all(tmp.join("project/.mling")).unwrap();

        // Found at the project root.
        // Normalize the expected path the way `find_project_root` does
        // (canonicalize + strip the Windows verbatim prefix) so the
        // comparison is immune to Windows 8.3 short-name / long-name
        // differences and the `\\?\` prefix.
        let root = deverbatim(&fs::canonicalize(tmp.join("project")).unwrap());
        assert_eq!(find_project_root(&root), Some(root.clone()));

        // Found by walking up from a deep subdirectory.
        assert_eq!(find_project_root(&root.join("src/deep/nested")), Some(root));

        // No `.mling` anywhere above.
        assert_eq!(
            find_project_root(&tmp.join("project/src").join("..").join("..")),
            None
        );

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn parses_class_entries() {
        let content = r#"
[[classes]]
name = "subcommand"
template = "classes/subcommand.rs"
output-dir = "src/command/"
description = "Add a subcommand"

[[classes]]
name = "resource"
template = "classes/resource.rs"
output-dir = "src/resource/"
"#;
        let entries = parse_classes(content).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].name, "subcommand");
        assert_eq!(entries[0].template, "classes/subcommand.rs");
        assert_eq!(entries[0].output_dir, "src/command/");
        assert_eq!(entries[0].description, "Add a subcommand");
        assert_eq!(entries[1].name, "resource");
        // Description is optional and defaults to empty.
        assert_eq!(entries[1].description, "");
    }

    #[test]
    fn parses_empty_classes() {
        let entries = parse_classes("").unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn missing_fields_are_rejected() {
        let content = r#"
[[classes]]
name = "subcommand"
"#;
        assert!(parse_classes(content).is_err());
    }
}
