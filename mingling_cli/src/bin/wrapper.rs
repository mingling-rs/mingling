use std::env;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::{self, Command};

fn main() {
    exec();
}

fn exec() {
    fn target_exe_path() -> PathBuf {
        let mut path = env::current_exe().expect("Fail to read current_exe");
        path.pop();

        let target_name = if cfg!(target_os = "windows") {
            "mingling-cli.exe"
        } else {
            "mingling-cli"
        };

        path.push(target_name);
        path
    }

    let mut args: Vec<OsString> = env::args_os().collect();
    let pass_args = if args.len() > 1 {
        args.split_off(1)
    } else {
        vec![]
    };

    let target = target_exe_path();

    let status = Command::new(&target)
        .args(&pass_args)
        .status()
        .unwrap_or_else(|_| {
            process::exit(1);
        });

    match status.code() {
        Some(code) => process::exit(code),
        None => {
            process::exit(1);
        }
    }
}
