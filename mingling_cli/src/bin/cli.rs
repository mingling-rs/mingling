use mingling::setup::{ExitCodeSetup, picker::HelpFlagSetup};
use mingling_cli::{
    ThisProgram, linter::registry::LintRegistrySetup, metadata::MinglingMetadataSetup,
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
