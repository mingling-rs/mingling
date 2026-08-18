use std::ffi::OsString;

use mingling::{
    Grouped, Routable,
    macros::{buffer, command, renderer},
    res::ResExitCode,
};

use crate::Next;
use crate::res::ResFeatureList;
use crate::task::run::run_parallel_checks;

#[command(node = "docs-build")]
pub async fn docs_build(features: &ResFeatureList) -> Next {
    let args = vec![
        OsString::from("cargo"),
        OsString::from("rustdoc"),
        OsString::from("--features"),
        OsString::from(features.list.join(",")),
        OsString::from("-p"),
        OsString::from("mingling"),
        OsString::from("--"),
        OsString::from("-D"),
        OsString::from("warnings"),
    ];
    let tasks = vec![("mingling".to_string(), "./mingling".to_string(), args)];
    let fail_count = run_parallel_checks("Docs-Build", "Docs", tasks).await;

    ResultDocsBuild { fail_count }.to_chain()
}

/// Number of failed doc builds (0 or 1).
#[derive(Grouped)]
pub struct ResultDocsBuild {
    pub fail_count: usize,
}

/// Silently sets a non-zero exit code when the doc build failed.
#[renderer(buffer)]
pub fn render_docs_build(r: ResultDocsBuild, exit_code: &mut ResExitCode) {
    if r.fail_count > 0 {
        exit_code.exit_code = 1;
    }
}
