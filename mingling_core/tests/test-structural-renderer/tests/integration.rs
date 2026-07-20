use mingling::{RenderResult, StructuralData, StructuralRenderer, StructuralRendererSetting};
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
    let result =
        StructuralRenderer::render(&test_data(), &StructuralRendererSetting::Disable, &mut r);
    assert!(result.is_ok());
    assert!(r.is_empty());
}

#[test]
fn test_render_json() {
    let mut r = RenderResult::default();
    let result = StructuralRenderer::render(&test_data(), &StructuralRendererSetting::Json, &mut r);
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
    let result = StructuralRenderer::render(&test_data(), &StructuralRendererSetting::Yaml, &mut r);
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
    let result = StructuralRenderer::render(&test_data(), &StructuralRendererSetting::Toml, &mut r);
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
    let result = StructuralRenderer::render(&test_data(), &StructuralRendererSetting::Ron, &mut r);
    assert!(result.is_ok());
    assert!(!r.is_empty());
    let output: String = r.into();
    assert!(output.contains("name:"));
    assert!(output.contains("\"test\""));
    assert!(output.contains("value:"));
    assert!(output.contains("42"));
}

mingling::macros::gen_program!();

