use arg_picker::IntoPicker;
use mingling_core::{Program, ProgramCollect, setup::ProgramSetup};

use crate::{
    setup::picker::REMAINS,
    setups::picker::{
        JSON_FLAG, JSON_PRETTY_FLAG, RON_FLAG, RON_PRETTY_FLAG, TOML_FLAG, YAML_FLAG,
    },
};

/// Sets up the structural renderer for the program:
///
/// - Adds a `--renderer` global argument to specify the renderer type
pub struct StructuralRendererSimpleSetup;

impl<C> ProgramSetup<C> for StructuralRendererSimpleSetup
where
    C: ProgramCollect<Enum = C>,
{
    fn setup(self, program: &mut Program<C>) {
        program.global_argument("--renderer", |p, renderer| {
            p.structural_renderer_name = renderer.into();
        });
    }
}

/// Sets up the structural renderer for the program:
///
/// - Adds global flags to specify the renderer type:
///   * `--json` for JSON output
///   * `--json-pretty` for pretty-printed JSON output
///   * `--yaml` for YAML output
///   * `--toml` for TOML output
///   * `--ron` for RON output
///   * `--ron-pretty` for pretty-printed RON output
pub struct StructuralRendererSetup;

impl<C> ProgramSetup<C> for StructuralRendererSetup
where
    C: ProgramCollect<Enum = C>,
{
    fn setup(self, program: &mut Program<C>) {
        let args = program.take_args();
        let args = process_renderer_flags(args, program);
        program.replace_args(args);
    }
}

fn process_renderer_flags<C>(args: Vec<String>, program: &mut Program<C>) -> Vec<String>
where
    C: ProgramCollect<Enum = C>,
{
    let (json, json_pretty, yaml, toml, ron, ron_pretty, remains) = args
        .pick(&JSON_FLAG)
        .pick(&JSON_PRETTY_FLAG)
        .pick(&YAML_FLAG)
        .pick(&TOML_FLAG)
        .pick(&RON_FLAG)
        .pick(&RON_PRETTY_FLAG)
        .pick(&REMAINS)
        .unwrap();

    #[cfg(feature = "json_serde_fmt")]
    if *json {
        program.structural_renderer_name = crate::StructuralRendererSetting::Json;
    }
    #[cfg(feature = "json_serde_fmt")]
    if *json_pretty {
        program.structural_renderer_name = crate::StructuralRendererSetting::JsonPretty;
    }
    #[cfg(feature = "yaml_serde_fmt")]
    if *yaml {
        program.structural_renderer_name = crate::StructuralRendererSetting::Yaml;
    }
    #[cfg(feature = "toml_serde_fmt")]
    if *toml {
        program.structural_renderer_name = crate::StructuralRendererSetting::Toml;
    }
    #[cfg(feature = "ron_serde_fmt")]
    if *ron {
        program.structural_renderer_name = crate::StructuralRendererSetting::Ron;
    }
    #[cfg(feature = "ron_serde_fmt")]
    if *ron_pretty {
        program.structural_renderer_name = crate::StructuralRendererSetting::RonPretty;
    }

    remains.into()
}
