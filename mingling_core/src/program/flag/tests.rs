// Doc Not Optimize
use crate::Flag;

#[test]
fn test_special_flag() {
    // Test flag found and removed
    let mut args = vec![
        "a".to_string(),
        "b".to_string(),
        "--help".to_string(),
        "c".to_string(),
    ];
    let result = special_flag!(args, "--help");
    assert!(result);
    assert_eq!(args, vec!["a", "b", "c"]);

    // Test flag found at beginning
    let mut args = vec![
        "--help".to_string(),
        "a".to_string(),
        "b".to_string(),
        "c".to_string(),
    ];
    let result = special_flag!(args, "--help");
    assert!(result);
    assert_eq!(args, vec!["a", "b", "c"]);

    // Test flag found at end
    let mut args = vec![
        "a".to_string(),
        "b".to_string(),
        "c".to_string(),
        "--help".to_string(),
    ];
    let result = special_flag!(args, "--help");
    assert!(result);
    assert_eq!(args, vec!["a", "b", "c"]);

    // Test flag not found
    let mut args = vec![
        "a".to_string(),
        "b".to_string(),
        "--other".to_string(),
        "c".to_string(),
    ];
    let result = special_flag!(args, "--help");
    assert!(!result);
    assert_eq!(args, vec!["a", "b", "--other", "c"]);

    // Test multiple same flags all removed
    let mut args = vec![
        "--help".to_string(),
        "a".to_string(),
        "--help".to_string(),
        "b".to_string(),
        "--help".to_string(),
    ];
    let result = special_flag!(args, "--help");
    assert!(result);
    assert_eq!(args, vec!["a", "b"]);

    // Test empty args
    let mut args: Vec<String> = Vec::new();
    let result = special_flag!(args, "--help");
    assert!(!result);
    assert_eq!(args, Vec::<String>::new());

    // Test flag with empty string
    let mut args = vec!["a".to_string(), "".to_string(), "b".to_string()];
    let result = special_flag!(args, "");
    assert!(result);
    assert_eq!(args, vec!["a", "b"]);

    // Test flag with dash in middle
    let mut args = vec!["a".to_string(), "test-flag".to_string(), "b".to_string()];
    let result = special_flag!(args, "test-flag");
    assert!(result);
    assert_eq!(args, vec!["a", "b"]);

    // Test flag that's a substring of another flag (should not match)
    let mut args = vec!["a".to_string(), "--helpful".to_string(), "b".to_string()];
    let result = special_flag!(args, "--help");
    assert!(!result);
    assert_eq!(args, vec!["a", "--helpful", "b"]);
}

#[test]
fn test_special_argument() {
    // Test extracting value after flag
    let mut args = vec![
        "a".to_string(),
        "b".to_string(),
        "--file".to_string(),
        "test.txt".to_string(),
        "c".to_string(),
    ];
    let result = special_argument!(args, "--file");
    assert_eq!(result, Some("test.txt".to_string()));
    assert_eq!(args, vec!["a", "b", "c"]);

    // Test extracting value when flag is at beginning
    let mut args = vec![
        "--file".to_string(),
        "test.txt".to_string(),
        "a".to_string(),
        "b".to_string(),
    ];
    let result = special_argument!(args, "--file");
    assert_eq!(result, Some("test.txt".to_string()));
    assert_eq!(args, vec!["a", "b"]);

    // Test extracting value when flag is at end
    let mut args = vec![
        "a".to_string(),
        "b".to_string(),
        "--file".to_string(),
        "test.txt".to_string(),
    ];
    let result = special_argument!(args, "--file");
    assert_eq!(result, Some("test.txt".to_string()));
    assert_eq!(args, vec!["a", "b"]);

    // Test flag without value (at end)
    let mut args = vec!["a".to_string(), "b".to_string(), "--file".to_string()];
    let result = special_argument!(args, "--file");
    assert_eq!(result, None);
    assert_eq!(args, vec!["a", "b"]);

    // Test flag without value (not at end)
    let mut args = vec!["a".to_string(), "--file".to_string(), "b".to_string()];
    let result = special_argument!(args, "--file");
    assert_eq!(result, Some("b".to_string()));
    assert_eq!(args, vec!["a"]);

    // Test flag not found
    let mut args = vec![
        "a".to_string(),
        "b".to_string(),
        "--other".to_string(),
        "value".to_string(),
    ];
    let result = special_argument!(args, "--file");
    assert_eq!(result, None);
    assert_eq!(args, vec!["a", "b", "--other", "value"]);

    // Test empty args
    let mut args: Vec<String> = Vec::new();
    let result = special_argument!(args, "--file");
    assert_eq!(result, None);
    assert_eq!(args, Vec::<String>::new());

    // Test multiple same flags (should only extract first)
    let mut args = vec![
        "--file".to_string(),
        "first.txt".to_string(),
        "--file".to_string(),
        "second.txt".to_string(),
    ];
    let result = special_argument!(args, "--file");
    assert_eq!(result, Some("first.txt".to_string()));
    assert_eq!(args, vec!["--file", "second.txt"]);

    // Test flag with empty string value
    let mut args = vec![
        "a".to_string(),
        "--file".to_string(),
        "".to_string(),
        "b".to_string(),
    ];
    let result = special_argument!(args, "--file");
    assert_eq!(result, Some("".to_string()));
    assert_eq!(args, vec!["a", "b"]);

    // Test flag with value starting with dash
    let mut args = vec![
        "a".to_string(),
        "--file".to_string(),
        "-value".to_string(),
        "b".to_string(),
    ];
    let result = special_argument!(args, "--file");
    assert_eq!(result, Some("-value".to_string()));
    assert_eq!(args, vec!["a", "b"]);
}

#[test]
fn test_special_arguments() {
    // Test extracting multiple values after flag
    let mut args = vec![
        "a".to_string(),
        "b".to_string(),
        "--list".to_string(),
        "a".to_string(),
        "b".to_string(),
        "c".to_string(),
        "d".to_string(),
        "--next".to_string(),
        "1".to_string(),
    ];
    let result = special_arguments!(args, "--list");
    assert_eq!(result, vec!["a", "b", "c", "d"]);
    assert_eq!(args, vec!["a", "b", "--next", "1"]);

    // Test extracting single value
    let mut args = vec![
        "a".to_string(),
        "b".to_string(),
        "--next".to_string(),
        "1".to_string(),
    ];
    let result = special_arguments!(args, "--next");
    assert_eq!(result, vec!["1"]);
    assert_eq!(args, vec!["a", "b"]);

    // Test extracting from beginning
    let mut args = vec![
        "--list".to_string(),
        "a".to_string(),
        "b".to_string(),
        "--next".to_string(),
        "1".to_string(),
    ];
    let result = special_arguments!(args, "--list");
    assert_eq!(result, vec!["a", "b"]);
    assert_eq!(args, vec!["--next", "1"]);

    // Test extracting when no values after flag
    let mut args = vec![
        "a".to_string(),
        "b".to_string(),
        "--list".to_string(),
        "--next".to_string(),
        "1".to_string(),
    ];
    let result = special_arguments!(args, "--list");
    assert_eq!(result, Vec::<String>::new());
    assert_eq!(args, vec!["a", "b", "--next", "1"]);

    // Test extracting when flag not found
    let mut args = vec![
        "a".to_string(),
        "b".to_string(),
        "--list".to_string(),
        "c".to_string(),
        "d".to_string(),
    ];
    let result = special_arguments!(args, "--none");
    assert_eq!(result, Vec::<String>::new());
    assert_eq!(args, vec!["a", "b", "--list", "c", "d"]);

    // Test extracting empty args
    let mut args: Vec<String> = Vec::new();
    let result = special_arguments!(args, "--list");
    assert_eq!(result, Vec::<String>::new());
    assert_eq!(args, Vec::<String>::new());

    // Test extracting with only flag at end
    let mut args = vec!["--list".to_string()];
    let result = special_arguments!(args, "--list");
    assert_eq!(result, Vec::<String>::new());
    assert_eq!(args, Vec::<String>::new());

    // Test extracting multiple values until end of args
    let mut args = vec![
        "--list".to_string(),
        "a".to_string(),
        "b".to_string(),
        "c".to_string(),
    ];
    let result = special_arguments!(args, "--list");
    assert_eq!(result, vec!["a", "b", "c"]);
    assert_eq!(args, Vec::<String>::new());

    // Test extracting with mixed non-dash values
    let mut args = vec![
        "--list".to_string(),
        "value1".to_string(),
        "value2".to_string(),
        "-next".to_string(),
        "value3".to_string(),
    ];
    let result = special_arguments!(args, "--list");
    assert_eq!(result, vec!["value1", "value2"]);
    assert_eq!(args, vec!["-next", "value3"]);

    // Test extracting with single dash values
    let mut args = vec![
        "--list".to_string(),
        "-a".to_string(),
        "-b".to_string(),
        "--next".to_string(),
        "1".to_string(),
    ];
    let result = special_arguments!(args, "--list");
    assert_eq!(result, Vec::<String>::new());
    assert_eq!(args, vec!["-a", "-b", "--next", "1"]);

    // Test extracting with empty flag
    let mut args = vec![
        "a".to_string(),
        "b".to_string(),
        "--next".to_string(),
        "1".to_string(),
    ];
    let result = special_arguments!(args, "");
    assert_eq!(result, vec!["a", "b"]);
    assert_eq!(args, vec!["--next", "1"]);
}

#[test]
fn test_flag_from_empty_tuple() {
    let flag = Flag::from(());
    assert_eq!(flag.as_ref(), &[] as &[&str]);
}

#[test]
fn test_flag_from_static_str() {
    let flag = Flag::from("-h");
    assert_eq!(flag.as_ref(), &["-h"]);
}

#[test]
fn test_flag_from_slice() {
    let flag = Flag::from(&["-h", "--help"][..]);
    assert_eq!(flag.as_ref(), &["-h", "--help"]);
}

#[test]
fn test_flag_from_array() {
    let flag = Flag::from(["-v", "--verbose"]);
    assert_eq!(flag.as_ref(), &["-v", "--verbose"]);
}

#[test]
fn test_flag_from_ref_array() {
    let flag = Flag::from(&["-f", "--file"]);
    assert_eq!(flag.as_ref(), &["-f", "--file"]);
}

#[test]
fn test_flag_from_ref_flag() {
    let original = Flag::from("-x");
    let cloned = Flag::from(&original);
    assert_eq!(cloned.as_ref(), &["-x"]);
}

#[test]
fn test_flag_as_ref() {
    let flag = Flag::from("-h");
    let r: &[&str] = flag.as_ref();
    assert_eq!(r, &["-h"]);
}

#[test]
fn test_flag_deref() {
    let flag = Flag::from(["-a", "-b"]);
    let collected: Vec<&&str> = flag.iter().collect();
    assert_eq!(collected, vec![&"-a", &"-b"]);
}
