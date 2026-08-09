use std::{
    collections::HashMap,
    fs::{self, File},
    io,
    path::{Path, PathBuf},
};

use just_template::Template;
use mingling::{
    Grouped, RenderResult, Routable,
    macros::{arg, chain, command, metadata, pack, pack_err, r_println, renderer, routeify},
    metadata::Description,
    picker::{EntryPicker, value::FilePath},
    res::ResCurrentDir,
};

use crate::{Entry, Next, eprintln_cargo, hprintln_cargo, println_cargo};

use super::rule_solver::{eval_rule, parse_checklist, parse_rules, resolve_answers};

/// The name of the checklist file the user edits and we re-read on generate.
const CHECKLIST_FILENAME: &str = "checklist.toml";
/// The name of the rules file declaring default params, display blocks and hides.
const RULE_FILENAME: &str = "rule.toml";
/// The directory under the project root where the template cache lives.
const CACHE_DIR_NAME: &str = "tmpl-cache";
/// The internal `.mling` directory name inside the template archive (with `tmpl_` prefix).
const TEMPLATE_MLING_DIR: &str = "tmpl_.mling";
/// Prefix marking directories/files that participate in expansion.
const TMPL_PREFIX: &str = "tmpl_";

pack!(StateProjectGenerate = ());
pack!(StateProjectChecklistReady = Vec<String>);

/// Result of the checklist phase: the extracted checklist handed to the user.
#[derive(Debug, Default, Grouped)]
pub struct ResultProjectChecklistReady {
    pub checklist: PathBuf,
}

/// Result of the generate phase: files produced (and hidden) by expansion.
#[derive(Debug, Default, Grouped)]
pub struct ResultProjectGenerate {
    pub generated: Vec<PathBuf>,
    pub hidden: Vec<PathBuf>,
}

pack_err!(ErrorTemplateZipNotProvided = ());
pack_err!(ErrorUnzipFailed = String);
pack_err!(ErrorChecklistMissing = String);
pack_err!(ErrorRuleParseFailed = String);
pack_err!(ErrorTemplateExpandFailed = String);

#[command(node = "proj-init", routeify)]
pub fn proj_init(args: Entry, cwd: &ResCurrentDir) -> Next {
    // Check if the checklist.toml file exists in the current directory
    if cwd.join(CHECKLIST_FILENAME).exists() {
        StateProjectGenerate::new(()).into()
    } else {
        StateProjectChecklistReady::new(args.inner).into()
    }
}

/// Phase 1: extract the user-provided template archive into
/// `./.mling/tmpl-cache/` and hand the checklist over for editing.
#[chain(routeify)]
pub fn handle_state_proj_checklist_ready(
    args: StateProjectChecklistReady,
    cwd: &ResCurrentDir,
) -> Next {
    let tmpl_file: Option<FilePath> = args.pick(&arg![Option<FilePath>]).to_result()?;

    // Validate
    let Some(tmpl_file) = tmpl_file else {
        return ErrorTemplateZipNotProvided::new(()).to_chain();
    };

    // Extract the file into the .mling directory under the current directory; create it if it doesn't exist
    let tmpl_cache = cwd.join(".mling").join(CACHE_DIR_NAME);
    fs::create_dir_all(&tmpl_cache).map_err(|e| {
        ErrorUnzipFailed::new(format!("failed to create {}: {e}", tmpl_cache.display()))
    })?;
    unzip_to(&tmpl_file, &tmpl_cache).map_err(ErrorUnzipFailed::new)?;

    // Move the internal checklist.toml to ./ for the user to fill in
    let checklist_src = tmpl_cache.join(CHECKLIST_FILENAME);
    if !checklist_src.is_file() {
        return ErrorChecklistMissing::new(format!(
            "no checklist.toml found inside {}",
            tmpl_file.display()
        ))
        .to_chain();
    }
    let checklist_dst = cwd.join(CHECKLIST_FILENAME);
    fs::rename(&checklist_src, &checklist_dst).map_err(|e| {
        ErrorUnzipFailed::new(format!(
            "failed to move checklist.toml to {}: {e}",
            checklist_dst.display()
        ))
    })?;

    ResultProjectChecklistReady {
        checklist: checklist_dst,
    }
    .to_chain()
}

/// Phase 2: expand the cached template with the checklist answers, driven by
/// `rule.toml` display/hide rules, then clean up the cache.
#[chain(routeify)]
pub fn handle_state_project_generate(_: StateProjectGenerate, cwd: &ResCurrentDir) -> Next {
    let tmpl_cache = cwd.join(".mling").join(CACHE_DIR_NAME);
    if !tmpl_cache.is_dir() {
        return ErrorChecklistMissing::new(format!(
            "template cache not found at {}; run `mling proj-init` with a template archive first",
            tmpl_cache.display()
        ))
        .to_chain();
    }

    // Read the user-filled checklist.toml
    let checklist_path = cwd.join(CHECKLIST_FILENAME);
    let checklist_content = fs::read_to_string(&checklist_path).map_err(|e| {
        ErrorChecklistMissing::new(format!("failed to read {}: {e}", checklist_path.display()))
    })?;
    let answers = parse_checklist(&checklist_content)
        .map_err(|e| ErrorRuleParseFailed::new(format!("invalid checklist.toml: {e}")))?;

    // Read rule.toml (template rules)
    let rule_content = fs::read_to_string(tmpl_cache.join(RULE_FILENAME))
        .map_err(|e| ErrorRuleParseFailed::new(format!("failed to read rule.toml: {e}")))?;
    let rules = parse_rules(&rule_content)
        .map_err(|e| ErrorRuleParseFailed::new(format!("invalid rule.toml: {e}")))?;

    // Compute final answers from checklist values + defaults declared in rule.toml
    let answers = resolve_answers(&answers, &rules);

    // Compute display block toggles based on rules: checklist values + rules whose display condition is true

    let mut params: HashMap<String, String> = answers.clone();
    for display in &rules.display {
        if eval_rule(&display.rule, &answers) {
            params.insert(display.name.clone(), String::new());
        }
    }

    // Expand all tmpl_* entries to the project root
    let mut generated = Vec::new();
    expand_tree(&tmpl_cache, cwd, &params, &mut generated)
        .map_err(ErrorTemplateExpandFailed::new)?;

    // hide-file: delete the corresponding generated file when the rule is true

    let mut hidden = Vec::new();
    for hide in &rules.hide_files {
        if !eval_rule(&hide.rule, &answers) {
            continue;
        }
        let target = cwd.join(hide.file.trim_start_matches("./"));
        remove_path(&target).map_err(|e| {
            ErrorTemplateExpandFailed::new(format!("failed to hide {}: {e}", target.display()))
        })?;
        hidden.push(target);
    }

    // Clean up the cache
    fs::remove_dir_all(&tmpl_cache).map_err(|e| {
        ErrorTemplateExpandFailed::new(format!("failed to remove {}: {e}", tmpl_cache.display()))
    })?;

    // Project generated; remove the temporary checklist file
    remove_path(&checklist_path).map_err(|e| {
        ErrorTemplateExpandFailed::new(format!(
            "failed to remove {}: {e}",
            checklist_path.display()
        ))
    })?;

    ResultProjectGenerate { generated, hidden }.to_chain()
}

/// Extract a ZIP archive into `dest`, guarding against path traversal.
fn unzip_to(zip_path: &Path, dest: &Path) -> Result<(), String> {
    let file = File::open(zip_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let Some(rel) = entry.enclosed_name() else {
            return Err(format!("illegal path in archive: {}", entry.name()));
        };
        let out_path = dest.join(rel);
        if entry.is_dir() {
            fs::create_dir_all(&out_path).map_err(|e| e.to_string())?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let mut out = File::create(&out_path).map_err(|e| e.to_string())?;
            io::copy(&mut entry, &mut out).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

/// Recursively expand the template cache into the project root.
///
/// Every path component with a `tmpl_` prefix is stripped for the target path
/// (`tmpl_src/tmpl_main.rs` -> `src/main.rs`). `tmpl_`-prefixed files are
/// rendered through `just_template` with the resolved params; other files are
/// skipped. `tmpl_.mling` is copied into `./.mling` without rendering.
fn expand_tree(
    src_root: &Path,
    dst_root: &Path,
    params: &HashMap<String, String>,
    generated: &mut Vec<PathBuf>,
) -> Result<(), String> {
    for entry in fs::read_dir(src_root).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let src = entry.path();

        // The template's own .mling tree is copied verbatim (minus `tmpl_` prefixes).
        if name == TEMPLATE_MLING_DIR && src.is_dir() {
            copy_tree(&src, &dst_root.join(".mling")).map_err(|e| e.to_string())?;
            continue;
        }

        let target_name = name
            .strip_prefix(TMPL_PREFIX)
            .map(str::to_owned)
            .unwrap_or(name.clone());
        let dst = dst_root.join(&target_name);

        if src.is_dir() {
            fs::create_dir_all(&dst).map_err(|e| e.to_string())?;
            expand_tree(&src, &dst, params, generated)?;
        } else if src.is_file() && name.starts_with(TMPL_PREFIX) {
            let content = fs::read_to_string(&src).map_err(|e| e.to_string())?;
            let mut tmpl = Template::from(content);
            for (key, value) in params {
                tmpl.insert_param(key.clone(), value.clone());
            }
            let expanded = tmpl
                .expand()
                .ok_or_else(|| format!("failed to expand template: {}", src.display()))?;
            if let Some(parent) = dst.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            fs::write(&dst, expanded).map_err(|e| e.to_string())?;
            generated.push(dst);
        }
    }
    Ok(())
}

/// Copy a directory tree into `dst`, stripping the `tmpl_` prefix from every
/// component name (e.g. `tmpl_command` -> `command`).
fn copy_tree(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let target_name = name
            .strip_prefix(TMPL_PREFIX)
            .map(str::to_owned)
            .unwrap_or(name);
        let from = entry.path();
        let to = dst.join(&target_name);
        if from.is_dir() {
            copy_tree(&from, &to)?;
        } else if from.is_file() {
            fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

/// Remove a file or directory, ignoring "not found".
fn remove_path(path: &Path) -> io::Result<()> {
    if path.is_dir() {
        fs::remove_dir_all(path)
    } else if path.is_file() {
        fs::remove_file(path)
    } else {
        Ok(())
    }
}

#[renderer]
pub fn render_result_project_checklist_ready(result: ResultProjectChecklistReady) -> RenderResult {
    let mut r = RenderResult::new();
    r_println!(r, "Template extracted.");
    r_println!(r, "");
    hprintln_cargo!(
        r,
        "Fill out {} and run `mling proj-init` again to generate the project.",
        result.checklist.display()
    );
    r
}

#[renderer]
pub fn render_result_project_generate(result: ResultProjectGenerate) -> RenderResult {
    let mut r = RenderResult::new();
    for file in &result.generated {
        println_cargo!(r, "Generated: {}", file.display());
    }
    for file in &result.hidden {
        println_cargo!(r, "Hidden: {}", file.display());
    }
    r
}

#[renderer]
pub fn render_error_template_zip_not_provided(_err: ErrorTemplateZipNotProvided) -> RenderResult {
    let mut r = RenderResult::new();
    eprintln_cargo!(
        r,
        "no template archive provided; pass the path to a mingling template zip file"
    );
    r
}

#[renderer]
pub fn render_error_unzip_failed(err: ErrorUnzipFailed) -> RenderResult {
    let mut r = RenderResult::new();
    eprintln_cargo!(r, "failed to extract template: {}", err.info);
    r
}

#[renderer]
pub fn render_error_checklist_missing(err: ErrorChecklistMissing) -> RenderResult {
    let mut r = RenderResult::new();
    eprintln_cargo!(r, "{}", err.info);
    r
}

#[renderer]
pub fn render_error_rule_parse_failed(err: ErrorRuleParseFailed) -> RenderResult {
    let mut r = RenderResult::new();
    eprintln_cargo!(r, "{}", err.info);
    r
}

#[renderer]
pub fn render_error_template_expand_failed(err: ErrorTemplateExpandFailed) -> RenderResult {
    let mut r = RenderResult::new();
    eprintln_cargo!(r, "{}", err.info);
    r
}

#[metadata(EntryProjInit)]
pub fn desc_proj_init() -> Description {
    "Guided creation of a Mingling project".into()
}
