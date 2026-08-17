// Doc Not Optimize
//! Errors that can occur during the pathfinding process for Rust module resolution.
//!
//! This module defines all possible failure modes when traversing the module graph
//! of a Rust project, including I/O failures, missing modules, invalid path
//! attributes, missing entry points, and syntax parsing errors.

use std::fmt;
use std::path::PathBuf;

/// Errors that can occur during the pathfinding process for Rust module resolution.
///
/// This enum captures all possible failure modes when traversing the module graph
/// of a Rust project, including I/O failures, missing modules, invalid path
/// attributes, missing entry points, and syntax parsing errors.
#[derive(Debug)]
pub enum MinglingPathfinderError {
    /// An underlying I/O error occurred (e.g., file not found, permission denied).
    IoError(std::io::Error),

    /// A specific module declaration could not be resolved.
    ///
    /// `parent` is the directory containing the file that declared the module.
    /// `module_name` is the name of the module that could not be found.
    ModuleNotFound {
        /// The directory containing the file that declared the module.
        parent: PathBuf,
        /// The name of the module that could not be found.
        module_name: String,
    },

    /// A `#[path = "..."]` attribute points outside the project root.
    ///
    /// `file` is the file containing the invalid attribute.
    /// `path_attr` is the value of the `#[path]` attribute.
    PathPointsOutside {
        /// The file containing the invalid `#[path]` attribute.
        file: PathBuf,
        /// The value of the `#[path]` attribute that points outside the project.
        path_attr: String,
    },

    /// No entry point file (`main.rs`, `lib.rs`, or any file under `bin/`) was found.
    NoEntryPointFound,

    /// Failed to parse a Rust source file into its syntax tree.
    ///
    /// `path` is the file that failed to parse.
    /// `message` contains details from the parser.
    SynError {
        /// The file that failed to parse.
        path: PathBuf,
        /// Details from the parser about the parse failure.
        message: String,
    },

    /// `cargo metadata` could not be executed or parsed.
    ///
    /// `message` contains the underlying error from the cargo invocation.
    CargoMetadata(String),
}

impl fmt::Display for MinglingPathfinderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "IO error: {e}"),
            Self::ModuleNotFound {
                parent,
                module_name,
            } => {
                write!(
                    f,
                    "Module `{module_name}` not found relative to {}",
                    parent.display()
                )
            }
            Self::PathPointsOutside { file, path_attr } => {
                write!(
                    f,
                    "#[path = \"{path_attr}\"] in {} points outside the project",
                    file.display()
                )
            }
            Self::NoEntryPointFound => {
                write!(f, "No entry point found (main.rs, lib.rs, or bin/*.rs)")
            }
            Self::SynError { path, message } => {
                write!(f, "Failed to parse {}: {message}", path.display())
            }
            Self::CargoMetadata(message) => write!(f, "cargo metadata failed: {message}"),
        }
    }
}

impl std::error::Error for MinglingPathfinderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IoError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for MinglingPathfinderError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}
