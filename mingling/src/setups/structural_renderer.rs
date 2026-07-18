#![allow(deprecated)]

use mingling_core::{Program, ProgramCollect, setup::ProgramSetup};

/// Sets up the structural renderer for the program:
///
/// - Adds a `--renderer` global argument to specify the renderer type
#[cfg_attr(
    feature = "picker",
    deprecated(
        note = "When the `picker` feature is enabled, you can use `mingling::setup::picker::StructuralRendererSimpleSetup` instead"
    )
)]
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
#[cfg_attr(
    feature = "picker",
    deprecated(
        note = "When the `picker` feature is enabled, you can use `mingling::setup::picker::StructuralRendererSetup` instead"
    )
)]
pub struct StructuralRendererSetup;

impl<C> ProgramSetup<C> for StructuralRendererSetup
where
    C: ProgramCollect<Enum = C>,
{
    #[allow(unused_variables)]
    fn setup(self, program: &mut Program<C>) {
        #[cfg(feature = "json_serde_fmt")]
        program.global_flag("--json", |p| {
            p.structural_renderer_name = crate::StructuralRendererSetting::Json;
        });
        #[cfg(feature = "json_serde_fmt")]
        program.global_flag("--json-pretty", |p| {
            p.structural_renderer_name = crate::StructuralRendererSetting::JsonPretty;
        });
        #[cfg(feature = "yaml_serde_fmt")]
        program.global_flag("--yaml", |p| {
            p.structural_renderer_name = crate::StructuralRendererSetting::Yaml;
        });
        #[cfg(feature = "toml_serde_fmt")]
        program.global_flag("--toml", |p| {
            p.structural_renderer_name = crate::StructuralRendererSetting::Toml;
        });
        #[cfg(feature = "ron_serde_fmt")]
        program.global_flag("--ron", |p| {
            p.structural_renderer_name = crate::StructuralRendererSetting::Ron;
        });
        #[cfg(feature = "ron_serde_fmt")]
        program.global_flag("--ron-pretty", |p| {
            p.structural_renderer_name = crate::StructuralRendererSetting::RonPretty;
        });
    }
}
