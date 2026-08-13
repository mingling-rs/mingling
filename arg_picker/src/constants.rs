// Doc Not Optimize
use std::marker::PhantomData;

use crate::{PickerArg, PickerArgs};

/// Remaining positional arguments (anything not consumed as an option).
/// - `full`: `[]` (empty — not triggered by any `--` prefix).
/// - `short`: (none)
/// - `positional`: `false` (this is a meta‑argument that collects everything left).
///   This constant is used internally to access any leftover arguments after
///   all defined flags/options have been processed.
pub const REMAINS: PickerArg<PickerArgs> = PickerArg::<PickerArgs> {
    full: &[],
    short: None,
    positional: false,
    internal_type: PhantomData,
};
