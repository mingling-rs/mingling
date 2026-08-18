use std::ffi::OsString;
use std::path::Path;

use mingling::{
    Grouped, Routable,
    macros::{buffer, command, renderer},
    res::ResExitCode,
};

use crate::Next;
use crate::res::{Manifests, ResCrateConfig};
use crate::task::run::run_parallel_checks;

#[command(node = "test-all")]
pub async fn test_all(manifests: &Manifests, config: &ResCrateConfig) -> Next {
    let tasks = manifests
        .package_dirs
        .iter()
        .map(|(name, path)| {
            let args = config.test_command(name).map_or_else(
                || test_args(path),
                |cmd| cmd.iter().map(|s| OsString::from(s.as_str())).collect(),
            );
            (name.clone(), args)
        })
        .collect();
    let fail_count = run_parallel_checks("Test-All", "Testing", tasks).await;
    ResultTestAll { fail_count }.to_chain()
}

/// Default: `cargo test --manifest-path <path>` (crates without a
/// `mingling-ci.toml` override).
fn test_args(path: &Path) -> Vec<OsString> {
    vec![
        "cargo".into(),
        "test".into(),
        "--manifest-path".into(),
        path.as_os_str().to_os_string(),
    ]
}

/// Number of packages that failed tests.
#[derive(Grouped)]
pub struct ResultTestAll {
    pub fail_count: usize,
}

/// Silently sets a non-zero exit code when any test failed.
#[renderer(buffer)]
pub fn render_test_all(r: ResultTestAll, exit_code: &mut ResExitCode) {
    if r.fail_count > 0 {
        exit_code.exit_code = 1;
    }
}
