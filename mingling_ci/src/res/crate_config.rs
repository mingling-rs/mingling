use std::collections::HashMap;
use std::path::Path;

use mingling::{Program, macros::program_setup};

use crate::ThisProgram;
use crate::res::{Manifests, ResFeatureList};

/// Per-crate CI overrides from `mingling-ci.toml` (optional, crate root).
///
/// Currently only `[test] command` is read; `clippy.command` / `build.command`
/// will follow the same shape.
#[derive(Default, Clone)]
pub struct ResCrateConfig {
    /// Package name -> test command argv (with `<<<features>>>` expanded).
    test_commands: HashMap<String, Vec<String>>,
}

impl ResCrateConfig {
    /// The configured `[test] command` for a package, if any.
    #[must_use]
    pub fn test_command(&self, package: &str) -> Option<&[String]> {
        self.test_commands.get(package).map(Vec::as_slice)
    }
}

#[program_setup]
pub fn crate_config_setup(p: &mut Program<ThisProgram>) {
    let features = p
        .res::<ResFeatureList>()
        .map(|f| f.list.clone())
        .unwrap_or_default();
    let joined_features = features.join(",");

    let Some(manifests) = p.res::<Manifests>() else {
        return;
    };

    let mut test_commands = HashMap::new();
    for (name, manifest_path) in &manifests.package_dirs {
        let config_path = manifest_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("mingling-ci.toml");

        let Ok(content) = std::fs::read_to_string(&config_path) else {
            continue;
        };

        let Ok(table) = content.parse::<toml::Value>() else {
            continue;
        };

        let Some(command) = table
            .get("test")
            .and_then(|t| t.get("command"))
            .and_then(|c| c.as_array())
        else {
            continue;
        };

        let argv: Vec<String> = command
            .iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect();

        if argv.is_empty() {
            continue;
        }

        let argv = argv
            .into_iter()
            .map(|arg| arg.replace("<<<features>>>", &joined_features))
            .collect();
        test_commands.insert(name.clone(), argv);
    }

    p.with_resource(ResCrateConfig { test_commands });
}
