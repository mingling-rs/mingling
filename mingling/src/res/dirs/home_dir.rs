// Doc Not Optimize
use std::path::{Path, PathBuf};

/// A global resource that provides the current user's home directory path.
///
/// This struct wraps a `PathBuf` representing the user's home directory.
///
/// **Platform notes:**
/// - On **Unix**: reads the `HOME` environment variable.
/// - On **Windows**: reads the `USERPROFILE` environment variable.
///
/// # Default behavior
///
/// The `Default` implementation calls [`ResHomeDir::new`] and **panics** if the home
/// directory cannot be determined (e.g., the relevant environment variable is not set).
/// Use [`ResHomeDir::new`] to handle this error gracefully.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResHomeDir {
    home: PathBuf,
}

impl ResHomeDir {
    /// Creates a new `ResHomeDir` by querying the environment for the user's home directory.
    ///
    /// # Errors
    ///
    /// Returns an error if the home directory cannot be determined (e.g., the `HOME` or
    /// `USERPROFILE` environment variable is not set).
    pub fn new() -> Result<Self, std::io::Error> {
        let home = home_dir_env().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "home directory not found")
        })?;
        Ok(Self { home })
    }
}

impl Default for ResHomeDir {
    fn default() -> Self {
        Self {
            home: home_dir_env().expect("home directory not found"),
        }
    }
}

impl From<PathBuf> for ResHomeDir {
    fn from(path: PathBuf) -> Self {
        Self { home: path }
    }
}

impl From<&Path> for ResHomeDir {
    fn from(path: &Path) -> Self {
        Self {
            home: path.to_path_buf(),
        }
    }
}

impl From<&PathBuf> for ResHomeDir {
    fn from(path: &PathBuf) -> Self {
        Self { home: path.clone() }
    }
}

impl From<ResHomeDir> for PathBuf {
    fn from(res: ResHomeDir) -> Self {
        res.home
    }
}

impl AsRef<Path> for ResHomeDir {
    fn as_ref(&self) -> &Path {
        self.home.as_path()
    }
}

impl std::ops::Deref for ResHomeDir {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.home
    }
}

impl std::ops::DerefMut for ResHomeDir {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.home
    }
}

/// Returns the user's home directory by checking platform-specific environment variables.
///
/// - Unix: `$HOME`
/// - Windows: `%USERPROFILE%`
fn home_dir_env() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var_os("USERPROFILE").map(PathBuf::from)
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::env::var_os("HOME").map(PathBuf::from)
    }
}
