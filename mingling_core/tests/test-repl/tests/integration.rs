use mingling::Flag;
use mingling::Node;
use mingling::RenderResult;
use mingling::core_res::ResREPL;

// ResREPL tests

#[test]
fn test_res_repl_default_exit_false() {
    let res = ResREPL::default();
    assert!(!res.exit);
}

#[test]
fn test_res_repl_exit_true() {
    let mut res = ResREPL::default();
    res.exit = true;
    assert!(res.exit);
}

// Node tests

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

// Flag tests

#[test]
fn test_flag_from_static_str() {
    let flag = Flag::from("-h");
    assert_eq!(flag.as_ref(), &["-h"]);
}

#[test]
fn test_flag_empty() {
    let flag = Flag::from(());
    assert!(flag.is_empty());
}

// RenderResult tests

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
