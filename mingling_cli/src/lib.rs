use mingling::{
    RenderResult, ShellContext, Suggest,
    consts::HELP_FLAG,
    macros::{completion, gen_program, help, renderer, suggest},
};

use crate::{
    metadata::setup::{
        ARG_ALL_FEATURES, ARG_FEATURES, ARG_MANIFEST_PATH, ARG_MESSAGE_FORMAT,
        ARG_NO_DEFAULT_FEATURES, ARG_NO_DEPS,
    },
    utils::display::ColorCode,
};

pub mod config;
pub mod diagnostic;
pub mod errors;
pub mod linter;
pub mod lints;
pub mod message;
pub mod metadata;
pub mod pkg_mgr;
pub mod proj_mgr;
pub mod updater;
pub mod utils;

#[help]
pub fn help_global(_: EntryFallback) -> String {
    format!("{}\n", include_str!("../help/help.txt").parse_color_code())
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

#[renderer]
pub fn handle_fallback(args: EntryFallback) -> RenderResult {
    let mut r = RenderResult::new();
    let args = args.0;
    if !args.is_empty() {
        eprintln_cargo!(
            r,
            "{}",
            format!("Unknown command `{}`", args.join(" ")).parse_color_code()
        )
    } else {
        hprintln_cargo!(
            r,
            "{}",
            "Welcome to MinglingCLI, please use `mling -h` to see available commands"
                .parse_color_code(),
        );
    }
    r
}

gen_program!();
