use std::ffi::OsString;
use std::path::Path;

use mingling::{
    Grouped, Routable,
    macros::{buffer, command, renderer},
    res::ResExitCode,
};

use crate::Next;
use crate::res::Manifests;
use crate::task::run::{location, run_parallel_checks};

#[command(node = "build-check")]
pub async fn build_check(manifests: &Manifests) -> Next {
    let tasks = manifests
        .package_dirs
        .iter()
        .map(|(name, path)| (name.clone(), location(path), build_args(path)))
        .collect();
    let fail_count = run_parallel_checks("Build-Check", "Building", tasks).await;
    ResultBuildCheck { fail_count }.to_chain()
}

/// `cargo build --manifest-path <path>`
fn build_args(path: &Path) -> Vec<OsString> {
    vec![
        "cargo".into(),
        "build".into(),
        "--manifest-path".into(),
        path.as_os_str().to_os_string(),
    ]
}

/// Number of packages that failed to build.
#[derive(Grouped)]
pub struct ResultBuildCheck {
    pub fail_count: usize,
}

/// Silently sets a non-zero exit code when any build failed.
#[renderer(buffer)]
pub fn render_build_check(r: ResultBuildCheck, exit_code: &mut ResExitCode) {
    if r.fail_count > 0 {
        exit_code.exit_code = 1;
    }
}
