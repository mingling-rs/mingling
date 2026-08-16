use mingling::Flag;
use mingling::NextProcess;
use mingling::RenderResult;
use mingling::StringVec;

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
