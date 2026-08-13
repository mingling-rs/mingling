// Doc Not Optimize
use crate::{
    ProgramCollect, RenderResult, StructuralRendererSetting,
    renderer::structural::error::StructuralRendererSerializeError,
};
use serde::Serialize;

pub mod error;
pub mod structural_data;

use structural_data::StructuralData;

/// A structural renderer that supports multiple serialization formats.
///
/// The `StructuralRenderer` provides methods to serialize data into various formats
/// including JSON, YAML, TOML, and RON, with support for both regular and
/// pretty-printed variants. It is designed to work with types that implement
/// the [`StructuralData`] trait (which implies `Serialize`).
pub struct StructuralRenderer;

impl StructuralRenderer {
    /// Renders data in the specified format to the given `RenderResult`.
    ///
    /// # Errors
    ///
    /// Returns `Err(StructuralRendererSerializeError)` if serialization fails.
    #[allow(unused_variables)]
    pub fn render<T, C>(
        data: &T,
        setting: &StructuralRendererSetting,
        r: &mut RenderResult,
    ) -> Result<(), StructuralRendererSerializeError>
    where
        T: StructuralData<C> + Send,
        C: ProgramCollect<Enum = C>,
    {
        match setting {
            StructuralRendererSetting::Disable => Ok(()),
            #[cfg(feature = "json_serde_fmt")]
            StructuralRendererSetting::Json => Self::render_to_json(data, r),
            #[cfg(feature = "json_serde_fmt")]
            StructuralRendererSetting::JsonPretty => Self::render_to_json_pretty(data, r),
            #[cfg(feature = "yaml_serde_fmt")]
            StructuralRendererSetting::Yaml => Self::render_to_yaml(data, r),
            #[cfg(feature = "toml_serde_fmt")]
            StructuralRendererSetting::Toml => Self::render_to_toml(data, r),
            #[cfg(feature = "ron_serde_fmt")]
            StructuralRendererSetting::Ron => Self::render_to_ron(data, r),
            #[cfg(feature = "ron_serde_fmt")]
            StructuralRendererSetting::RonPretty => Self::render_to_ron_pretty(data, r),
        }
    }

    /// Serializes data to JSON format and writes it to the render result.
    ///
    /// # Errors
    ///
    /// Returns `Err(StructuralRendererSerializeError)` if serialization fails.
    #[cfg(feature = "json_serde_fmt")]
    fn render_to_json<T: Serialize + Send>(
        data: &T,
        r: &mut RenderResult,
    ) -> Result<(), StructuralRendererSerializeError> {
        let json_string = serde_json::to_string(data)
            .map_err(|e| StructuralRendererSerializeError::new(e.to_string()))?;
        r.print(&json_string);
        Ok(())
    }

    /// Serializes data to pretty-printed JSON format and writes it to the render result.
    ///
    /// # Errors
    ///
    /// Returns `Err(StructuralRendererSerializeError)` if serialization fails.
    #[cfg(feature = "json_serde_fmt")]
    fn render_to_json_pretty<T: Serialize + Send>(
        data: &T,
        r: &mut RenderResult,
    ) -> Result<(), StructuralRendererSerializeError> {
        let json_string = serde_json::to_string_pretty(data)
            .map_err(|e| StructuralRendererSerializeError::new(e.to_string()))?;
        r.print(&json_string);
        Ok(())
    }

    /// Serializes data to RON format and writes it to the render result.
    ///
    /// # Errors
    ///
    /// Returns `Err(StructuralRendererSerializeError)` if serialization fails.
    #[cfg(feature = "ron_serde_fmt")]
    fn render_to_ron<T: Serialize + Send>(
        data: &T,
        r: &mut RenderResult,
    ) -> Result<(), StructuralRendererSerializeError> {
        let ron_string = ron::ser::to_string(data)
            .map_err(|e| StructuralRendererSerializeError::new(e.to_string()))?;
        r.print(&ron_string);
        Ok(())
    }

    /// Serializes data to pretty-printed RON format and writes it to the render result.
    ///
    /// # Errors
    ///
    /// Returns `Err(StructuralRendererSerializeError)` if serialization fails.
    #[cfg(feature = "ron_serde_fmt")]
    fn render_to_ron_pretty<T: Serialize + Send>(
        data: &T,
        r: &mut RenderResult,
    ) -> Result<(), StructuralRendererSerializeError> {
        let pretty_config = ron::ser::PrettyConfig::new().new_line("\n").indentor("  ");

        let ron_string = ron::ser::to_string_pretty(data, pretty_config)
            .map_err(|e| StructuralRendererSerializeError::new(e.to_string()))?;
        r.print(&ron_string);
        Ok(())
    }

    /// Serializes data to TOML format and writes it to the render result.
    ///
    /// # Errors
    ///
    /// Returns `Err(StructuralRendererSerializeError)` if serialization fails.
    #[cfg(feature = "toml_serde_fmt")]
    fn render_to_toml<T: Serialize + Send>(
        data: &T,
        r: &mut RenderResult,
    ) -> Result<(), StructuralRendererSerializeError> {
        let toml_string = toml::to_string(data)
            .map_err(|e| StructuralRendererSerializeError::new(e.to_string()))?;
        r.print(&toml_string);
        Ok(())
    }

    /// Serializes data to YAML format and writes it to the render result.
    ///
    /// # Errors
    ///
    /// Returns `Err(StructuralRendererSerializeError)` if serialization fails.
    #[cfg(feature = "yaml_serde_fmt")]
    fn render_to_yaml<T: Serialize + Send>(
        data: &T,
        r: &mut RenderResult,
    ) -> Result<(), StructuralRendererSerializeError> {
        let yaml_string = serde_yaml::to_string(data)
            .map_err(|e| StructuralRendererSerializeError::new(e.to_string()))?;
        r.print(&yaml_string);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MockProgramCollect, RenderResult};
    use serde::Serialize;

    #[derive(Debug, Clone, PartialEq, Serialize)]
    struct TestData {
        name: String,
        value: i32,
    }

    impl crate::__private::StructuralDataSealed<MockProgramCollect> for TestData {}
    impl StructuralData<MockProgramCollect> for TestData {}

    fn test_data() -> TestData {
        TestData {
            name: "hello".into(),
            value: 42,
        }
    }

    #[test]
    fn test_render_disable_does_nothing() {
        let mut r = RenderResult::default();
        let result =
            StructuralRenderer::render(&test_data(), &StructuralRendererSetting::Disable, &mut r);
        assert!(result.is_ok());
        assert!(r.is_empty());
    }

    #[cfg(feature = "json_serde_fmt")]
    #[test]
    fn test_render_to_json() {
        let mut r = RenderResult::default();
        let result =
            StructuralRenderer::render(&test_data(), &StructuralRendererSetting::Json, &mut r);
        assert!(result.is_ok());
        assert!(!r.is_empty());
        let output: String = r.into();
        assert!(output.contains("\"name\""));
        assert!(output.contains("\"hello\""));
        assert!(output.contains("\"value\""));
        assert!(output.contains("42"));
    }

    #[cfg(feature = "json_serde_fmt")]
    #[test]
    fn test_render_to_json_pretty() {
        let mut r = RenderResult::default();
        let result = StructuralRenderer::render(
            &test_data(),
            &StructuralRendererSetting::JsonPretty,
            &mut r,
        );
        assert!(result.is_ok());
        let output: String = r.into();
        // Pretty JSON has newlines
        assert!(output.contains('\n'));
    }

    #[cfg(feature = "yaml_serde_fmt")]
    #[test]
    fn test_render_to_yaml() {
        let mut r = RenderResult::default();
        let result =
            StructuralRenderer::render(&test_data(), &StructuralRendererSetting::Yaml, &mut r);
        assert!(result.is_ok());
        assert!(!r.is_empty());
    }

    #[cfg(feature = "toml_serde_fmt")]
    #[test]
    fn test_render_to_toml() {
        let mut r = RenderResult::default();
        let result =
            StructuralRenderer::render(&test_data(), &StructuralRendererSetting::Toml, &mut r);
        assert!(result.is_ok());
        assert!(!r.is_empty());
    }

    #[cfg(feature = "ron_serde_fmt")]
    #[test]
    fn test_render_to_ron() {
        let mut r = RenderResult::default();
        let result =
            StructuralRenderer::render(&test_data(), &StructuralRendererSetting::Ron, &mut r);
        assert!(result.is_ok());
        assert!(!r.is_empty());
    }

    #[cfg(feature = "ron_serde_fmt")]
    #[test]
    fn test_render_to_ron_pretty() {
        let mut r = RenderResult::default();
        let result =
            StructuralRenderer::render(&test_data(), &StructuralRendererSetting::RonPretty, &mut r);
        assert!(result.is_ok());
        let output: String = r.into();
        assert!(output.contains('\n'));
    }

    #[test]
    fn test_render_dispatches_correct_format() {
        // Test that render dispatches to the right format handler
        let mut r = RenderResult::default();

        // Disable
        let result =
            StructuralRenderer::render(&test_data(), &StructuralRendererSetting::Disable, &mut r);
        assert!(result.is_ok());
        assert!(r.is_empty());
    }

    #[cfg(feature = "json_serde_fmt")]
    #[test]
    fn test_render_dispatches_json() {
        let mut r = RenderResult::default();
        let result =
            StructuralRenderer::render(&test_data(), &StructuralRendererSetting::Json, &mut r);
        assert!(result.is_ok());
        assert!(!r.is_empty());
    }
}
