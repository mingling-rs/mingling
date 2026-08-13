// Doc Not Optimize
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    PickerArgResult::{self, NotFound, Parsed, Unparsed},
    SinglePickable,
    value::{
        DirPath, FilePath, NoDirPath, NoFilePath, NoPath, NoSymlinkPath, RecursiveFiles,
        SymlinkPath,
    },
};

impl SinglePickable for FilePath {
    fn pick_single(str: Option<&str>) -> PickerArgResult<Self> {
        match <PathBuf as SinglePickable>::pick_single(str) {
            Parsed(path) => {
                if path.exists() && path.is_file() {
                    Parsed(Self::from(path))
                } else {
                    NotFound
                }
            }
            Unparsed => Unparsed,
            NotFound => NotFound,
        }
    }
}

impl SinglePickable for NoFilePath {
    fn pick_single(str: Option<&str>) -> PickerArgResult<Self> {
        match <PathBuf as SinglePickable>::pick_single(str) {
            Parsed(path) => {
                if !path.exists() || !path.is_file() {
                    Parsed(Self::from(path))
                } else {
                    NotFound
                }
            }
            Unparsed => Unparsed,
            NotFound => NotFound,
        }
    }
}

impl SinglePickable for DirPath {
    fn pick_single(str: Option<&str>) -> PickerArgResult<Self> {
        match <PathBuf as SinglePickable>::pick_single(str) {
            Parsed(path) => {
                if path.exists() && path.is_dir() {
                    Parsed(Self::from(path))
                } else {
                    NotFound
                }
            }
            Unparsed => Unparsed,
            NotFound => NotFound,
        }
    }
}

impl SinglePickable for NoDirPath {
    fn pick_single(str: Option<&str>) -> PickerArgResult<Self> {
        match <PathBuf as SinglePickable>::pick_single(str) {
            Parsed(path) => {
                if !path.exists() || !path.is_dir() {
                    Parsed(Self::from(path))
                } else {
                    NotFound
                }
            }
            Unparsed => Unparsed,
            NotFound => NotFound,
        }
    }
}

impl SinglePickable for SymlinkPath {
    fn pick_single(str: Option<&str>) -> PickerArgResult<Self> {
        match <PathBuf as SinglePickable>::pick_single(str) {
            Parsed(path) => {
                if path.exists() && path.is_symlink() {
                    Parsed(Self::from(path))
                } else {
                    NotFound
                }
            }
            Unparsed => Unparsed,
            NotFound => NotFound,
        }
    }
}

impl SinglePickable for NoSymlinkPath {
    fn pick_single(str: Option<&str>) -> PickerArgResult<Self> {
        match <PathBuf as SinglePickable>::pick_single(str) {
            Parsed(path) => {
                if !path.exists() || !path.is_symlink() {
                    Parsed(Self::from(path))
                } else {
                    NotFound
                }
            }
            Unparsed => Unparsed,
            NotFound => NotFound,
        }
    }
}

impl SinglePickable for NoPath {
    fn pick_single(str: Option<&str>) -> PickerArgResult<Self> {
        match <PathBuf as SinglePickable>::pick_single(str) {
            Parsed(path) => {
                if path.exists() {
                    NotFound
                } else {
                    Parsed(Self::from(path))
                }
            }
            Unparsed => Unparsed,
            NotFound => NotFound,
        }
    }
}

impl SinglePickable for RecursiveFiles {
    fn pick_single(str: Option<&str>) -> PickerArgResult<Self> {
        match <PathBuf as SinglePickable>::pick_single(str) {
            Parsed(path) => {
                if !path.exists() {
                    return NotFound;
                }
                if path.is_file() || path.is_symlink() {
                    return Parsed(Self::from(vec![path]));
                }
                let mut entries = Vec::new();
                if let Ok(dir_entries) = fs::read_dir(&path) {
                    for entry in dir_entries.flatten() {
                        let entry_path = entry.path();
                        if entry_path.is_file() || entry_path.is_symlink() {
                            entries.push(entry_path);
                        } else if entry_path.is_dir() {
                            collect_files(&entry_path, &mut entries);
                        }
                    }
                }
                Parsed(Self::from(entries))
            }
            Unparsed => Unparsed,
            NotFound => NotFound,
        }
    }
}

fn collect_files(dir: &Path, entries: &mut Vec<PathBuf>) {
    if let Ok(dir_entries) = fs::read_dir(dir) {
        for entry in dir_entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_file() || entry_path.is_symlink() {
                entries.push(entry_path);
            } else if entry_path.is_dir() {
                collect_files(&entry_path, entries);
            }
        }
    }
}
