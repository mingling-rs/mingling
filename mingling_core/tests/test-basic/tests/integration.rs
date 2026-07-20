use mingling::Flag;
use mingling::NextProcess;
use mingling::Node;
use mingling::RenderResult;
use mingling::StringVec;

#[test]
fn test_node_from_str() {
    let node = Node::from("a.b.c");
    assert_eq!(node.to_string(), "a.b.c");
}

#[test]
fn test_node_kebab_case() {
    let node = Node::from("HelloWorld.FooBar");
    assert_eq!(node.to_string(), "hello-world.foo-bar");
}

#[test]
fn test_node_join() {
    let node = Node::from("base").join("sub");
    assert_eq!(node.to_string(), "base.sub");
}

#[test]
fn test_node_eq() {
    let a = Node::from("x.y");
    let b = Node::from("x.y");
    let c = Node::from("x.z");
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn test_flag_from_static_str() {
    let flag = Flag::from("-h");
    assert_eq!(flag.as_ref(), &["-h"]);
}

#[test]
fn test_flag_from_array() {
    let flag = Flag::from(["-h", "--help"]);
    assert_eq!(flag.as_ref(), &["-h", "--help"]);
}

#[test]
fn test_flag_empty() {
    let flag = Flag::from(());
    assert!(flag.is_empty());
}

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

#[test]
fn test_render_result_clear() {
    let mut r = RenderResult::default();
    r.print("data");
    assert!(!r.is_empty());
    r.clear();
    assert!(r.is_empty());
}

#[test]
fn test_next_process_display() {
    assert_eq!(format!("{}", NextProcess::Chain), "Chain");
    assert_eq!(format!("{}", NextProcess::Renderer), "Renderer");
}

#[test]
fn test_string_vec_from_array() {
    let sv = StringVec::from(["a", "b", "c"]);
    let v: Vec<String> = sv.into();
    assert_eq!(v, vec!["a", "b", "c"]);
}

#[test]
fn test_string_vec_from_vec() {
    let original = vec!["x".to_string(), "y".to_string()];
    let sv = StringVec::from(original.clone());
    assert_eq!(*sv, original);
}
