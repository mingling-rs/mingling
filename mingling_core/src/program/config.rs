/// Program stdout settings
#[derive(Debug, Clone)]
pub struct ProgramStdoutSetting {
    /// Output error messages
    pub error_output: bool,

    /// Render results and output
    pub render_output: bool,

    /// Silence panic messages
    pub silence_panic: bool,

    /// Verbose output: provide detailed information
    ///
    /// **NOTE**: Convention only, not a configuration
    pub verbose: bool,

    /// Quiet mode: suppress status messages, show only errors and results
    ///
    /// **NOTE**: Convention only, not a configuration
    pub quiet: bool,

    /// Debug mode: output internal state and detailed diagnostics
    ///
    /// **NOTE**: Convention only, not a configuration
    pub debug: bool,

    /// Enable colored output
    ///
    /// **NOTE**: Convention only, not a configuration
    pub color: bool,

    /// Show progress indicators (e.g. progress bars, spinners)
    ///
    /// Automatically disabled when stdout is not a tty.
    ///
    /// **NOTE**: Convention only, not a configuration
    pub progress: bool,

    #[cfg(feature = "clap")]
    /// Behavior when Clap Dispatcher outputs help information
    pub clap_help_print_behaviour: ClapHelpPrintBehaviour,
}

#[cfg(feature = "clap")]
#[derive(Debug, Default, Clone)]
pub enum ClapHelpPrintBehaviour {
    /// Write to RenderResult
    WriteToRenderResult,

    /// Print directly
    #[default]
    PrintDirectly,
}

impl Default for ProgramStdoutSetting {
    fn default() -> Self {
        ProgramStdoutSetting {
            error_output: true,
            render_output: true,
            silence_panic: false,
            verbose: false,
            quiet: false,
            debug: false,
            color: true,
            progress: true,
            #[cfg(feature = "clap")]
            clap_help_print_behaviour: ClapHelpPrintBehaviour::default(),
        }
    }
}

/// Program user context
#[derive(Debug, Clone)]
pub struct ProgramUserContext {
    /// View help information instead of running the command
    pub help: bool,

    /// Execute hooks during the program lifecycle
    pub run_hook: bool,

    /// Skip user confirmation step
    ///
    /// **NOTE**: Convention only, not a configuration
    pub confirm: bool,

    /// Dry-run mode: simulate actions without making changes
    ///
    /// **NOTE**: Convention only, not a configuration
    pub dry_run: bool,

    /// Force execution, skipping safety checks
    ///
    /// **NOTE**: Convention only, not a configuration
    pub force: bool,

    /// Whether the program is running in an interactive terminal (has a tty)
    ///
    /// **NOTE**: Convention only, not a configuration
    pub interactive: bool,

    /// Assume "yes" for all confirmation prompts
    ///
    /// **NOTE**: Convention only, not a configuration
    pub assume_yes: bool,
}

impl Default for ProgramUserContext {
    fn default() -> Self {
        Self {
            help: false,
            run_hook: true,
            confirm: false,
            dry_run: false,
            force: false,
            interactive: false,
            assume_yes: false,
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
            "disable" => Ok(StructuralRendererSetting::Disable),
            #[cfg(feature = "json_serde_fmt")]
            "json" => Ok(StructuralRendererSetting::Json),
            #[cfg(feature = "json_serde_fmt")]
            "json-pretty" => Ok(StructuralRendererSetting::JsonPretty),
            #[cfg(feature = "yaml_serde_fmt")]
            "yaml" => Ok(StructuralRendererSetting::Yaml),
            #[cfg(feature = "toml_serde_fmt")]
            "toml" => Ok(StructuralRendererSetting::Toml),
            #[cfg(feature = "ron_serde_fmt")]
            "ron" => Ok(StructuralRendererSetting::Ron),
            #[cfg(feature = "ron_serde_fmt")]
            "ron-pretty" => Ok(StructuralRendererSetting::RonPretty),
            _ => Err(format!("Invalid renderer: '{s}'")),
        }
    }
}

#[cfg(feature = "structural_renderer")]
impl From<&str> for StructuralRendererSetting {
    fn from(s: &str) -> Self {
        s.parse().unwrap_or(StructuralRendererSetting::Disable)
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
            StructuralRendererSetting::Disable => write!(f, "disable"),
            #[cfg(feature = "json_serde_fmt")]
            StructuralRendererSetting::Json => write!(f, "json"),
            #[cfg(feature = "json_serde_fmt")]
            StructuralRendererSetting::JsonPretty => write!(f, "json-pretty"),
            #[cfg(feature = "yaml_serde_fmt")]
            StructuralRendererSetting::Yaml => write!(f, "yaml"),
            #[cfg(feature = "toml_serde_fmt")]
            StructuralRendererSetting::Toml => write!(f, "toml"),
            #[cfg(feature = "ron_serde_fmt")]
            StructuralRendererSetting::Ron => write!(f, "ron"),
            #[cfg(feature = "ron_serde_fmt")]
            StructuralRendererSetting::RonPretty => write!(f, "ron-pretty"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn program_stdout_setting_default() {
        let s = ProgramStdoutSetting::default();
        assert!(s.error_output);
        assert!(s.render_output);
        assert!(!s.silence_panic);
        assert!(!s.verbose);
        assert!(!s.quiet);
        assert!(!s.debug);
        assert!(s.color);
        assert!(s.progress);
    }

    #[test]
    fn program_user_context_default() {
        let ctx = ProgramUserContext::default();
        assert!(!ctx.help);
        assert!(ctx.run_hook);
        assert!(!ctx.confirm);
        assert!(!ctx.dry_run);
        assert!(!ctx.force);
        assert!(!ctx.interactive);
        assert!(!ctx.assume_yes);
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
