// Doc Not Optimize
use std::{
    env::current_exe,
    path::{Path, PathBuf},
};

/// A global resource that provides the path of the current executable.
///
/// This struct wraps a `PathBuf` representing the full filesystem path of the
/// currently running executable at the time of creation.
///
/// # Default behavior
///
/// The `Default` implementation calls `std::env::current_exe().unwrap()` internally,
/// which will **panic** if the executable path cannot be determined (e.g., if the
/// `/proc` filesystem is not available). If you want to handle this error gracefully,
/// use [`ResCurrentExe::new`] instead, which returns a `Result`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResCurrentExe {
    exe: PathBuf,
}

impl ResCurrentExe {
    /// Creates a new `ResCurrentExe` by querying the OS for the current executable path.
    ///
    /// # Errors
    ///
    /// Returns an [`std::io::Error`] if the OS is unable to provide the path of the
    /// current executable. This can occur, for example, when the `/proc` filesystem
    /// is not mounted on Linux, or when the process handle is invalid on Windows.
    pub fn new() -> Result<Self, std::io::Error> {
        Ok(Self {
            exe: current_exe()?,
        })
    }
}

impl Default for ResCurrentExe {
    fn default() -> Self {
        Self {
            exe: current_exe().unwrap(),
        }
    }
}

impl From<PathBuf> for ResCurrentExe {
    fn from(path: PathBuf) -> Self {
        Self { exe: path }
    }
}

impl From<&Path> for ResCurrentExe {
    fn from(path: &Path) -> Self {
        Self {
            exe: path.to_path_buf(),
        }
    }
}

impl From<&PathBuf> for ResCurrentExe {
    fn from(path: &PathBuf) -> Self {
        Self { exe: path.clone() }
    }
}

impl From<ResCurrentExe> for PathBuf {
    fn from(res: ResCurrentExe) -> Self {
        res.exe
    }
}

impl AsRef<Path> for ResCurrentExe {
    fn as_ref(&self) -> &Path {
        self.exe.as_path()
    }
}

impl std::ops::Deref for ResCurrentExe {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.exe
    }
}

impl std::ops::DerefMut for ResCurrentExe {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.exe
    }
}
