use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::{self, Command};

use flate2::read::GzDecoder;
use tar::Archive;

fn main() {
    apply_update_if_present();
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

/// The update package staged by `mling update` at `{data_dir}/mingling/update.tar.gz`.
fn update_package_path() -> Option<PathBuf> {
    dirs::data_dir().map(|data_dir| data_dir.join("mingling").join("update.tar.gz"))
}

/// Apply a staged update, if any: unpack it over the installation directory
/// (never replacing the running wrapper itself), then remove the staged file.
/// This runs before forwarding to `mingling-cli`, so the new version is loaded
/// by this very invocation.
fn apply_update_if_present() {
    let Some(update_path) = update_package_path() else {
        return;
    };
    if !update_path.is_file() {
        return;
    }
    let Ok(current_exe) = env::current_exe() else {
        return;
    };

    // The package mirrors the install layout: the wrapper lives at
    // `<root>/bin/mling` and the archive root maps onto `<root>`, so entries
    // like `bin/mingling-cli` replace the files next to this wrapper.
    let Some(exe_dir) = current_exe.parent() else {
        return;
    };
    let Some(install_root) = exe_dir.parent() else {
        return;
    };

    match unpack_update(&update_path, &current_exe, install_root) {
        Ok(()) => {
            let _ = fs::remove_file(update_path);
        }
        Err(e) => eprintln!("mling: failed to apply update: {e}"),
    }
}

/// Extract `update.tar.gz` into `install_root`, skipping the running wrapper.
fn unpack_update(
    update_path: &Path,
    current_exe: &Path,
    install_root: &Path,
) -> std::io::Result<()> {
    let file = fs::File::open(update_path)?;
    let mut archive = Archive::new(GzDecoder::new(file));

    for entry in archive.entries()? {
        let mut entry = entry?;
        let entry_type = entry.header().entry_type();
        let rel_path = sanitize_relative_path(&entry.path()?);
        if rel_path.as_os_str().is_empty() {
            continue;
        }
        let dest = install_root.join(rel_path);
        // The running executable cannot (and must not) replace itself.
        if dest == current_exe {
            continue;
        }
        if entry_type.is_dir() {
            fs::create_dir_all(dest)?;
            continue;
        }
        if !entry_type.is_file() {
            continue;
        }
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        entry.unpack(dest)?;
    }
    Ok(())
}

/// Keep only normal path components so entries cannot escape `install_root`.
fn sanitize_relative_path(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        if let Component::Normal(part) = component {
            out.push(part);
        }
    }
    out
}
