use std::{env::current_dir, path::PathBuf};

use cargo_metadata::{CargoOpt, MetadataCommand};
use mingling::{
    LazyInit, Program,
    macros::program_setup,
    picker::{IntoPicker, PickerArg, value::Flag},
    prelude::*,
    res::ResCurrentDir,
};

/// Name of Cargo manifest file
pub const CARGO_TOML: &str = "Cargo.toml";

use cargo_metadata::Metadata;
use serde::Serialize;

/// Resource holding parsed `cargo metadata` output.
///
/// This is lazily initialized during program setup by calling `cargo metadata`
/// with the appropriate CLI flags (e.g., `--features`, `--all-features`,
/// `--no-default-features`, `--no-deps`, `--message-format`).
///
/// Access the inner [`Metadata`] via [`ResMetadata::data()`], which panics if
/// called before initialization (guaranteed not to happen in normal usage).
#[derive(Default, Clone, Serialize)]
pub struct ResMetadata {
    data: Option<Metadata>,
}

/// Resource indicating whether the output format is JSON.
///
/// Set to `true` when `--message-format json` (or similar) is passed.
/// Used by renderers to decide whether to serialize structs as JSON.
#[derive(Default, Clone, Serialize)]
pub struct ResUsingJson {
    pub using: bool,
}

/// Argument: list of features to enable
pub static ARG_FEATURES: PickerArg<Vec<String>> = arg![features: Vec<String>];

/// Argument: custom path to Cargo.toml
pub static ARG_MANIFEST_PATH: PickerArg<Option<String>> = arg![manifest_path: Option<String>];

/// Argument: output message format
pub static ARG_MESSAGE_FORMAT: PickerArg<String> = arg![message_format: String];

/// Argument: enable all features
pub static ARG_ALL_FEATURES: PickerArg<Flag> = arg![all_features: Flag];

/// Argument: disable default features
pub static ARG_NO_DEFAULT_FEATURES: PickerArg<Flag> = arg![no_default_features: Flag];

/// Argument: do not include dependencies in metadata
pub static ARG_NO_DEPS: PickerArg<Flag> = arg![no_deps: Flag];

impl ResMetadata {
    /// Returns a reference to the parsed `cargo metadata`.
    ///
    /// # Panics
    ///
    /// This function does **not** panic in practice, because `ResMetadata` is
    /// always initialized via [`LazyInit`] inside the program setup, and the
    /// initialization either succeeds (setting `data` to `Some(...)`) or fails
    /// by propagating the error from `cmd.exec().unwrap()`. Therefore, by the
    /// time this getter is called, `self.data` is guaranteed to be `Some`.
    pub fn data(&self) -> &Metadata {
        self.data.as_ref().unwrap()
    }
}

#[program_setup]
pub fn cargo_metadata_setup(program: &mut Program<crate::ThisProgram>) {
    let args = program.get_args().to_vec();

    let (
        feature_args,
        manifest_path,
        message_format,
        enable_all_features,
        no_default_features,
        no_deps,
    ) = args
        .pick(&ARG_FEATURES)
        .pick(&ARG_MANIFEST_PATH)
        .pick_or(&ARG_MESSAGE_FORMAT, || "disable".to_string())
        .pick(&ARG_ALL_FEATURES)
        .pick(&ARG_NO_DEFAULT_FEATURES)
        .pick(&ARG_NO_DEPS)
        .unwrap();

    // Is Using Json
    program.with_resource(ResUsingJson {
        using: message_format.contains("json"),
    });

    // Current Dir
    let current_dir = current_dir().unwrap();

    // Leak `feature_args` into a static slice so it can be captured by the metadata closure.
    // Since the process does not loop, this memory leak is harmless.
    let feature_args_leaked: &[String] = Box::leak(feature_args.into_boxed_slice());
    let manifest_path_clone = manifest_path.clone();
    let current_dir_clone = current_dir.clone();

    program.with_resource(ResMetadata::lazy_init(move || {
        // Paths
        let metadata_path = match manifest_path_clone {
            Some(ref path_str) => PathBuf::from(path_str),
            None => find_manifest().unwrap(),
        };

        // Cargo Metadata - bind to a longer-lived variable
        let mut cmd = MetadataCommand::new();
        cmd.manifest_path(metadata_path)
            .current_dir(&current_dir_clone);

        if *enable_all_features {
            cmd.features(CargoOpt::AllFeatures);
        }

        if *no_default_features {
            cmd.features(CargoOpt::NoDefaultFeatures);
        }

        if *no_deps {
            cmd.no_deps();
        }

        cmd.features(CargoOpt::SomeFeatures(feature_args_leaked.to_vec()));

        ResMetadata {
            data: Some(cmd.exec().unwrap()),
        }
    }));

    // Current Dir
    program.with_resource(ResCurrentDir::from(current_dir));
}

/// Find `Cargo.toml` by searching upward from `current_dir`.
fn find_manifest() -> Option<PathBuf> {
    let mut dir = current_dir().ok()?;
    loop {
        let candidate = dir.join(CARGO_TOML);
        if candidate.is_file() {
            return Some(candidate);
        }
        if !dir.pop() {
            return None;
        }
    }
}
