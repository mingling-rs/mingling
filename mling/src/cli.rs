use crate::{
    CMDCompletion, ErrorDispatcherNotFound, Next, PackageManagerSetup, ProjectManagerSetup,
    ThisProgram,
    display::markdown,
    eformat_cargo, eprintln_cargo, hformat_cargo,
    pkg_mgr::{CMDInstall, CMDListNamespace, CMDRemoveNamespace},
    res::{ResCurrentDir, ResManifestPath},
};
use colored::Colorize;
use mingling::{
    Groupped, Program,
    hook::ProgramHook,
    macros::{chain, help, pack, program_setup, r_println, renderer},
    res::ResExitCode,
    setup::{ExitCodeSetup, GeneralRendererSetup, HelpFlagSetup, QuietFlagSetup},
};
use std::{env::current_dir, path::PathBuf, process::exit, str::FromStr};

pub fn run() {
    #[cfg(windows)]
    colored::control::set_virtual_terminal(true).unwrap();

    // Preprocess args to handle cargo-mling invocations
    let mut args: Vec<String> = std::env::args().collect();
    if args.first().is_some_and(|a| a.contains("cargo-mling")) {
        args[0] = "cargo-mling".to_string();
    }
    if args.get(1).is_some_and(|a| a == "mling") {
        args.remove(1);
    }

    // Build program with preprocessed args
    let mut program = Program::<ThisProgram>::new_with_args(args);

    // Intercept Version
    program.global_flag(["-V", "--version"], |_| {
        eprintln!(include_str!("helps/version.txt"));
        exit(0)
    });

    // Intercept Help
    program.with_hook(ProgramHook::empty().on_post_dispatch(|c| match c {
        // When dispatcher is not found
        ThisProgram::ErrorDispatcherNotFound
            // And user requests Help
            if ThisProgram::this().user_context.help => {
                // Print help
                eprintln!("{}", markdown(include_str!("helps/mling_help.txt")));
                exit(0)
            }
        _ => {}
    }));

    // Commands
    program.with_dispatcher(CMDCompletion);

    // Setups
    program.with_setup(HelpFlagSetup::new(["-h", "--help"]));
    program.with_setup(GeneralRendererSetup);
    program.with_setup(ExitCodeSetup::default());

    program.with_setup(StandardOutputSetup);
    program.with_setup(PackageManagerSetup);
    program.with_setup(ProjectManagerSetup);

    // Resources
    program.with_resource(ResCurrentDir {
        path: current_dir().unwrap(),
    });

    let manifest_path = program.pick_global_argument(["-P", "--manifest-path"]);
    program.with_resource(ResManifestPath {
        raw: manifest_path,
        resolved: None,
    });

    // Manifest Path Check
    if !program.is_completing() {
        program.with_hook(ProgramHook::empty().on_post_dispatch(|_| {
            let p = ThisProgram::this();
            p.modify_res(|manifest_path: &mut ResManifestPath| {
                manifest_path.resolved = Some(resolve_manifest_path(manifest_path.raw.clone()));
            });
        }));
    }

    // Execute
    let quiet = program.stdout_setting.quiet;
    let error_output = program.stdout_setting.error_output && !quiet;
    let render_output = program.stdout_setting.render_output && !quiet;
    let result = program.exec_without_render().unwrap();
    if !result.is_empty() {
        if result.exit_code == 0 && render_output {
            println!("{}", result.trim());
        } else if error_output {
            eprintln!("{}", result.trim());
        }
    }
    exit(result.exit_code);
}

#[program_setup]
fn standard_output_setup(program: &mut Program<ThisProgram>) {
    program.with_setup(QuietFlagSetup::new("--silence"));
    program.global_flag(["--no-error"], |program| {
        program.stdout_setting.error_output = false;
    });
    program.global_flag(["--no-result"], |program| {
        program.stdout_setting.render_output = false;
    });
    program.global_flag(["--silence", "--quiet"], |program| {
        program.stdout_setting.quiet = true;
    });
}

fn resolve_manifest_path(provided: Option<String>) -> PathBuf {
    let p = ThisProgram::this();
    let disable_error_output =
        // --no-error
        !p.stdout_setting.error_output ||
        // or --quiet/--silence
        p.stdout_setting.quiet;

    if let Some(path) = provided {
        let p = PathBuf::from_str(&path).unwrap();
        if p.is_dir() {
            let candidate = p.join("Cargo.toml");
            if candidate.exists() {
                return candidate;
            }
            if !disable_error_output {
                eprintln_cargo!("`{}` is not a crate root", p.display());
            }
            exit(1);
        }
        return p;
    }
    // Walk up from current directory to find nearest Cargo.toml
    let mut dir = current_dir().unwrap();
    loop {
        let candidate = dir.join("Cargo.toml");
        if candidate.exists() {
            return candidate;
        }
        if !dir.pop() {
            // Reached filesystem root without finding Cargo.toml
            if !disable_error_output {
                eprintln_cargo!("`{}` is not a crate root", current_dir().unwrap().display());
            }
            exit(1);
        }
    }
}

pack!(ResultMlingHelp = ());
pack!(ResultUnknownCommand = String);

#[chain]
pub fn handle_error_dispatcher_not_found(err: ErrorDispatcherNotFound) -> Next {
    if err.is_empty() {
        ResultMlingHelp::default().to_render()
    } else {
        ResultUnknownCommand::new(err.join(" ")).to_render()
    }
}

#[renderer]
pub fn render_mling_help(_prev: ResultMlingHelp, ec: &mut ResExitCode) {
    r_println!("{}", markdown(include_str!("helps/mling_help.txt")));
    ec.exit_code = 0;
}

#[renderer]
pub fn render_unknown_command(prev: ResultUnknownCommand, ec: &mut ResExitCode) {
    r_println!(
        "{}",
        eformat_cargo!("no such command: `{}`", prev.bright_yellow().bold())
    );
    r_println!();
    r_println!(
        "{}",
        hformat_cargo!("view all commands with `cargo help mling`")
    );
    ec.exit_code = 101;
}
