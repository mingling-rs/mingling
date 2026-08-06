use crate::metadata::{
    MinglingMetadataSetup,
    setup::{
        ARG_ALL_FEATURES, ARG_FEATURES, ARG_MANIFEST_PATH, ARG_MESSAGE_FORMAT,
        ARG_NO_DEFAULT_FEATURES, ARG_NO_DEPS,
    },
};
use mingling::{
    ShellContext, Suggest,
    consts::{HELP_FLAG, JSON_FLAG, JSON_PRETTY_FLAG},
    macros::{completion, gen_program, help, suggest},
    setup::{
        ExitCodeSetup,
        picker::{HelpFlagSetup, StructuralRendererSetup},
    },
};

pub mod diagnostic;
pub mod errors;
pub mod linter;
pub mod lints;
pub mod message;
pub mod metadata;
pub mod pkg_mgr;
pub mod proj_mgr;

#[tokio::main]
async fn main() {
    let mut program = ThisProgram::new();

    // Setups
    program.with_setup(StructuralRendererSetup);
    program.with_setup(HelpFlagSetup::default());
    program.with_setup(ExitCodeSetup::default());

    program.with_setup(MinglingMetadataSetup);

    // Exec
    program.exec_and_exit().await;
}

#[help]
pub fn help_global(_: EntryFallback) -> String {
    include_str!("../help/help.txt").to_string()
}

#[completion(EntryFallback)]
pub fn complete_global(_ctx: &ShellContext) -> Suggest {
    suggest! {
        HELP_FLAG: "Show help messages",
        ARG_FEATURES.clone(): "List of features to enable",
        ARG_MANIFEST_PATH.clone(): "Custom path to Cargo.toml",
        ARG_MESSAGE_FORMAT.clone(): "Output message format",
        ARG_ALL_FEATURES: "Enable all features",
        ARG_NO_DEFAULT_FEATURES: "Disable default features",
        ARG_NO_DEPS: "Do not include dependencies in metadata",
        JSON_FLAG: "Render results in JSON format",
        JSON_PRETTY_FLAG: "Render results in pretty JSON format"
    }
}

gen_program!();
