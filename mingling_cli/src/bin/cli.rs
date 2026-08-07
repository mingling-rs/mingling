use mingling::{
    ShellContext, Suggest,
    consts::HELP_FLAG,
    macros::{completion, help, suggest},
    setup::{ExitCodeSetup, picker::HelpFlagSetup},
};
use mingling_cli::{
    EntryFallback, ThisProgram,
    linter::registry::LintRegistrySetup,
    metadata::{
        MinglingMetadataSetup,
        setup::{
            ARG_ALL_FEATURES, ARG_FEATURES, ARG_MANIFEST_PATH, ARG_MESSAGE_FORMAT,
            ARG_NO_DEFAULT_FEATURES, ARG_NO_DEPS,
        },
    },
    utils::display::ColorCode,
};

#[tokio::main]
async fn main() {
    let mut program = ThisProgram::new();

    // Setups
    program.with_setup(HelpFlagSetup::default());
    program.with_setup(ExitCodeSetup::default());

    program.with_setup(MinglingMetadataSetup);
    program.with_setup(LintRegistrySetup);

    // Exec
    program.exec_and_exit().await;
}

#[help]
pub fn help_global(_: EntryFallback) -> String {
    include_str!("../../help/help.txt").parse_color_code()
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
    }
}
