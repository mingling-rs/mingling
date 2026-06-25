use crate::{
    GeneralRendererSetting, RenderResult, renderer::general::error::GeneralRendererSerializeError,
};
use serde::Serialize;

pub mod error;
pub mod structural_data;

use structural_data::StructuralData;

/// A general renderer that supports multiple serialization formats.
///
/// The `GeneralRenderer` provides methods to serialize data into various formats
/// including JSON, YAML, TOML, and RON, with support for both regular and
/// pretty-printed variants. It is designed to work with types that implement
/// the [`StructuralData`] trait (which implies `Serialize`).
pub struct GeneralRenderer;

impl GeneralRenderer {
    /// Renders data in the specified format to the given `RenderResult`.
    ///
    /// # Errors
    ///
    /// Returns `Err(GeneralRendererSerializeError)` if serialization fails.
    #[allow(unused_variables)]
    pub fn render<T: StructuralData + Send>(
        data: &T,
        setting: &GeneralRendererSetting,
        r: &mut RenderResult,
    ) -> Result<(), GeneralRendererSerializeError> {
        match setting {
            GeneralRendererSetting::Disable => Ok(()),
            #[cfg(feature = "json_serde_fmt")]
            GeneralRendererSetting::Json => Self::render_to_json(data, r),
            #[cfg(feature = "json_serde_fmt")]
            GeneralRendererSetting::JsonPretty => Self::render_to_json_pretty(data, r),
            #[cfg(feature = "yaml_serde_fmt")]
            GeneralRendererSetting::Yaml => Self::render_to_yaml(data, r),
            #[cfg(feature = "toml_serde_fmt")]
            GeneralRendererSetting::Toml => Self::render_to_toml(data, r),
            #[cfg(feature = "ron_serde_fmt")]
            GeneralRendererSetting::Ron => Self::render_to_ron(data, r),
            #[cfg(feature = "ron_serde_fmt")]
            GeneralRendererSetting::RonPretty => Self::render_to_ron_pretty(data, r),
        }
    }

    /// Serializes data to JSON format and writes it to the render result.
    ///
    /// # Errors
    ///
    /// Returns `Err(GeneralRendererSerializeError)` if serialization fails.
    #[cfg(feature = "json_serde_fmt")]
    fn render_to_json<T: Serialize + Send>(
        data: &T,
        r: &mut RenderResult,
    ) -> Result<(), GeneralRendererSerializeError> {
        let json_string = serde_json::to_string(data)
            .map_err(|e| GeneralRendererSerializeError::new(e.to_string()))?;
        r.print(&json_string);
        Ok(())
    }

    /// Serializes data to pretty-printed JSON format and writes it to the render result.
    ///
    /// # Errors
    ///
    /// Returns `Err(GeneralRendererSerializeError)` if serialization fails.
    #[cfg(feature = "json_serde_fmt")]
    fn render_to_json_pretty<T: Serialize + Send>(
        data: &T,
        r: &mut RenderResult,
    ) -> Result<(), GeneralRendererSerializeError> {
        let json_string = serde_json::to_string_pretty(data)
            .map_err(|e| GeneralRendererSerializeError::new(e.to_string()))?;
        r.print(&json_string);
        Ok(())
    }

    /// Serializes data to RON format and writes it to the render result.
    ///
    /// # Errors
    ///
    /// Returns `Err(GeneralRendererSerializeError)` if serialization fails.
    #[cfg(feature = "ron_serde_fmt")]
    fn render_to_ron<T: Serialize + Send>(
        data: &T,
        r: &mut RenderResult,
    ) -> Result<(), GeneralRendererSerializeError> {
        let ron_string = ron::ser::to_string(data)
            .map_err(|e| GeneralRendererSerializeError::new(e.to_string()))?;
        r.print(&ron_string);
        Ok(())
    }

    /// Serializes data to pretty-printed RON format and writes it to the render result.
    ///
    /// # Errors
    ///
    /// Returns `Err(GeneralRendererSerializeError)` if serialization fails.
    #[cfg(feature = "ron_serde_fmt")]
    fn render_to_ron_pretty<T: Serialize + Send>(
        data: &T,
        r: &mut RenderResult,
    ) -> Result<(), GeneralRendererSerializeError> {
        let pretty_config = ron::ser::PrettyConfig::new()
            .new_line("\n")
            .indentor("  ");

        let ron_string = ron::ser::to_string_pretty(data, pretty_config)
            .map_err(|e| GeneralRendererSerializeError::new(e.to_string()))?;
        r.print(&ron_string);
        Ok(())
    }

    /// Serializes data to TOML format and writes it to the render result.
    ///
    /// # Errors
    ///
    /// Returns `Err(GeneralRendererSerializeError)` if serialization fails.
    #[cfg(feature = "toml_serde_fmt")]
    fn render_to_toml<T: Serialize + Send>(
        data: &T,
        r: &mut RenderResult,
    ) -> Result<(), GeneralRendererSerializeError> {
        let toml_string =
            toml::to_string(data).map_err(|e| GeneralRendererSerializeError::new(e.to_string()))?;
        r.print(&toml_string);
        Ok(())
    }

    /// Serializes data to YAML format and writes it to the render result.
    ///
    /// # Errors
    ///
    /// Returns `Err(GeneralRendererSerializeError)` if serialization fails.
    #[cfg(feature = "yaml_serde_fmt")]
    fn render_to_yaml<T: Serialize + Send>(
        data: &T,
        r: &mut RenderResult,
    ) -> Result<(), GeneralRendererSerializeError> {
        let yaml_string = serde_yaml::to_string(data)
            .map_err(|e| GeneralRendererSerializeError::new(e.to_string()))?;
        r.print(&yaml_string);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RenderResult;
    use serde::Serialize;

    #[derive(Debug, Clone, PartialEq, Serialize)]
    struct TestData {
        name: String,
        value: i32,
    }

    impl crate::__private::StructuralDataSealed for TestData {}
    impl StructuralData for TestData {}

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
            GeneralRenderer::render(&test_data(), &GeneralRendererSetting::Disable, &mut r);
        assert!(result.is_ok());
        assert!(r.is_empty());
    }

    #[cfg(feature = "json_serde_fmt")]
    #[test]
    fn test_render_to_json() {
        let mut r = RenderResult::default();
        let result =
            GeneralRenderer::render(&test_data(), &GeneralRendererSetting::Json, &mut r);
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
        let result =
            GeneralRenderer::render(&test_data(), &GeneralRendererSetting::JsonPretty, &mut r);
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
            GeneralRenderer::render(&test_data(), &GeneralRendererSetting::Yaml, &mut r);
        assert!(result.is_ok());
        assert!(!r.is_empty());
    }

    #[cfg(feature = "toml_serde_fmt")]
    #[test]
    fn test_render_to_toml() {
        let mut r = RenderResult::default();
        let result =
            GeneralRenderer::render(&test_data(), &GeneralRendererSetting::Toml, &mut r);
        assert!(result.is_ok());
        assert!(!r.is_empty());
    }

    #[cfg(feature = "ron_serde_fmt")]
    #[test]
    fn test_render_to_ron() {
        let mut r = RenderResult::default();
        let result =
            GeneralRenderer::render(&test_data(), &GeneralRendererSetting::Ron, &mut r);
        assert!(result.is_ok());
        assert!(!r.is_empty());
    }

    #[cfg(feature = "ron_serde_fmt")]
    #[test]
    fn test_render_to_ron_pretty() {
        let mut r = RenderResult::default();
        let result =
            GeneralRenderer::render(&test_data(), &GeneralRendererSetting::RonPretty, &mut r);
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
            GeneralRenderer::render(&test_data(), &GeneralRendererSetting::Disable, &mut r);
        assert!(result.is_ok());
        assert!(r.is_empty());
    }

    #[cfg(feature = "json_serde_fmt")]
    #[test]
    fn test_render_dispatches_json() {
        let mut r = RenderResult::default();
        let result = GeneralRenderer::render(&test_data(), &GeneralRendererSetting::Json, &mut r);
        assert!(result.is_ok());
        assert!(!r.is_empty());
    }
}
