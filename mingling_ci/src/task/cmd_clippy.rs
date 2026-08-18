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

#[command(node = "clippy-all")]
pub async fn clippy_all(manifests: &Manifests) -> Next {
    let tasks = manifests
        .package_dirs
        .iter()
        .map(|(name, path)| (name.clone(), clippy_args(path)))
        .collect();
    let fail_count = run_parallel_checks("Clippy-All", "Clippy", tasks).await;
    ResultClippyAll { fail_count }.to_chain()
}

/// `cargo clippy --manifest-path <path> -- -D warnings`
fn clippy_args(path: &Path) -> Vec<OsString> {
    vec![
        "cargo".into(),
        "clippy".into(),
        "--manifest-path".into(),
        path.as_os_str().to_os_string(),
        "--".into(),
        "-D".into(),
        "warnings".into(),
    ]
}

/// Number of packages that failed clippy.
#[derive(Grouped)]
pub struct ResultClippyAll {
    pub fail_count: usize,
}

/// Silently sets a non-zero exit code when any clippy check failed.
#[renderer(buffer)]
pub fn render_clippy_all(r: ResultClippyAll, exit_code: &mut ResExitCode) {
    if r.fail_count > 0 {
        exit_code.exit_code = 1;
    }
}
