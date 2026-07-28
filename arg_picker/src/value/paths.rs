use std::{
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
};

/// A file path.
///
/// `FilePath` is a wrapper type around `PathBuf` representing an arbitrary file path.
///
/// It implements `Pickable` and can be parsed by a `Picker`.
///
/// # Parsing Behavior
///
/// At the moment of parsing, checks whether the path exists and is a file.
/// If not a file, returns `NotFound`.
#[non_exhaustive]
pub struct FilePath {
    path: PathBuf,
}

/// A file path that should not exist.
///
/// `NoFilePath` is a wrapper type around `PathBuf` representing an arbitrary path
/// that must not currently point to an existing file.
///
/// It implements `Pickable` and can be parsed by a `Picker`.
///
/// # Parsing Behavior
///
/// At the moment of parsing, checks whether no file exists at the given path.
/// If a file already exists (regardless of whether it is a regular file, directory,
/// symlink, or other type), returns `NotFound`.
#[non_exhaustive]
pub struct NoFilePath {
    path: PathBuf,
}

/// A directory path.
///
/// `DirPath` is a wrapper type around `PathBuf` representing an arbitrary existing
/// directory path.
///
/// It implements `Pickable` and can be parsed by a `Picker`.
///
/// # Parsing Behavior
///
/// At the moment of parsing, checks whether a directory exists at the given path.
/// If no directory exists, returns `NotFound`.
#[non_exhaustive]
pub struct DirPath {
    path: PathBuf,
}

/// A directory path that should not exist.
///
/// `NoDirPath` is a wrapper type around `PathBuf` representing an arbitrary path
/// that must not currently point to an existing directory.
///
/// It implements `Pickable` and can be parsed by a `Picker`.
///
/// # Parsing Behavior
///
/// At the moment of parsing, checks whether no directory exists at the given path.
/// If a directory already exists (regardless of whether it is a regular file, file,
/// symlink, or other type), returns `NotFound`.
#[non_exhaustive]
pub struct NoDirPath {
    path: PathBuf,
}

/// A symbolic link path.
///
/// `SymlinkPath` is a wrapper type around `PathBuf` representing an existing symbolic link.
///
/// It implements `Pickable` and can be parsed by a `Picker`.
///
/// # Parsing Behavior
///
/// At the moment of parsing, checks whether the path exists and is a symlink.
/// If not a symlink, returns `NotFound`.
#[non_exhaustive]
pub struct SymlinkPath {
    path: PathBuf,
}

/// A symbolic link path that should not exist.
///
/// `NoSymlinkPath` is a wrapper type around `PathBuf` representing an arbitrary path
/// that must not currently point to an existing symbolic link.
///
/// It implements `Pickable` and can be parsed by a `Picker`.
///
/// # Parsing Behavior
///
/// At the moment of parsing, checks whether no symlink exists at the given path.
/// If a symlink already exists (regardless of whether it points to a file, directory,
/// or other type), returns `NotFound`.
#[non_exhaustive]
pub struct NoSymlinkPath {
    path: PathBuf,
}

/// A path that should not exist at all.
///
/// `NoPath` is a wrapper type around `PathBuf` representing an arbitrary path
/// that must not currently exist on the filesystem (as a file, directory, symlink,
/// or any other type).
///
/// It implements `Pickable` and can be parsed by a `Picker`.
///
/// # Parsing Behavior
///
/// At the moment of parsing, checks whether any filesystem entry exists at the given path.
/// If anything exists at that path, returns `NotFound`.
#[non_exhaustive]
pub struct NoPath {
    path: PathBuf,
}

/// Implements common trait impls (From, AsRef, Deref, DerefMut) for a path wrapper type.
macro_rules! impl_path_traits {
    ($type:ident) => {
        impl From<PathBuf> for $type {
            fn from(value: PathBuf) -> Self {
                $type { path: value }
            }
        }

        impl From<&PathBuf> for $type {
            fn from(value: &PathBuf) -> Self {
                $type {
                    path: value.clone(),
                }
            }
        }

        impl AsRef<Path> for $type {
            fn as_ref(&self) -> &Path {
                &self.path
            }
        }

        impl DerefMut for $type {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.path
            }
        }

        impl Deref for $type {
            type Target = PathBuf;

            fn deref(&self) -> &Self::Target {
                &self.path
            }
        }

        impl From<$type> for PathBuf {
            fn from(value: $type) -> Self {
                value.path
            }
        }

        impl From<&$type> for PathBuf {
            fn from(value: &$type) -> Self {
                value.path.clone()
            }
        }
    };
}

impl_path_traits!(FilePath);
impl_path_traits!(NoFilePath);
impl_path_traits!(DirPath);
impl_path_traits!(NoDirPath);
impl_path_traits!(SymlinkPath);
impl_path_traits!(NoSymlinkPath);
impl_path_traits!(NoPath);

/// Recursive file paths.
///
/// `RecursiveFiles` is a wrapper type around `Vec<PathBuf>` representing a list of
/// existing files.
///
/// It implements `Pickable` and can be parsed by a `Picker`.
///
/// # Parsing Behavior
///
/// - If a file path is given, returns a list of length 1 containing that file.
/// - If a directory path is given, recursively collects all files under that directory
///   and returns them as a list.
#[non_exhaustive]
pub struct RecursiveFiles {
    paths: Vec<PathBuf>,
}

impl From<Vec<PathBuf>> for RecursiveFiles {
    fn from(paths: Vec<PathBuf>) -> Self {
        Self { paths }
    }
}

impl From<RecursiveFiles> for Vec<PathBuf> {
    fn from(value: RecursiveFiles) -> Self {
        value.paths
    }
}

impl AsRef<[PathBuf]> for RecursiveFiles {
    fn as_ref(&self) -> &[PathBuf] {
        &self.paths
    }
}

impl Deref for RecursiveFiles {
    type Target = Vec<PathBuf>;

    fn deref(&self) -> &Self::Target {
        &self.paths
    }
}

impl DerefMut for RecursiveFiles {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.paths
    }
}

impl RecursiveFiles {
    /// Returns the number of file paths.
    pub fn len(&self) -> usize {
        self.paths.len()
    }

    /// Returns `true` if there are no file paths.
    pub fn is_empty(&self) -> bool {
        self.paths.is_empty()
    }

    /// Returns an iterator over the file paths.
    pub fn iter(&self) -> std::slice::Iter<'_, PathBuf> {
        self.paths.iter()
    }
}

impl From<Vec<RecursiveFiles>> for RecursiveFiles {
    fn from(value: Vec<RecursiveFiles>) -> Self {
        Self {
            paths: value.into_iter().flat_map(|r| r.paths).collect(),
        }
    }
}

/// Trait for types that can be combined into a single `RecursiveFiles`.
pub trait IntoRecursiveFiles {
    /// Combines multiple sources of file paths into a single `RecursiveFiles`.
    fn combine(self) -> RecursiveFiles;
}

impl<T> IntoRecursiveFiles for Vec<T>
where
    T: Into<RecursiveFiles>,
{
    fn combine(self) -> RecursiveFiles {
        self.into_iter()
            .map(Into::into)
            .collect::<Vec<RecursiveFiles>>()
            .into()
    }
}

impl<T> IntoRecursiveFiles for &[T]
where
    T: Into<RecursiveFiles> + Clone,
{
    fn combine(self) -> RecursiveFiles {
        self.iter()
            .cloned()
            .map(Into::into)
            .collect::<Vec<RecursiveFiles>>()
            .into()
    }
}

impl<T, const N: usize> IntoRecursiveFiles for [T; N]
where
    T: Into<RecursiveFiles>,
{
    fn combine(self) -> RecursiveFiles {
        self.into_iter()
            .map(Into::into)
            .collect::<Vec<RecursiveFiles>>()
            .into()
    }
}
