/// Path check rule
#[derive(Default)]
pub struct PathCheckRule {
    pub exist_check: Option<PathExistCheck>,
    pub type_check: Option<PathTypeCheck>,
}

/// Path existence check
pub enum PathExistCheck {
    Exists,
    NotExists,
}

/// Path type check
pub struct PathTypeCheck {
    /// Whether the path is allowed to be a file
    pub allow_file: bool,

    /// Whether the path is allowed to be a directory
    pub allow_dir: bool,

    /// Whether the path is allowed to be a symlink
    pub allow_symlink: bool,
}

impl PathCheckRule {
    /// Creates a new `PathCheckRule` with default values
    #[must_use]
    pub const fn new() -> Self {
        Self {
            exist_check: None,
            type_check: None,
        }
    }

    /// Allows the path to be a file
    #[must_use]
    pub const fn allow_file(self) -> Self {
        match self.type_check {
            Some(type_check) => Self {
                type_check: Some(PathTypeCheck {
                    allow_file: true,
                    allow_dir: type_check.allow_dir,
                    allow_symlink: type_check.allow_symlink,
                }),
                ..self
            },
            None => Self {
                type_check: Some(PathTypeCheck {
                    allow_file: true,
                    allow_dir: false,
                    allow_symlink: false,
                }),
                ..self
            },
        }
    }

    /// Allows the path to be a directory
    #[must_use]
    pub const fn allow_dir(self) -> Self {
        match self.type_check {
            Some(type_check) => Self {
                type_check: Some(PathTypeCheck {
                    allow_file: type_check.allow_file,
                    allow_dir: true,
                    allow_symlink: type_check.allow_symlink,
                }),
                ..self
            },
            None => Self {
                type_check: Some(PathTypeCheck {
                    allow_file: false,
                    allow_dir: true,
                    allow_symlink: false,
                }),
                ..self
            },
        }
    }

    /// Allows the path to be a symlink
    #[must_use]
    pub const fn allow_symlink(self) -> Self {
        match self.type_check {
            Some(type_check) => Self {
                type_check: Some(PathTypeCheck {
                    allow_file: type_check.allow_file,
                    allow_dir: type_check.allow_dir,
                    allow_symlink: true,
                }),
                ..self
            },
            None => Self {
                type_check: Some(PathTypeCheck {
                    allow_file: false,
                    allow_dir: false,
                    allow_symlink: true,
                }),
                ..self
            },
        }
    }

    /// Denies the path from being a file
    #[must_use]
    pub const fn deny_file(self) -> Self {
        match self.type_check {
            Some(type_check) => Self {
                type_check: Some(PathTypeCheck {
                    allow_file: false,
                    allow_dir: type_check.allow_dir,
                    allow_symlink: type_check.allow_symlink,
                }),
                ..self
            },
            None => Self {
                type_check: Some(PathTypeCheck {
                    allow_file: false,
                    allow_dir: true,
                    allow_symlink: true,
                }),
                ..self
            },
        }
    }

    /// Denies the path from being a directory
    #[must_use]
    pub const fn deny_dir(self) -> Self {
        match self.type_check {
            Some(type_check) => Self {
                type_check: Some(PathTypeCheck {
                    allow_file: type_check.allow_file,
                    allow_dir: false,
                    allow_symlink: type_check.allow_symlink,
                }),
                ..self
            },
            None => Self {
                type_check: Some(PathTypeCheck {
                    allow_file: true,
                    allow_dir: false,
                    allow_symlink: true,
                }),
                ..self
            },
        }
    }

    /// Denies the path from being a symlink
    #[must_use]
    pub const fn deny_symlink(self) -> Self {
        match self.type_check {
            Some(type_check) => Self {
                type_check: Some(PathTypeCheck {
                    allow_file: type_check.allow_file,
                    allow_dir: type_check.allow_dir,
                    allow_symlink: false,
                }),
                ..self
            },
            None => Self {
                type_check: Some(PathTypeCheck {
                    allow_file: true,
                    allow_dir: true,
                    allow_symlink: false,
                }),
                ..self
            },
        }
    }

    /// Requires the path to be a file (overrides type checks)
    #[must_use]
    pub const fn must_file(self) -> Self {
        Self {
            type_check: Some(PathTypeCheck {
                allow_file: true,
                allow_dir: false,
                allow_symlink: false,
            }),
            ..self
        }
    }

    /// Requires the path to be a directory (overrides type checks)
    #[must_use]
    pub const fn must_dir(self) -> Self {
        Self {
            type_check: Some(PathTypeCheck {
                allow_file: false,
                allow_dir: true,
                allow_symlink: false,
            }),
            ..self
        }
    }

    /// Requires the path to be a symlink (overrides type checks)
    #[must_use]
    pub const fn must_symlink(self) -> Self {
        Self {
            type_check: Some(PathTypeCheck {
                allow_file: false,
                allow_dir: false,
                allow_symlink: true,
            }),
            ..self
        }
    }

    /// Requires the path to exist
    #[must_use]
    pub const fn must_exist(self) -> Self {
        Self {
            exist_check: Some(PathExistCheck::Exists),
            ..self
        }
    }

    /// Requires the path to not exist
    #[must_use]
    pub const fn must_not_exist(self) -> Self {
        Self {
            exist_check: Some(PathExistCheck::NotExists),
            ..self
        }
    }
}
