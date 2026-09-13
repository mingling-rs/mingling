/// Exit code indicating successful command execution.
pub const EXIT_SUCCESS: i32 = 0;

/// Exit code for general errors.
pub const EXIT_GENERAL_ERR: i32 = 1;

/// Exit code for incorrect command usage (or invalid arguments).
pub const EXIT_USAGE_ERR: i32 = 2;

/// Exit code indicating permission denied (or the command is not executable).
pub const EXIT_PERM_DENIED: i32 = 126;

/// Exit code for command not found (or PATH error).
pub const EXIT_CMD_NOT_FOUND: i32 = 127;
