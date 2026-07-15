use mingling_picker::{IntoPicker, macros::flag};

#[test]
fn test_picker_pipeline() {
    let args = vec!["--flag"];
    let parsed = args
        .with_route::<String>()
        .pick(&flag![flag: bool])
        .unwrap();
    assert_eq!(parsed, true);
}
