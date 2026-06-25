use mingling::{GeneralRenderer, GeneralRendererSetting, RenderResult, StructuralData};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize, StructuralData)]
struct TestData {
    name: String,
    value: i32,
}

fn test_data() -> TestData {
    TestData {
        name: "test".into(),
        value: 42,
    }
}

#[test]
fn test_render_disable() {
    let mut r = RenderResult::default();
    let result = GeneralRenderer::render(&test_data(), &GeneralRendererSetting::Disable, &mut r);
    assert!(result.is_ok());
    assert!(r.is_empty());
}

#[test]
fn test_render_json() {
    let mut r = RenderResult::default();
    let result = GeneralRenderer::render(&test_data(), &GeneralRendererSetting::Json, &mut r);
    assert!(result.is_ok());
    assert!(!r.is_empty());
    let output: String = r.into();
    assert!(output.contains("\"name\""));
    assert!(output.contains("\"test\""));
    assert!(output.contains("\"value\""));
    assert!(output.contains("42"));
}

#[test]
fn test_render_yaml() {
    let mut r = RenderResult::default();
    let result = GeneralRenderer::render(&test_data(), &GeneralRendererSetting::Yaml, &mut r);
    assert!(result.is_ok());
    assert!(!r.is_empty());
    let output: String = r.into();
    assert!(output.contains("name:"));
    assert!(output.contains("test"));
    assert!(output.contains("value:"));
    assert!(output.contains("42"));
}

#[test]
fn test_render_toml() {
    let mut r = RenderResult::default();
    let result = GeneralRenderer::render(&test_data(), &GeneralRendererSetting::Toml, &mut r);
    assert!(result.is_ok());
    assert!(!r.is_empty());
    let output: String = r.into();
    assert!(output.contains("name = "));
    assert!(output.contains("test"));
    assert!(output.contains("value = "));
    assert!(output.contains("42"));
}

#[test]
fn test_render_ron() {
    let mut r = RenderResult::default();
    let result = GeneralRenderer::render(&test_data(), &GeneralRendererSetting::Ron, &mut r);
    assert!(result.is_ok());
    assert!(!r.is_empty());
    let output: String = r.into();
    assert!(output.contains("name:"));
    assert!(output.contains("\"test\""));
    assert!(output.contains("value:"));
    assert!(output.contains("42"));
}
