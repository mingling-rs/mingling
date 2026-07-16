use mingling_picker::{IntoPicker, macros::arg};

// Basic named String — present / absent

#[test]
fn test_string_named_present() {
    let val: String = vec!["--name", "Alice"]
        .to_picker()
        .pick(&arg![name: String])
        .or_default()
        .unwrap();
    assert_eq!(val, "Alice");
}

#[test]
fn test_string_named_absent_uses_default() {
    let val: String = Vec::<&str>::new()
        .to_picker()
        .pick(&arg![name: String])
        .or_default()
        .unwrap();
    assert_eq!(val, "");
}

// Named String — eq mode

#[test]
fn test_string_named_eq_mode() {
    let val: String = vec!["--name=Alice"]
        .to_picker()
        .pick(&arg![name: String])
        .or_default()
        .unwrap();
    assert_eq!(val, "Alice");
}

// Named String — short flag

#[test]
fn test_string_named_short_flag() {
    let val: String = vec!["-n", "Alice"]
        .to_picker()
        .pick(&arg![name: String, 'n'])
        .or_default()
        .unwrap();
    assert_eq!(val, "Alice");
}

// Named String — no value after flag

#[test]
fn test_string_named_missing_value_triggers_default() {
    // --name at end with no following arg → pick returns NotFound → or_default gives ""
    let val: String = vec!["--name"]
        .to_picker()
        .pick(&arg![name: String])
        .or_default()
        .unwrap();
    assert_eq!(val, "");
}

// Positional String

#[test]
fn test_string_positional() {
    let val: String = vec!["file.txt"]
        .to_picker()
        .pick(&arg![String])
        .or_default()
        .unwrap();
    assert_eq!(val, "file.txt");
}

#[test]
fn test_string_positional_takes_first() {
    let val: String = vec!["first", "second"]
        .to_picker()
        .pick(&arg![String])
        .or_default()
        .unwrap();
    assert_eq!(val, "first");
}

// Multiple occurrences (Single only tags one occurrence per Parseable)

#[test]
fn test_string_two_named_flags() {
    let (a, b): (String, String) = vec!["--name", "Alice", "--greeting", "Hello"]
        .to_picker()
        .pick(&arg![name: String])
        .or_default()
        .pick(&arg![greeting: String])
        .or_default()
        .unwrap();
    assert_eq!(a, "Alice");
    assert_eq!(b, "Hello");
}

// Mixed named + positional

#[test]
fn test_string_named_and_positional() {
    let (name, file): (String, String) = vec!["--name", "Alice", "file.txt"]
        .to_picker()
        .pick(&arg![name: String])
        .or_default()
        .pick(&arg![String])
        .or_default()
        .unwrap();
    assert_eq!(name, "Alice");
    assert_eq!(file, "file.txt");
}

// After `--` (end-of-options)

#[test]
fn test_string_named_after_end_of_options() {
    // Named arg after `--` should not be matched → default
    let val: String = vec!["--", "--name", "Alice"]
        .to_picker()
        .pick(&arg![name: String])
        .or_default()
        .unwrap();
    assert_eq!(val, "");
}

// Unrelated flag should not match → default

#[test]
fn test_string_unrelated_flag() {
    let val: String = vec!["--other", "value"]
        .to_picker()
        .pick(&arg![name: String])
        .or_default()
        .unwrap();
    assert_eq!(val, "");
}

// to_result / to_option

#[test]
fn test_string_to_result() {
    let result: Result<String, ()> = vec!["--name", "Alice"]
        .to_picker()
        .pick(&arg![name: String])
        .or_default()
        .to_result();
    assert_eq!(result, Ok("Alice".to_string()));
}

#[test]
fn test_string_to_option() {
    let opt: Option<String> = vec!["--name", "Alice"]
        .to_picker()
        .pick(&arg![name: String])
        .or_default()
        .to_option();
    assert_eq!(opt, Some("Alice".to_string()));
}

// Custom default via .or()

#[test]
fn test_string_custom_default() {
    let val: String = Vec::<&str>::new()
        .to_picker()
        .pick(&arg![name: String])
        .or(|| "default_name".to_string())
        .unwrap();
    assert_eq!(val, "default_name");
}
