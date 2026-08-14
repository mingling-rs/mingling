use mingling::Flag;
use mingling::NextProcess;
use mingling::Node;
use mingling::Program;
use mingling::RenderResult;
use mingling::StringVec;
use mingling::StructuralData;
use mingling::StructuralRenderer;
use mingling::config::StructuralRendererSetting;
use mingling::core_res::ResREPL;
use mingling::hook::ProgramHook;
use mingling::{ShellContext, ShellFlag, Suggest};
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};

// ShellContext

#[test]
fn test_shell_context_from_args() {
    let ctx = ShellContext::try_from(vec![
        "-f".to_string(),
        "app greet".to_string(),
        "-F".to_string(),
        "zsh".to_string(),
    ])
    .unwrap();
    assert!(matches!(ctx.shell_flag, ShellFlag::Zsh));
    assert_eq!(ctx.all_words, vec!["app", "greet"]);
}

// Suggest

#[test]
fn test_suggest_creation() {
    let s: Suggest = vec!["--help".to_string()].into();
    assert!(matches!(s, Suggest::Suggest(_)));
}

// ResREPL

#[test]
fn test_res_repl_default() {
    let res = ResREPL::default();
    assert!(!res.exit);
}

// Node

#[test]
fn test_node_creation() {
    let node = Node::from("a.b.c");
    assert_eq!(node.to_string(), "a.b.c");
}

#[test]
fn test_node_kebab() {
    let node = Node::from("HelloWorld.FooBar");
    assert_eq!(node.to_string(), "hello-world.foo-bar");
}

// Flag

#[test]
fn test_flag_conversion() {
    let flag = Flag::from(["-h", "--help"]);
    assert_eq!(flag.as_ref(), &["-h", "--help"]);
}

#[test]
fn test_flag_empty() {
    let flag = Flag::from(());
    assert!(flag.is_empty());
}

// RenderResult

#[test]
fn test_render_result_default() {
    let r = RenderResult::default();
    assert!(r.is_empty());
    assert_eq!(r.exit_code, 0);
}

#[test]
fn test_render_result_print() {
    let mut r = RenderResult::default();
    r.print("hello");
    assert_eq!(r.to_string().as_str(), "hello");
}

// StructuralRenderer

#[derive(Debug, Clone, PartialEq, Serialize, StructuralData)]
struct TestData {
    name: String,
    value: i32,
}

#[test]
fn test_structural_renderer_disable() {
    let data = TestData {
        name: "test".into(),
        value: 42,
    };
    let mut r = RenderResult::default();
    let result = StructuralRenderer::render(&data, &StructuralRendererSetting::Disable, &mut r);
    assert!(result.is_ok());
    assert!(r.is_empty());
}

#[test]
fn test_structural_renderer_json() {
    let data = TestData {
        name: "test".into(),
        value: 42,
    };
    let mut r = RenderResult::default();
    let result = StructuralRenderer::render(&data, &StructuralRendererSetting::Json, &mut r);
    assert!(result.is_ok());
    assert!(!r.is_empty());
}

// is_completing

#[test]
fn test_is_completing() {
    let program: Program<crate::ThisProgram> = Program::new_with_args(["app", "__comp"]);
    assert!(program.is_completing());
}

#[test]
fn test_is_not_completing() {
    let program: Program<crate::ThisProgram> = Program::new_with_args(["app", "greet"]);
    assert!(!program.is_completing());
}

// Hooks

#[test]
fn test_hook_setup() {
    static CALLED: AtomicBool = AtomicBool::new(false);

    let hook = ProgramHook::<crate::ThisProgram>::empty().on_begin::<_, ()>(|_| {
        CALLED.store(true, Ordering::SeqCst);
    });

    assert!(hook.begin.is_some());
    (hook.begin.unwrap())(&mingling::hook::HookBeginInfo {});
    assert!(CALLED.load(Ordering::SeqCst));
}

// NextProcess

#[test]
fn test_next_process_display() {
    assert_eq!(format!("{}", NextProcess::Chain), "Chain");
    assert_eq!(format!("{}", NextProcess::Renderer), "Renderer");
}

// StringVec

#[test]
fn test_string_vec_from_array() {
    let sv = StringVec::from(["a", "b", "c"]);
    let v: Vec<String> = sv.into();
    assert_eq!(v, vec!["a", "b", "c"]);
}

mingling::macros::gen_program!();
