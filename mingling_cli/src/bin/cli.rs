use mingling::setup::{DirectoryEnvironmentSetup, ExitCodeSetup, picker::HelpFlagSetup};
use mingling_cli::{
    ThisProgram, config::MlingConfigSetup, linter::registry::LintRegistrySetup,
    metadata::MinglingMetadataSetup, pkg_mgr::PackageManagerSetup,
};

#[tokio::main]
async fn main() {
    let mut program = ThisProgram::new();

    // Setups
    program.with_setup(HelpFlagSetup::default());
    program.with_setup(ExitCodeSetup);
    program.with_setup(DirectoryEnvironmentSetup);

    program.with_setup(MinglingMetadataSetup);
    program.with_setup(MlingConfigSetup);
    program.with_setup(LintRegistrySetup);
    program.with_setup(PackageManagerSetup);

    // Exec
    program.exec_and_exit().await;
}
