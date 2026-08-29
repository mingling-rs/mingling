use just_fmt::snake_case;

/// Represents the shell environment for which the output format is intended.
///
/// This enum defines the supported shell types that can be used for
/// generating shell-specific command syntax, scripts, or completions.
///
/// # Behavior under `structural_renderer` feature
///
/// When the `structural_renderer` feature is enabled, this enum derives
/// [`serde::Serialize`](https://docs.rs/serde/latest/serde/trait.Serialize.html).
/// The serialization produces shell-specific string identifiers:
///
/// - `Bash` serializes to `"bash"`
/// - `Zsh` serializes to `"zsh"`
/// - `Fish` serializes to `"fish"`
/// - `Powershell` serializes to `"powershell"`
/// - `Elvish` serializes to `"elvish"`
/// - `Nushell` serializes to `"nushell"`
/// - `Other(name)` serializes to the inner string value
///
/// This allows the shell type to be transmitted as a plain string over
/// serialization boundaries (e.g., JSON, YAML) when using structural
/// rendering, while deserialization is handled by a separate process
/// (such as the `From<String>` implementation).
#[derive(Default, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "structural_renderer", derive(serde::Serialize))]
pub enum ShellFlag {
    /// Represents the Bash shell.
    #[default]
    Bash,
    /// Represents the Zsh shell.
    Zsh,
    /// Represents the Fish shell.
    Fish,
    /// Represents `PowerShell`.
    Powershell,
    /// Represents the Elvish shell.
    Elvish,
    /// Represents Nushell.
    Nushell,
    /// A custom or unsupported shell type, identified by the provided string.
    Other(String),
}

impl From<String> for ShellFlag {
    fn from(s: String) -> Self {
        match s.trim().to_lowercase().as_str() {
            "zsh" => Self::Zsh,
            "bash" => Self::Bash,
            "fish" => Self::Fish,
            "pwsh" | "ps1" | "powershell" => Self::Powershell,
            "elvish" | "elv" => Self::Elvish,
            "nushell" | "nu" => Self::Nushell,
            other => Self::Other(snake_case!(other)),
        }
    }
}

impl From<ShellFlag> for String {
    fn from(flag: ShellFlag) -> Self {
        match flag {
            ShellFlag::Zsh => "zsh".to_string(),
            ShellFlag::Bash => "bash".to_string(),
            ShellFlag::Fish => "fish".to_string(),
            ShellFlag::Powershell => "powershell".to_string(),
            ShellFlag::Elvish => "elvish".to_string(),
            ShellFlag::Nushell => "nushell".to_string(),
            ShellFlag::Other(s) => s,
        }
    }
}
