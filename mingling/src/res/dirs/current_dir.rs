use std::{
    env::current_dir,
    path::{Path, PathBuf},
};

/// A global resource for the Mingling library that provides the current working directory.
///
/// This struct wraps a `PathBuf` representing the current working directory at the time of
/// creation. It is used as a shared resource in the Mingling framework so that multiple
/// components can access the same base directory without repeatedly querying the OS.
///
/// # Default behavior
///
/// The `Default` implementation calls `std::env::current_dir().unwrap()` internally, which will
/// **panic** if the current directory cannot be determined (e.g., if it has been deleted). If
/// you want to handle this error gracefully, use [`ResCurrentDir::new`] instead, which returns
/// a `Result`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResCurrentDir {
    cwd: PathBuf,
}

impl ResCurrentDir {
    /// Creates a new `ResCurrentDir` by querying the OS for the current working directory.
    ///
    /// # Errors
    ///
    /// Returns an `Err` if the current directory cannot be determined (e.g., the directory has
    /// been deleted or permissions are insufficient).
    ///
    /// Unlike the `Default` implementation, this method does not panic on failure.
    pub fn new() -> Result<Self, std::io::Error> {
        Ok(Self {
            cwd: current_dir()?,
        })
    }
}

impl Default for ResCurrentDir {
    fn default() -> Self {
        Self {
            cwd: current_dir().unwrap(),
        }
    }
}

impl From<PathBuf> for ResCurrentDir {
    fn from(path: PathBuf) -> Self {
        Self { cwd: path }
    }
}

impl From<&Path> for ResCurrentDir {
    fn from(path: &Path) -> Self {
        Self {
            cwd: path.to_path_buf(),
        }
    }
}

impl From<&PathBuf> for ResCurrentDir {
    fn from(path: &PathBuf) -> Self {
        Self { cwd: path.clone() }
    }
}

impl From<ResCurrentDir> for PathBuf {
    fn from(res: ResCurrentDir) -> Self {
        res.cwd
    }
}

impl AsRef<Path> for ResCurrentDir {
    fn as_ref(&self) -> &Path {
        self.cwd.as_path()
    }
}

impl std::ops::Deref for ResCurrentDir {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.cwd
    }
}

impl std::ops::DerefMut for ResCurrentDir {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cwd
    }
}
