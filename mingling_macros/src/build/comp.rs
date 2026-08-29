use std::path::PathBuf;

use just_template::tmpl;

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
pub(crate) enum ShellFlag {
    /// Represents the Bash shell.
    #[default]
    Bash,
    /// Represents the Zsh shell.
    Zsh,
    /// Represents the Fish shell.
    Fish,
    /// Represents `PowerShell`.
    Powershell,
    /// Represents the Elvish shell. (WIP)
    #[allow(unused)]
    Elvish,
    /// Represents Nushell. (WIP)
    #[allow(unused)]
    Nushell,
}

const TMPL_COMP_BASH: &str = include_str!("../../tmpls/comps/bash.sh");
const TMPL_COMP_ZSH: &str = include_str!("../../tmpls/comps/zsh.zsh");
const TMPL_COMP_FISH: &str = include_str!("../../tmpls/comps/fish.fish");
const TMPL_COMP_PWSH: &str = include_str!("../../tmpls/comps/pwsh.ps1");
const TMPL_COMP_ELVISH: &str = include_str!("../../tmpls/comps/elvish.elv");
const TMPL_COMP_NUSH: &str = include_str!("../../tmpls/comps/nush.nu");

/// Generate shell completion scripts for a given binary name.
///
/// Generates completion scripts for all supported shells regardless of the
/// platform: PowerShell, Zsh, Bash, Fish, Elvish, and Nushell.
/// Scripts are written to the `OUT_DIR` (or `target/` if `OUT_DIR` is not set).
///
/// # Errors
///
/// Returns an [`std::io::Error`] if a script cannot be written.
pub(crate) fn build_comp_scripts(name: &str) -> Result<(), std::io::Error> {
    build_comp_script(&ShellFlag::Powershell, name)?;
    build_comp_script(&ShellFlag::Zsh, name)?;
    build_comp_script(&ShellFlag::Bash, name)?;
    build_comp_script(&ShellFlag::Fish, name)?;

    // The completion logic for these scripts is not yet complete,
    // so they are temporarily excluded from the generation list.
    //
    // build_comp_script(&ShellFlag::Elvish, name)?;
    // build_comp_script(&ShellFlag::Nushell, name)?;
    Ok(())
}

/// Generate a shell completion script for a specific shell.
///
/// This function takes a shell flag and a binary name, selects the appropriate
/// template, substitutes the binary name into the template, and writes the
/// resulting completion script to the Mingling build directory
/// (`{target_directory}/mingling/`, resolved via `cargo metadata`).
///
/// # Errors
///
/// Returns an [`std::io::Error`] if the script cannot be written.
pub(crate) fn build_comp_script(
    shell_flag: &ShellFlag,
    bin_name: &str,
) -> Result<(), std::io::Error> {
    let output_dir = comp_output_dir()?;
    build_comp_script_to(shell_flag, bin_name, &output_dir.to_string_lossy())
}

/// The directory where completion scripts are written: `{target_directory}/mingling/`.
fn comp_output_dir() -> Result<PathBuf, std::io::Error> {
    mingling_pathf::build_output_dir().map_err(|e| std::io::Error::other(e.to_string()))
}

/// Generate a shell completion script to a specified directory.
///
/// This function takes a shell flag, a binary name, and a target directory path,
/// selects the appropriate template, substitutes the binary name into the template,
/// and writes the resulting completion script to the specified directory.
///
/// # Errors
///
/// Returns an [`std::io::Error`] if the script cannot be written.
pub(crate) fn build_comp_script_to(
    shell_flag: &ShellFlag,
    bin_name: &str,
    target_dir: &str,
) -> Result<(), std::io::Error> {
    let (tmpl_str, ext) = get_tmpl(shell_flag);
    let mut tmpl = just_template::Template::from(tmpl_str);
    tmpl!(bin_name = bin_name);
    let target_path = std::path::PathBuf::from(target_dir);
    std::fs::create_dir_all(&target_path)?;
    let output_path = target_path.join(format!("{bin_name}_comp{ext}"));
    std::fs::write(&output_path, tmpl.to_string())
}

const fn get_tmpl(shell_flag: &ShellFlag) -> (&'static str, &'static str) {
    match shell_flag {
        ShellFlag::Bash => (TMPL_COMP_BASH, ".sh"),
        ShellFlag::Zsh => (TMPL_COMP_ZSH, ".zsh"),
        ShellFlag::Fish => (TMPL_COMP_FISH, ".fish"),
        ShellFlag::Powershell => (TMPL_COMP_PWSH, ".ps1"),
        ShellFlag::Elvish => (TMPL_COMP_ELVISH, ".elv"),
        ShellFlag::Nushell => (TMPL_COMP_NUSH, ".nu"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_tmpl_bash() {
        let (tmpl, ext) = get_tmpl(&ShellFlag::Bash);
        assert_eq!(ext, ".sh");
        assert!(!tmpl.is_empty(), "bash template should not be empty");
    }

    #[test]
    fn get_tmpl_zsh() {
        let (tmpl, ext) = get_tmpl(&ShellFlag::Zsh);
        assert_eq!(ext, ".zsh");
        assert!(!tmpl.is_empty(), "zsh template should not be empty");
    }

    #[test]
    fn get_tmpl_fish() {
        let (tmpl, ext) = get_tmpl(&ShellFlag::Fish);
        assert_eq!(ext, ".fish");
        assert!(!tmpl.is_empty(), "fish template should not be empty");
    }

    #[test]
    fn get_tmpl_powershell() {
        let (tmpl, ext) = get_tmpl(&ShellFlag::Powershell);
        assert_eq!(ext, ".ps1");
        assert!(!tmpl.is_empty(), "powershell template should not be empty");
    }

    #[test]
    fn get_tmpl_elvish() {
        let (tmpl, ext) = get_tmpl(&ShellFlag::Elvish);
        assert_eq!(ext, ".elv");
        assert!(!tmpl.is_empty(), "elvish template should not be empty");
    }

    #[test]
    fn get_tmpl_nushell() {
        let (tmpl, ext) = get_tmpl(&ShellFlag::Nushell);
        assert_eq!(ext, ".nu");
        assert!(!tmpl.is_empty(), "nushell template should not be empty");
    }
}
