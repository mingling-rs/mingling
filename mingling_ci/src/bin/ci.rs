use mingling::setup::{
    ConfirmSetup, DirectoryEnvironmentSetup, ExitCodeSetup,
    picker::{ConfirmFlagSetup, HelpFlagSetup, QuietFlagSetup},
};

use mingling_ci_system::ThisProgram;
use mingling_ci_system::res::*;

#[tokio::main]
async fn main() {
    let mut program = ThisProgram::new();

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
    program.with_setup(CrateConfigSetup);
    program.with_setup(ReportSetup);

    program.exec_and_exit().await;
}
