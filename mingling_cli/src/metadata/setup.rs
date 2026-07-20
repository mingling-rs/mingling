use std::{env::current_dir, path::PathBuf};

use cargo_metadata::{CargoOpt, MetadataCommand};
use mingling::{
    LazyInit, Program,
    consts::REMAINS,
    macros::program_setup,
    picker::{IntoPicker, value::Flag},
    prelude::*,
    res::ResCurrentDir,
};

/// Name of Cargo manifest file
pub const CARGO_TOML: &str = "Cargo.toml";

use cargo_metadata::Metadata;
use serde::Serialize;

#[derive(Default, Clone, Serialize)]
pub struct ResMetadata {
    data: Option<Metadata>,
}

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
    let args = program.take_args();
    let (
        feature_args,
        manifest_path,
        message_format,
        enable_all_features,
        no_default_features,
        no_deps,
        args,
    ) = args
        .pick(&arg![features: Vec<String>])
        .pick(&arg![manifest_path: Option<String>])
        .pick_or(&arg![message_format: String], || "disable".to_string())
        .pick(&arg![all_features: Flag])
        .pick(&arg![no_default_features: Flag])
        .pick(&arg![no_deps: Flag])
        .pick(&REMAINS)
        .unwrap();
    program.replace_args(args.into());

    // Bind format setting to StructuralRenderer
    program.structural_renderer_name = message_format.into();

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
