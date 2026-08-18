use just_progress::{
    progress::{self},
    renderer::ProgressSimpleRenderer,
};
use mingling::{
    hook::ProgramHook,
    setup::{
        ConfirmSetup, DirectoryEnvironmentSetup, ExitCodeSetup,
        picker::{ConfirmFlagSetup, HelpFlagSetup, QuietFlagSetup},
    },
};
use tokio::join;

use mingling_ci_system::ThisProgram;
use mingling_ci_system::res::*;

#[tokio::main]
async fn main() {
    let center = progress::init();
    let renderer = ProgressSimpleRenderer::new().with_subprogress(true);
    let bind = progress::bind(center, move |name, state| renderer.update(name, state));

    let (_, exit_code) = join!(bind, mingling_ci_begin());
    std::process::exit(exit_code);
}

async fn mingling_ci_begin() -> i32 {
    let mut program = ThisProgram::new();

    program.with_hook(ProgramHook::empty().on_finish(|_| progress::close()));

    // Plugins
    program.with_setup(ExitCodeSetup::default());
    program.with_setup(DirectoryEnvironmentSetup::default());

    program.with_setup(HelpFlagSetup::default());
    program.with_setup(ConfirmFlagSetup::default());
    program.with_setup(QuietFlagSetup::default());

    program.with_setup(ConfirmSetup);

    // CI Plugins
    program.with_setup(ManifestsSetup);
    program.with_setup(FeaturesSetup);
    program.with_setup(ReportSetup);

    program.exec().await
}
