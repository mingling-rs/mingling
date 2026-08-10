/// Output mode for error messages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorOutput {
    /// Show error messages
    Show,
    /// Hide error messages
    Hide,
}

/// Output mode for rendered results
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderOutput {
    /// Render results and output
    Show,
    /// Hide rendered results
    Hide,
}

/// Panic message handling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanicSilence {
    /// Allow panic messages to be shown
    Show,
    /// Silence panic messages
    Silence,
}

/// Verbosity level for program output
///
/// **NOTE**: Convention only, not a configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verbosity {
    /// Normal output
    Normal,
    /// Verbose output: provide detailed information
    Verbose,
    /// Quiet mode: suppress status messages, show only errors and results
    Quiet,
    /// Debug mode: output internal state and detailed diagnostics
    Debug,
}

/// Color output mode
///
/// **NOTE**: Convention only, not a configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorOutput {
    /// Enable colored output
    Enabled,
    /// Disable colored output
    Disabled,
}

/// Progress indicator mode
///
/// Automatically disabled when stdout is not a tty.
///
/// **NOTE**: Convention only, not a configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressOutput {
    /// Show progress indicators (e.g. progress bars, spinners)
    Enabled,
    /// Hide progress indicators
    Disabled,
}

/// Program stdout settings
#[derive(Debug, Clone)]
pub struct ProgramStdoutSetting {
    /// Output error messages
    pub error_output: ErrorOutput,

    /// Render results and output
    pub render_output: RenderOutput,

    /// Silence panic messages
    pub silence_panic: PanicSilence,

    /// Verbosity level for program output
    ///
    /// **NOTE**: Convention only, not a configuration
    pub verbosity: Verbosity,

    /// Enable colored output
    ///
    /// **NOTE**: Convention only, not a configuration
    pub color: ColorOutput,

    /// Show progress indicators (e.g. progress bars, spinners)
    ///
    /// **NOTE**: Convention only, not a configuration
    pub progress: ProgressOutput,

    #[cfg(feature = "clap")]
    /// Behavior when Clap Dispatcher outputs help information
    pub clap_help_print_behaviour: ClapHelpPrintBehaviour,
}

/// Behavior when Clap Dispatcher outputs help information
#[cfg(feature = "clap")]
#[derive(Debug, Default, Clone)]
pub enum ClapHelpPrintBehaviour {
    /// Write help information to `RenderResult` instead of printing to stdout directly.
    ///
    /// This allows the help text to be captured and processed as part of the program's
    /// structured output, which is useful when integrating with external tools or
    /// when the output needs to be further transformed.
    WriteToRenderResult,

    /// Print help information directly to stdout.
    ///
    /// This is the default behavior, which prints help text immediately to the terminal
    /// without any intermediate processing or capture.
    #[default]
    PrintDirectly,
}

impl Default for ProgramStdoutSetting {
    fn default() -> Self {
        Self {
            error_output: ErrorOutput::Show,
            render_output: RenderOutput::Show,
            silence_panic: PanicSilence::Show,
            verbosity: Verbosity::Normal,
            color: ColorOutput::Enabled,
            progress: ProgressOutput::Enabled,
            #[cfg(feature = "clap")]
            clap_help_print_behaviour: ClapHelpPrintBehaviour::default(),
        }
    }
}

/// Confirmation mode for user prompts
///
/// **NOTE**: Convention only, not a configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmationMode {
    /// Require confirmation from the user
    Confirm,
    /// Skip user confirmation step
    Skip,
}

/// Execution mode
///
/// **NOTE**: Convention only, not a configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    /// Normal execution
    Normal,
    /// Dry-run mode: simulate actions without making changes
    DryRun,
    /// Force execution, skipping safety checks
    Force,
}

/// Interaction mode
///
/// **NOTE**: Convention only, not a configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractionMode {
    /// Interactive terminal (has a tty)
    Interactive,
    /// Non-interactive terminal
    NonInteractive,
}

/// Yes assumption mode for prompts
///
/// **NOTE**: Convention only, not a configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YesAssumption {
    /// Do not assume "yes" for any prompt
    None,
    /// Assume "yes" for all confirmation prompts
    AssumeYes,
}

/// Program user context
#[derive(Debug, Clone)]
pub struct ProgramUserContext {
    /// View help information instead of running the command
    pub help: bool,

    /// Execute hooks during the program lifecycle
    pub run_hook: bool,

    /// Confirmation mode for user prompts
    ///
    /// **NOTE**: Convention only, not a configuration
    pub confirmation: ConfirmationMode,

    /// Execution mode
    ///
    /// **NOTE**: Convention only, not a configuration
    pub execution: ExecutionMode,

    /// Whether the program is running in an interactive terminal (has a tty)
    ///
    /// **NOTE**: Convention only, not a configuration
    pub interaction: InteractionMode,

    /// Whether to assume "yes" for all confirmation prompts
    ///
    /// **NOTE**: Convention only, not a configuration
    pub yes_assumption: YesAssumption,
}

impl Default for ProgramUserContext {
    fn default() -> Self {
        Self {
            help: false,
            run_hook: true,
            confirmation: ConfirmationMode::Confirm,
            execution: ExecutionMode::Normal,
            interaction: InteractionMode::NonInteractive,
            yes_assumption: YesAssumption::None,
        }
    }
}

#[cfg(feature = "structural_renderer")]
#[derive(Debug, Clone, Default)]
/// Settings for the structural renderer output format.
///
/// Controls how structured data (e.g., JSON, YAML, TOML) is rendered to stdout.
pub enum StructuralRendererSetting {
    /// Do not render structured output (use default formatting).
    #[default]
    Disable,
    /// Render output as compact JSON.
    #[cfg(feature = "json_serde_fmt")]
    Json,
    /// Render output as pretty-printed JSON.
    #[cfg(feature = "json_serde_fmt")]
    JsonPretty,
    /// Render output as YAML.
    #[cfg(feature = "yaml_serde_fmt")]
    Yaml,
    /// Render output as TOML.
    #[cfg(feature = "toml_serde_fmt")]
    Toml,
    /// Render output as RON.
    #[cfg(feature = "ron_serde_fmt")]
    Ron,
    /// Render output as pretty-printed RON.
    #[cfg(feature = "ron_serde_fmt")]
    RonPretty,
}

#[cfg(feature = "structural_renderer")]
impl std::str::FromStr for StructuralRendererSetting {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match just_fmt::kebab_case!(s).as_str() {
            "disable" => Ok(Self::Disable),
            #[cfg(feature = "json_serde_fmt")]
            "json" => Ok(Self::Json),
            #[cfg(feature = "json_serde_fmt")]
            "json-pretty" => Ok(Self::JsonPretty),
            #[cfg(feature = "yaml_serde_fmt")]
            "yaml" => Ok(Self::Yaml),
            #[cfg(feature = "toml_serde_fmt")]
            "toml" => Ok(Self::Toml),
            #[cfg(feature = "ron_serde_fmt")]
            "ron" => Ok(Self::Ron),
            #[cfg(feature = "ron_serde_fmt")]
            "ron-pretty" => Ok(Self::RonPretty),
            _ => Err(format!("Invalid renderer: '{s}'")),
        }
    }
}

#[cfg(feature = "structural_renderer")]
impl From<&str> for StructuralRendererSetting {
    fn from(s: &str) -> Self {
        s.parse().unwrap_or(Self::Disable)
    }
}

#[cfg(feature = "structural_renderer")]
impl From<String> for StructuralRendererSetting {
    fn from(s: String) -> Self {
        s.as_str().into()
    }
}

#[cfg(feature = "structural_renderer")]
impl std::fmt::Display for StructuralRendererSetting {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Disable => write!(f, "disable"),
            #[cfg(feature = "json_serde_fmt")]
            Self::Json => write!(f, "json"),
            #[cfg(feature = "json_serde_fmt")]
            Self::JsonPretty => write!(f, "json-pretty"),
            #[cfg(feature = "yaml_serde_fmt")]
            Self::Yaml => write!(f, "yaml"),
            #[cfg(feature = "toml_serde_fmt")]
            Self::Toml => write!(f, "toml"),
            #[cfg(feature = "ron_serde_fmt")]
            Self::Ron => write!(f, "ron"),
            #[cfg(feature = "ron_serde_fmt")]
            Self::RonPretty => write!(f, "ron-pretty"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn program_stdout_setting_default() {
        let s = ProgramStdoutSetting::default();
        assert_eq!(s.error_output, ErrorOutput::Show);
        assert_eq!(s.render_output, RenderOutput::Show);
        assert_eq!(s.silence_panic, PanicSilence::Show);
        assert_eq!(s.verbosity, Verbosity::Normal);
        assert_eq!(s.color, ColorOutput::Enabled);
        assert_eq!(s.progress, ProgressOutput::Enabled);
    }

    #[test]
    fn program_user_context_default() {
        let ctx = ProgramUserContext::default();
        assert!(!ctx.help);
        assert!(ctx.run_hook);
        assert_eq!(ctx.confirmation, ConfirmationMode::Confirm);
        assert_eq!(ctx.execution, ExecutionMode::Normal);
        assert_eq!(ctx.interaction, InteractionMode::NonInteractive);
        assert_eq!(ctx.yes_assumption, YesAssumption::None);
    }

    #[cfg(feature = "structural_renderer")]
    mod structural_renderer_tests {
        use super::*;

        #[test]
        fn from_str_disable() {
            let val: StructuralRendererSetting = "disable".parse().unwrap();
            assert!(matches!(val, StructuralRendererSetting::Disable));
        }

        #[cfg(feature = "json_serde_fmt")]
        #[test]
        fn from_str_json() {
            let val: StructuralRendererSetting = "json".parse().unwrap();
            assert!(matches!(val, StructuralRendererSetting::Json));
        }

        #[cfg(feature = "json_serde_fmt")]
        #[test]
        fn from_str_json_pretty() {
            let val: StructuralRendererSetting = "json-pretty".parse().unwrap();
            assert!(matches!(val, StructuralRendererSetting::JsonPretty));
        }

        #[cfg(feature = "yaml_serde_fmt")]
        #[test]
        fn from_str_yaml() {
            let val: StructuralRendererSetting = "yaml".parse().unwrap();
            assert!(matches!(val, StructuralRendererSetting::Yaml));
        }

        #[cfg(feature = "toml_serde_fmt")]
        #[test]
        fn from_str_toml() {
            let val: StructuralRendererSetting = "toml".parse().unwrap();
            assert!(matches!(val, StructuralRendererSetting::Toml));
        }

        #[cfg(feature = "ron_serde_fmt")]
        #[test]
        fn from_str_ron() {
            let val: StructuralRendererSetting = "ron".parse().unwrap();
            assert!(matches!(val, StructuralRendererSetting::Ron));
        }

        #[cfg(feature = "ron_serde_fmt")]
        #[test]
        fn from_str_ron_pretty() {
            let val: StructuralRendererSetting = "ron-pretty".parse().unwrap();
            assert!(matches!(val, StructuralRendererSetting::RonPretty));
        }

        #[test]
        fn from_str_invalid() {
            let res: Result<StructuralRendererSetting, String> = "invalid".parse();
            assert!(res.is_err());
        }

        #[test]
        fn from_str_kebab_case() {
            let val: StructuralRendererSetting = "JsonPretty".parse().unwrap();
            assert!(matches!(val, StructuralRendererSetting::JsonPretty));
        }

        #[test]
        fn from_str_case_insensitive() {
            let val: StructuralRendererSetting = "JSON".parse().unwrap();
            assert!(matches!(val, StructuralRendererSetting::Json));
        }

        #[test]
        fn from_and_str() {
            let val = <StructuralRendererSetting as From<&str>>::from("json");
            assert!(
                matches!(val, StructuralRendererSetting::Disable)
                    || matches!(val, StructuralRendererSetting::Json)
            );

            let val = <StructuralRendererSetting as From<&str>>::from("invalid");
            assert!(matches!(val, StructuralRendererSetting::Disable));
        }

        #[test]
        fn from_string() {
            let val = <StructuralRendererSetting as From<String>>::from("json-pretty".to_string());
            assert!(
                matches!(val, StructuralRendererSetting::Disable)
                    || matches!(val, StructuralRendererSetting::JsonPretty)
            );
        }

        #[test]
        fn display_disable() {
            assert_eq!(StructuralRendererSetting::Disable.to_string(), "disable");
        }

        #[cfg(feature = "json_serde_fmt")]
        #[test]
        fn display_json() {
            assert_eq!(StructuralRendererSetting::Json.to_string(), "json");
        }

        #[cfg(feature = "json_serde_fmt")]
        #[test]
        fn display_json_pretty() {
            assert_eq!(
                StructuralRendererSetting::JsonPretty.to_string(),
                "json-pretty"
            );
        }
    }
}
