use std::marker::PhantomData;

use arg_picker::{PickerArg, value::Flag};

/// Help flag: display usage information.
/// - `full`: `["help"]`
/// - `short`: `'h'`
pub const HELP_FLAG: PickerArg<Flag> = PickerArg::<Flag> {
    full: &["help"],
    short: Some('h'),
    positional: false,
    internal_type: PhantomData,
};

/// Quiet flag: suppress output.
/// - `full`: `["quiet"]`
/// - `short`: `'q'`
pub const QUIET_FLAG: PickerArg<Flag> = PickerArg::<Flag> {
    full: &["quiet"],
    short: Some('q'),
    positional: false,
    internal_type: PhantomData,
};

/// Confirm flag: require user confirmation before proceeding.
/// - `full`: `["confirm"]`
/// - `short`: `'C'`
pub const CONFIRM_FLAG: PickerArg<Flag> = PickerArg::<Flag> {
    full: &["confirm"],
    short: Some('C'),
    positional: false,
    internal_type: PhantomData,
};

/// JSON flag: enable JSON output format.
/// - `full`: `["json"]`
/// - `short`: (none)
///   Available only when the `json_serde_fmt` feature is enabled.
#[cfg(feature = "json_serde_fmt")]
pub const JSON_FLAG: PickerArg<Flag> = PickerArg::<Flag> {
    full: &["json"],
    short: None,
    positional: false,
    internal_type: PhantomData,
};

/// JSON pretty flag: enable pretty-printed JSON output format.
/// - `full`: `["json_pretty"]`
/// - `short`: (none)
///   Available only when the `json_serde_fmt` feature is enabled.
#[cfg(feature = "json_serde_fmt")]
pub const JSON_PRETTY_FLAG: PickerArg<Flag> = PickerArg::<Flag> {
    full: &["json_pretty"],
    short: None,
    positional: false,
    internal_type: PhantomData,
};

/// YAML flag: enable YAML output format.
/// - `full`: `["yaml"]`
/// - `short`: (none)
///   Available only when the `yaml_serde_fmt` feature is enabled.
#[cfg(feature = "yaml_serde_fmt")]
pub const YAML_FLAG: PickerArg<Flag> = PickerArg::<Flag> {
    full: &["yaml"],
    short: None,
    positional: false,
    internal_type: PhantomData,
};

/// TOML flag: enable TOML output format.
/// - `full`: `["toml"]`
/// - `short`: (none)
///   Available only when the `toml_serde_fmt` feature is enabled.
#[cfg(feature = "toml_serde_fmt")]
pub const TOML_FLAG: PickerArg<Flag> = PickerArg::<Flag> {
    full: &["toml"],
    short: None,
    positional: false,
    internal_type: PhantomData,
};

/// RON flag: enable RON output format.
/// - `full`: `["ron"]`
/// - `short`: (none)
///   Available only when the `ron_serde_fmt` feature is enabled.
#[cfg(feature = "ron_serde_fmt")]
pub const RON_FLAG: PickerArg<Flag> = PickerArg::<Flag> {
    full: &["ron"],
    short: None,
    positional: false,
    internal_type: PhantomData,
};

/// RON pretty flag: enable pretty-printed RON output format.
/// - `full`: `["ron_pretty"]`
/// - `short`: (none)
///   Available only when the `ron_serde_fmt` feature is enabled.
#[cfg(feature = "ron_serde_fmt")]
pub const RON_PRETTY_FLAG: PickerArg<Flag> = PickerArg::<Flag> {
    full: &["ron_pretty"],
    short: None,
    positional: false,
    internal_type: PhantomData,
};
