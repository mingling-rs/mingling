// Doc Not Optimize
use std::path::{Path, PathBuf};

use crate::parser::Pickable;

mod rule;
pub use rule::*;

impl Pickable for Vec<PathBuf> {
    type Output = Self;

    fn pick(args: &mut crate::parser::Argument, flag: mingling_core::Flag) -> Option<Self::Output> {
        let raw: Vec<String> = args.pick_arguments(flag);
        let paths = raw.into_iter().map(PathBuf::from).collect();
        Some(paths)
    }
}

impl Pickable for PathBuf {
    type Output = Self;

    fn pick(args: &mut crate::parser::Argument, flag: mingling_core::Flag) -> Option<Self::Output> {
        let raw: String = args.pick_argument(flag)?;
        Some(Self::from(raw))
    }
}

/// Provides path checking methods for [`Vec<PathBuf>`]
///
/// This trait automatically provides implementations for `Into<Vec<PathBuf>>`
pub trait PathsChecker {
    /// Check if all paths in the list satisfy the rule
    fn is_all_passed(&self, rule: &PathCheckRule) -> bool
    where
        Self: Into<Vec<PathBuf>> + Clone,
    {
        check_paths(self.clone(), rule).is_ok()
    }

    /// Classify paths into (Passed, Stripped)
    ///
    /// Passed means paths that satisfy the rule, Stripped means paths that do not.
    fn classify(self, rule: &PathCheckRule) -> (Vec<PathBuf>, Vec<PathBuf>)
    where
        Self: Into<Vec<PathBuf>>,
    {
        let paths = self.into();
        let mut passed = Vec::new();
        let mut stripped = Vec::new();
        for path in paths {
            if check_path(&path, rule).is_ok() {
                passed.push(path);
            } else {
                stripped.push(path);
            }
        }
        (passed, stripped)
    }

    /// Return paths that satisfy the rule
    fn passed(self, rule: &PathCheckRule) -> Vec<PathBuf>
    where
        Self: Into<Vec<PathBuf>>,
    {
        self.classify(rule).0
    }

    /// Return paths that do not satisfy the rule
    fn stripped(self, rule: &PathCheckRule) -> Vec<PathBuf>
    where
        Self: Into<Vec<PathBuf>>,
    {
        self.classify(rule).1
    }
}

/// Provides path checking methods for [`PathBuf`]
///
/// This trait automatically provides implementations for `Into<PathBuf>`
pub trait PathChecker {
    fn is_passed(&self, rule: &PathCheckRule) -> bool
    where
        Self: Into<PathBuf> + Clone,
    {
        check_path(self.clone(), rule).is_ok()
    }
}

impl<T: Into<Vec<PathBuf>>> PathsChecker for T {}
impl<T: Into<PathBuf>> PathChecker for T {}

fn check_paths(path: impl Into<Vec<PathBuf>>, rule: &PathCheckRule) -> Result<(), ()> {
    let paths = path.into();
    for p in &paths {
        check_exist(p, rule)?;
        check_type(p, rule)?;
    }

    Ok(())
}

fn check_path(path: impl Into<PathBuf>, rule: &PathCheckRule) -> Result<(), ()> {
    let p = path.into();
    check_exist(&p, rule)?;
    check_type(&p, rule)?;

    Ok(())
}

fn check_exist(path: &Path, rule: &PathCheckRule) -> Result<(), ()> {
    let Some(exist_check) = &rule.exist_check else {
        return Ok(());
    };

    match exist_check {
        PathExistCheck::Exists => bool_to_result(path.exists()),
        PathExistCheck::NotExists => bool_to_result(!path.exists()),
    }
}

fn check_type(path: &Path, rule: &PathCheckRule) -> Result<(), ()> {
    let Some(type_check) = &rule.type_check else {
        return Ok(());
    };

    let is_dir = path.is_dir();
    let is_file = path.is_file();
    let is_symlink = path.is_symlink();

    if type_check.allow_dir && is_dir {
        return Ok(());
    }
    if type_check.allow_file && is_file {
        return Ok(());
    }
    if type_check.allow_symlink && is_symlink {
        return Ok(());
    }

    Err(())
}

const fn bool_to_result(b: bool) -> Result<(), ()> {
    if b { Ok(()) } else { Err(()) }
}
