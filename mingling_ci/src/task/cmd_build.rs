use std::ffi::OsString;
use std::path::Path;

use mingling::{
    Grouped, Routable,
    macros::{buffer, command, renderer},
    res::ResExitCode,
};

use crate::Next;
use crate::res::Manifests;
use crate::task::run::run_parallel_checks;

#[command(node = "build-all")]
pub async fn build_all(manifests: &Manifests) -> Next {
    let fail_count = run_parallel_checks("Build-All", "Building", build_args, manifests).await;
    ResultBuildAll { fail_count }.to_chain()
}

/// `cargo build --manifest-path <path>`
fn build_args(path: &Path) -> Vec<OsString> {
    vec![
        "build".into(),
        "--manifest-path".into(),
        path.as_os_str().to_os_string(),
    ]
}

/// Number of packages that failed to build.
#[derive(Grouped)]
pub struct ResultBuildAll {
    pub fail_count: usize,
}

/// Silently sets a non-zero exit code when any build failed.
#[renderer(buffer)]
pub fn render_build_all(r: ResultBuildAll, exit_code: &mut ResExitCode) {
    if r.fail_count > 0 {
        exit_code.exit_code = 1;
    }
}
