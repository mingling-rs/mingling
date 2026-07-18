use mingling_core::{Program, ProgramCollect, setup::ProgramSetup};

use crate::{
    consts::RENDERER_ARG,
    picker::{pick_global_argument, pick_global_flag},
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
        if let Some(renderer) = pick_global_argument(program, &RENDERER_ARG) {
            program.structural_renderer_name = renderer.into();
        }
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
///
/// # Flag priority
///
/// If multiple flags are specified, the last matching flag in the following
/// declaration order takes precedence:
///   1. `--json`
///   2. `--json-pretty`
///   3. `--yaml`
///   4. `--toml`
///   5. `--ron`
///   6. `--ron-pretty`
pub struct StructuralRendererSetup;

impl<C> ProgramSetup<C> for StructuralRendererSetup
where
    C: ProgramCollect<Enum = C>,
{
    fn setup(self, program: &mut Program<C>) {
        #[cfg(feature = "json_serde_fmt")]
        if pick_global_flag(program, &crate::consts::JSON_FLAG) {
            program.structural_renderer_name = crate::StructuralRendererSetting::Json;
        }
        #[cfg(feature = "json_serde_fmt")]
        if pick_global_flag(program, &crate::consts::JSON_PRETTY_FLAG) {
            program.structural_renderer_name = crate::StructuralRendererSetting::JsonPretty;
        }
        #[cfg(feature = "yaml_serde_fmt")]
        if pick_global_flag(program, &crate::consts::YAML_FLAG) {
            program.structural_renderer_name = crate::StructuralRendererSetting::Yaml;
        }
        #[cfg(feature = "toml_serde_fmt")]
        if pick_global_flag(program, &crate::consts::TOML_FLAG) {
            program.structural_renderer_name = crate::StructuralRendererSetting::Toml;
        }
        #[cfg(feature = "ron_serde_fmt")]
        if pick_global_flag(program, &crate::consts::RON_FLAG) {
            program.structural_renderer_name = crate::StructuralRendererSetting::Ron;
        }
        #[cfg(feature = "ron_serde_fmt")]
        if pick_global_flag(program, &crate::consts::RON_PRETTY_FLAG) {
            program.structural_renderer_name = crate::StructuralRendererSetting::RonPretty;
        }
    }
}
