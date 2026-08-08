//! Package the versioned mingling project templates under
//! `mingling_cli/tmpls/mingling_tmpl/` into per-version ZIP archives
//! (e.g. `mingling-tmpl-0.4.zip`) plus SHA-256 checksum files
//! (e.g. `mingling_tmpl-0.4.sha256`), written into `templates/`.

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use tools::{dependency_order::find_workspace_root, eprintln_cargo_style, println_cargo_style};
use zip::write::SimpleFileOptions;
use zip::CompressionMethod;

fn main() {
    // 1. Determine the workspace root
    let cwd = std::env::current_dir().expect("failed to get current working directory");
    let workspace_root = find_workspace_root(&cwd).expect("not inside a Cargo workspace");
    println_cargo_style!("Workspace: {}", workspace_root.display());

    // 2. Locate the versioned template directories
    let tmpls_root = workspace_root.join("mingling_cli/tmpls/mingling_tmpl");
    if !tmpls_root.is_dir() {
        eprintln_cargo_style!(format!(
            "template root not found: {}",
            tmpls_root.display()
        ));
        std::process::exit(1);
    }

    let mut versions: Vec<String> = Vec::new();
    for entry in std::fs::read_dir(&tmpls_root).expect("failed to read template root") {
        let entry = entry.expect("failed to read template root entry");
        let name = entry.file_name().to_string_lossy().to_string();
        if entry.path().is_dir() && is_version(&name) {
            versions.push(name);
        }
    }

    if versions.is_empty() {
        eprintln_cargo_style!("no versioned template directories found");
        std::process::exit(1);
    }

    versions.sort_by(|a, b| cmp_versions(a, b));
    for v in &versions {
        println_cargo_style!("Template: {} ({})", v, tmpls_root.join(v).display());
    }

    // 3. Prepare the output directory
    let out_dir = workspace_root.join("templates");
    println_cargo_style!("Clean: templates/");
    let _ = std::fs::remove_dir_all(&out_dir);
    std::fs::create_dir_all(&out_dir).expect("failed to create templates/");

    // 4. Package each version: ZIP + SHA-256
    let mut failed = false;
    for version in &versions {
        let src_dir = tmpls_root.join(version);
        let zip_name = format!("mingling-tmpl-{version}.zip");
        let sha_name = format!("mingling_tmpl-{version}.sha256");
        let zip_path = out_dir.join(&zip_name);
        let sha_path = out_dir.join(&sha_name);

        println_cargo_style!("Package: {}", zip_name);
        if let Err(e) = package_template(&src_dir, &zip_path) {
            eprintln_cargo_style!(format!("failed to package {}: {e}", zip_name));
            failed = true;
            continue;
        }

        let digest = sha256_of_file(&zip_path).unwrap_or_else(|e| {
            eprintln_cargo_style!(format!("failed to hash {}: {e}", zip_path.display()));
            std::process::exit(1);
        });

        // sha256sum-compatible line: "<hex>  <zip file name>"
        let checksum_line = format!("{digest}  {zip_name}\n");
        std::fs::write(&sha_path, checksum_line)
            .unwrap_or_else(|e| panic!("failed to write {}: {e}", sha_path.display()));
        println_cargo_style!("Checksum: {}  {}", digest, sha_name);
    }

    if failed {
        eprintln_cargo_style!("one or more templates failed to package");
        std::process::exit(1);
    }

    println_cargo_style!("Done: templates/ is ready");
}

/// Version directory names look like `0.4`, `0.3`, `1.2.3`, ...
fn is_version(name: &str) -> bool {
    !name.is_empty()
        && name
            .split('.')
            .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()))
}

/// Compare two version strings numerically, component by component.
fn cmp_versions(a: &str, b: &str) -> std::cmp::Ordering {
    let a_parts: Vec<u64> = a.split('.').filter_map(|p| p.parse().ok()).collect();
    let b_parts: Vec<u64> = b.split('.').filter_map(|p| p.parse().ok()).collect();
    for (x, y) in a_parts.iter().zip(b_parts.iter()) {
        match x.cmp(y) {
            std::cmp::Ordering::Equal => continue,
            ord => return ord,
        }
    }
    a_parts.len().cmp(&b_parts.len())
}

/// Recursively ZIP the *contents* of `src_dir` (files at the archive root)
/// into `dest_zip`, preserving the relative directory structure.
fn package_template(src_dir: &Path, dest_zip: &Path) -> Result<(), zip::result::ZipError> {
    let file = File::create(dest_zip).map_err(zip::result::ZipError::Io)?;
    let mut writer = zip::ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    // Collect entries first so the archive is written in a deterministic order.
    let mut entries: Vec<PathBuf> = Vec::new();
    collect_entries(src_dir, src_dir, &mut entries).map_err(zip::result::ZipError::Io)?;
    entries.sort();

    for rel_path in &entries {
        let full_path = src_dir.join(rel_path);
        // ZIP archives always use `/` as the path separator.
        let name = rel_path.to_string_lossy().replace('\\', "/");
        if full_path.is_dir() {
            writer.add_directory(name, options)?;
        } else {
            writer.start_file(name, options)?;
            let mut file = File::open(&full_path).map_err(zip::result::ZipError::Io)?;
            std::io::copy(&mut file, &mut writer).map_err(zip::result::ZipError::Io)?;
        }
    }

    writer.finish().map(|_| ())
}

/// Recursively collect all relative paths (files and directories) under `dir`.
fn collect_entries(root: &Path, dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    let mut children: Vec<PathBuf> = std::fs::read_dir(dir)?
        .map(|e| e.map(|e| e.path()))
        .collect::<std::io::Result<Vec<_>>>()?;
    children.sort();

    for child in children {
        let rel = child
            .strip_prefix(root)
            .expect("collected entry is inside root")
            .to_path_buf();
        if child.is_dir() {
            out.push(rel);
            collect_entries(root, &child, out)?;
        } else {
            out.push(rel);
        }
    }
    Ok(())
}

/// Compute the lowercase hex SHA-256 digest of a file.
fn sha256_of_file(path: &Path) -> std::io::Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    let digest = hasher.finalize();
    Ok(digest.iter().map(|b| format!("{b:02x}")).collect())
}
