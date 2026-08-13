// Doc Not Optimize
use std::{
    env::temp_dir,
    path::{Path, PathBuf},
};

/// A global resource that provides the system temporary directory path.
///
/// This struct wraps a `PathBuf` representing the platform's temporary directory
/// (e.g., `/tmp` on Unix, `%TEMP%` on Windows).
///
/// Note that `std::env::temp_dir()` is infallible — it always returns a path,
/// falling back to a platform-specific default (e.g., `/tmp` on Unix) if the
/// environment variable is not set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResTempDir {
    tmp: PathBuf,
}

impl ResTempDir {
    /// Creates a new `ResTempDir` by querying the OS for the system temporary directory.
    ///
    /// This method is infallible since `std::env::temp_dir()` always succeeds,
    /// returning a platform-specific default when environment variables are unset.
    #[must_use]
    pub fn new() -> Self {
        Self { tmp: temp_dir() }
    }
}

impl Default for ResTempDir {
    fn default() -> Self {
        Self::new()
    }
}

impl From<PathBuf> for ResTempDir {
    fn from(path: PathBuf) -> Self {
        Self { tmp: path }
    }
}

impl From<&Path> for ResTempDir {
    fn from(path: &Path) -> Self {
        Self {
            tmp: path.to_path_buf(),
        }
    }
}

impl From<&PathBuf> for ResTempDir {
    fn from(path: &PathBuf) -> Self {
        Self { tmp: path.clone() }
    }
}

impl From<ResTempDir> for PathBuf {
    fn from(res: ResTempDir) -> Self {
        res.tmp
    }
}

impl AsRef<Path> for ResTempDir {
    fn as_ref(&self) -> &Path {
        self.tmp.as_path()
    }
}

impl std::ops::Deref for ResTempDir {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.tmp
    }
}

impl std::ops::DerefMut for ResTempDir {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tmp
    }
}
