use arg_picker::{PickerArg, value::Flag};

/// Help flag: display usage information.
/// - `full`: `["help"]`
/// - `short`: `'h'`
pub const HELP_FLAG: PickerArg<Flag> = PickerArg::<Flag>::new(&["help"], Some('h'), false);

/// Quiet flag: suppress output.
/// - `full`: `["quiet"]`
/// - `short`: `'q'`
pub const QUIET_FLAG: PickerArg<Flag> = PickerArg::<Flag>::new(&["quiet"], Some('q'), false);

/// Confirm flag: require user confirmation before proceeding.
/// - `full`: `["confirm"]`
/// - `short`: `'C'`
pub const CONFIRM_FLAG: PickerArg<Flag> = PickerArg::<Flag>::new(&["confirm"], Some('C'), false);

/// Version flag: display version information.
/// - `full`: `["version"]`
/// - `short`: `'v'`
pub const VERSION_FLAG: PickerArg<Flag> = PickerArg::<Flag>::new(&["version"], Some('v'), false);

/// Verbose flag: enable detailed output.
/// - `full`: `["verbose"]`
/// - `short`: `'V'`
pub const VERBOSE_FLAG: PickerArg<Flag> = PickerArg::<Flag>::new(&["verbose"], Some('V'), false);

/// Renderer flag: explicitly specify the Structural Renderer argument.
/// - `full`: `["renderer"]`
/// - `short`: (none)
#[cfg(feature = "structural_renderer")]
pub const RENDERER_ARG: PickerArg<String> = PickerArg::<String>::new(&["renderer"], None, false);

/// JSON flag: enable JSON output format.
/// - `full`: `["json"]`
/// - `short`: (none)
///   Available only when the `json_serde_fmt` feature is enabled.
#[cfg(feature = "json_serde_fmt")]
pub const JSON_FLAG: PickerArg<Flag> = PickerArg::<Flag>::new(&["json"], None, false);

/// JSON pretty flag: enable pretty-printed JSON output format.
/// - `full`: `["json_pretty"]`
/// - `short`: (none)
///   Available only when the `json_serde_fmt` feature is enabled.
#[cfg(feature = "json_serde_fmt")]
pub const JSON_PRETTY_FLAG: PickerArg<Flag> = PickerArg::<Flag>::new(&["json_pretty"], None, false);

/// YAML flag: enable YAML output format.
/// - `full`: `["yaml"]`
/// - `short`: (none)
///   Available only when the `yaml_serde_fmt` feature is enabled.
#[cfg(feature = "yaml_serde_fmt")]
pub const YAML_FLAG: PickerArg<Flag> = PickerArg::<Flag>::new(&["yaml"], None, false);

/// TOML flag: enable TOML output format.
/// - `full`: `["toml"]`
/// - `short`: (none)
///   Available only when the `toml_serde_fmt` feature is enabled.
#[cfg(feature = "toml_serde_fmt")]
pub const TOML_FLAG: PickerArg<Flag> = PickerArg::<Flag>::new(&["toml"], None, false);

/// RON flag: enable RON output format.
/// - `full`: `["ron"]`
/// - `short`: (none)
///   Available only when the `ron_serde_fmt` feature is enabled.
#[cfg(feature = "ron_serde_fmt")]
pub const RON_FLAG: PickerArg<Flag> = PickerArg::<Flag>::new(&["ron"], None, false);

/// RON pretty flag: enable pretty-printed RON output format.
/// - `full`: `["ron_pretty"]`
/// - `short`: (none)
///   Available only when the `ron_serde_fmt` feature is enabled.
#[cfg(feature = "ron_serde_fmt")]
pub const RON_PRETTY_FLAG: PickerArg<Flag> = PickerArg::<Flag>::new(&["ron_pretty"], None, false);
