//! Aggregated incremental cache for lint results.
//!
//! Everything lives in a **single** binary file at
//! `<target-dir>/mingling/mlint/cache.bin`:
//!
//! - [`CacheData::roots_signature`] — hash of the crate roots that were linted,
//!   so a change to the target set invalidates the manifest.
//! - [`CacheData::files`] — the module files previously discovered (path +
//!   mtime). Reusing this list avoids re-parsing every file just to rediscover
//!   the module tree.
//! - [`CacheData::entries`] — per-file cached reports (source-stripped),
//!   keyed by a hash of the file path.
//!
//! A no-op run (nothing changed) therefore costs only `cargo metadata` + a few
//! stat calls + deserializing the cache; no source file is read unless it
//! actually produced reports (so their byte-accurate rendering needs the text).
//!
//! When something did change, the caller falls back to full discovery and
//! re-lints, reusing unchanged entries. Tasks run in parallel and only read the
//! shared map; the caller merges results and calls [`MlintCache::store`] once.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::linter::mlint_report::MlintReport;

/// One cached entry for a single source file.
#[derive(Serialize, Deserialize, Clone)]
pub struct FileCacheEntry {
    /// File modification time (nanoseconds since the Unix epoch).
    pub mtime: u128,
    /// SHA-256 of the file content that produced `reports`.
    pub content_hash: [u8; 32],
    /// Cached lint reports (with `source_code` stripped).
    pub reports: Vec<MlintReport>,
}

/// A previously discovered module file, used to validate cheaply on the next run.
#[derive(Serialize, Deserialize, Clone)]
pub struct CachedFileInfo {
    /// Absolute path of the source file.
    pub path: String,
    /// File modification time at cache time (nanoseconds since the Unix epoch).
    pub mtime: u128,
}

/// The whole cached state for a project.
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct CacheData {
    /// Hash of the sorted crate-root source paths that produced `files`.
    pub roots_signature: [u8; 32],
    /// Module files previously discovered (the lint input set).
    pub files: Vec<CachedFileInfo>,
    /// Per-file cached reports, keyed by `cache_key(path)`.
    pub entries: HashMap<[u8; 32], FileCacheEntry>,
}

/// Cache schema version; bumping it invalidates all cached entries.
pub const CACHE_VERSION: u32 = 3;

/// On-disk cache of lint reports.
pub struct MlintCache {
    file: PathBuf,
}

impl MlintCache {
    /// Single cache file rooted at `<target-dir>/mingling/mlint/cache.bin`.
    pub fn new(target_dir: &Path) -> Self {
        Self {
            file: target_dir.join("mingling").join("mlint").join("cache.bin"),
        }
    }

    /// Load the whole cache. Returns an empty default when missing/corrupt/outdated.
    pub fn load(&self) -> CacheData {
        let bytes = match std::fs::read(&self.file) {
            Ok(bytes) => bytes,
            Err(_) => return CacheData::default(),
        };
        let wrapper: CacheFile = match bincode::deserialize(&bytes) {
            Ok(wrapper) => wrapper,
            Err(_) => return CacheData::default(),
        };
        if wrapper.version != CACHE_VERSION {
            return CacheData::default();
        }
        wrapper.data
    }

    /// Persist the whole cache (best-effort, atomic via temp file + rename).
    pub fn store(&self, data: &CacheData) {
        if let Some(dir) = self.file.parent()
            && std::fs::create_dir_all(dir).is_err()
        {
            return;
        }
        let wrapper = CacheFile {
            version: CACHE_VERSION,
            data: data.clone(),
        };
        let Ok(bytes) = bincode::serialize(&wrapper) else {
            return;
        };
        let tmp = self.file.with_extension("bin.tmp");
        if std::fs::write(&tmp, bytes).is_ok() {
            let _ = std::fs::rename(&tmp, &self.file);
        }
    }
}

#[derive(Serialize, Deserialize)]
struct CacheFile {
    version: u32,
    data: CacheData,
}

/// Hash the sorted crate-root `.rs` paths so a target change invalidates the cache.
pub fn roots_signature(roots: &[String]) -> [u8; 32] {
    let mut sorted = roots.to_vec();
    sorted.sort();
    let mut hasher = Sha256::new();
    for root in &sorted {
        hasher.update(root.as_bytes());
        hasher.update(b"\0");
    }
    let digest = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    out
}

/// Stable raw cache key for a source file (SHA-256 of its path).
pub fn cache_key(file: &Path) -> [u8; 32] {
    to_array(&Sha256::digest(file.to_string_lossy().as_bytes()))
}

/// SHA-256 of `content`.
pub fn content_hash(content: &[u8]) -> [u8; 32] {
    to_array(&Sha256::digest(content))
}

fn to_array(
    digest: &sha2::digest::generic_array::GenericArray<u8, sha2::digest::consts::U32>,
) -> [u8; 32] {
    let mut out = [0u8; 32];
    out.copy_from_slice(digest);
    out
}

/// Nanoseconds since the Unix epoch for a file, or `None` when unavailable.
pub fn file_mtime_nanos(path: &Path) -> Option<u128> {
    std::fs::metadata(path)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| {
            t.duration_since(std::time::UNIX_EPOCH)
                .ok()
                .map(|d| d.as_nanos())
        })
}

/// Clear the (potentially large) `source_code` field from every report so it is
/// not duplicated in the cache.
pub fn strip_source(reports: &mut [MlintReport]) {
    for report in reports {
        report.source_code.clear();
        strip_source(&mut report.attached_reports);
    }
}

/// Restore `source_code` on every report from the current file content.
pub fn fill_source(reports: &mut [MlintReport], source: &str) {
    for report in reports {
        report.source_code = source.to_string();
        fill_source(&mut report.attached_reports, source);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linter::mlint_report::{LintSuggestion, MlintLevel};

    fn sample_report(content: &str) -> MlintReport {
        let mut report = MlintReport {
            file_name: "sample.rs".to_string(),
            source_code: content.to_string(),
            level: MlintLevel::Warning,
            lint_code: "direct_stdout_bypass".into(),
            message: "msg".into(),
            suggestions: vec![LintSuggestion {
                source: content.lines().next().unwrap_or_default().to_string(),
                line_start: 1,
                byte_range: 2..6,
                replacement: String::new(),
            }],
            ..Default::default()
        };
        let child = MlintReport {
            message: "help".into(),
            source_code: content.to_string(),
            ..Default::default()
        };
        report.attached_reports = vec![child];
        report
    }

    #[test]
    fn aggregated_cache_round_trip() {
        let dir = std::env::temp_dir().join(format!("mling-lint-cache-agg-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let cache = MlintCache::new(&dir);

        let content = "fn main() { println!(\"好的！\"); }\n";
        let file_a = dir.join("src").join("foo.rs");
        let file_b = dir.join("src").join("bar.rs");
        for f in [&file_a, &file_b] {
            std::fs::create_dir_all(f.parent().unwrap()).unwrap();
        }
        std::fs::write(&file_a, content).unwrap();
        std::fs::write(&file_b, "fn other() {}\n").unwrap();

        let mut a_reports = vec![sample_report(content)];
        strip_source(&mut a_reports);
        let mut entries = HashMap::new();
        entries.insert(
            cache_key(&file_a),
            FileCacheEntry {
                mtime: 1,
                content_hash: content_hash(content.as_bytes()),
                reports: a_reports,
            },
        );
        let data = CacheData {
            roots_signature: [7u8; 32],
            files: vec![
                CachedFileInfo {
                    path: file_a.to_string_lossy().into_owned(),
                    mtime: 1,
                },
                CachedFileInfo {
                    path: file_b.to_string_lossy().into_owned(),
                    mtime: 2,
                },
            ],
            entries,
        };
        cache.store(&data);

        let loaded = cache.load();
        assert_eq!(loaded.roots_signature, [7u8; 32]);
        assert_eq!(loaded.files.len(), 2);
        let entry = loaded.entries.get(&cache_key(&file_a)).unwrap();
        assert!(entry.reports[0].source_code.is_empty());
        assert!(entry.reports[0].attached_reports[0].source_code.is_empty());
        assert_eq!(entry.reports[0].suggestions[0].byte_range, 2..6);

        let mut restored = entry.reports.clone();
        fill_source(&mut restored, content);
        assert_eq!(restored[0].source_code, content);
        assert_eq!(restored[0].attached_reports[0].source_code, content);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_ignores_missing_or_bad_version() {
        let dir = std::env::temp_dir().join(format!("mling-lint-cache-bad-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let cache = MlintCache::new(&dir);
        assert!(cache.load().entries.is_empty());

        let dir2 = dir.join("sub");
        let cache2 = MlintCache::new(&dir2);
        std::fs::create_dir_all(cache2.file.parent().unwrap()).unwrap();
        std::fs::write(
            &cache2.file,
            bincode::serialize(&CacheFile {
                version: CACHE_VERSION + 1,
                data: CacheData::default(),
            })
            .unwrap(),
        )
        .unwrap();
        assert!(cache2.load().entries.is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
