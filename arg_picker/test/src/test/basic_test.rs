use arg_picker::{macros::arg, IntoPicker};

// Basic bool flag — present / absent

#[test]
fn test_bool_flag_present() {
    let args = vec!["--verbose"];
    let parsed = args
        .to_picker()
        .pick(&arg![verbose: bool])
        .or_default()
        .unwrap();
    assert_eq!(parsed, true);
}

#[test]
fn test_bool_flag_absent() {
    let args: Vec<&str> = vec![];
    let parsed = args
        .to_picker()
        .pick(&arg![verbose: bool])
        .or_default()
        .unwrap();
    assert_eq!(parsed, false);
}

// Short flag — '-v'

#[test]
fn test_bool_short_flag_present() {
    let args = vec!["-v"];
    let parsed = args
        .to_picker()
        .pick(&arg![verbose: bool, 'v'])
        .or_default()
        .unwrap();
    assert_eq!(parsed, true);
}

// Multiple bool flags at once

#[test]
fn test_two_bool_flags_both_present() {
    // UNIX_STYLE now uses Kebab naming: `flag_a` → "flag-a" → --flag-a
    let args = vec!["--flag-a", "--flag-b"];
    let (a, b) = args
        .to_picker()
        .pick(&arg![flag_a: bool])
        .or_default()
        .pick(&arg![flag_b: bool])
        .or_default()
        .unwrap();
    assert_eq!(a, true);
    assert_eq!(b, true);
}

#[test]
fn test_two_bool_flags_one_present() {
    let args = vec!["--flag-a"];
    let (a, b) = args
        .to_picker()
        .pick(&arg![flag_a: bool])
        .or_default()
        .pick(&arg![flag_b: bool])
        .or_default()
        .unwrap();
    assert_eq!(a, true);
    assert_eq!(b, false);
}

#[test]
fn test_two_bool_flags_neither_present() {
    let args: Vec<&str> = vec![];
    let (a, b) = args
        .to_picker()
        .pick(&arg![flag_a: bool])
        .or_default()
        .pick(&arg![flag_b: bool])
        .or_default()
        .unwrap();
    assert_eq!(a, false);
    assert_eq!(b, false);
}

// Mixed short and long flags

#[test]
fn test_short_and_long_flags() {
    let args = vec!["-a", "--long-b"];
    let (a, b) = args
        .to_picker()
        .pick(&arg![flag_a: bool, 'a'])
        .or_default()
        .pick(&arg![long_b: bool])
        .or_default()
        .unwrap();
    assert_eq!(a, true);
    assert_eq!(b, true);
}

// Flags after `--` (end-of-options marker) should not be parsed.

#[test]
fn test_flag_after_end_of_options() {
    let args = vec!["--", "--verbose"];
    let parsed = args
        .to_picker()
        .pick(&arg![verbose: bool])
        .or_default()
        .unwrap();
    assert_eq!(parsed, false);
}

// Alias matching for bool flags

#[test]
fn test_bool_flag_with_alias() {
    let args = vec!["--cfg"];
    let parsed = args
        .to_picker()
        .pick(&arg![config: bool, "cfg"])
        .or_default()
        .unwrap();
    assert_eq!(parsed, true);
}

#[test]
fn test_bool_flag_primary_name() {
    let args = vec!["--config"];
    let parsed = args
        .to_picker()
        .pick(&arg![config: bool, "cfg"])
        .or_default()
        .unwrap();
    assert_eq!(parsed, true);
}

// Short flag + alias for bool flag

#[test]
fn test_bool_flag_short_and_alias() {
    let args = vec!["-v"];
    let parsed = args
        .to_picker()
        .pick(&arg![verbose: bool, 'v', "cfg"])
        .or_default()
        .unwrap();
    assert_eq!(parsed, true);
}

// Default values: .or() / .or_default()

#[test]
fn test_or_default_without_args() {
    let args: Vec<&str> = vec![];
    let parsed = args
        .to_picker()
        .pick(&arg![verbose: bool])
        .or_default()
        .unwrap();
    assert_eq!(parsed, false);
}

#[test]
fn test_or_custom_default() {
    let args: Vec<&str> = vec![];
    let parsed = args
        .to_picker()
        .pick(&arg![verbose: bool])
        .or(|| true)
        .unwrap();
    assert_eq!(parsed, true);
}

// to_result / to_option interface

#[test]
fn test_to_result_ok() {
    let args = vec!["--verbose"];
    let result = args
        .to_picker()
        .pick(&arg![verbose: bool])
        .or_default()
        .to_result();
    assert_eq!(result, Ok(true));
}

#[test]
fn test_to_option_some() {
    let args = vec!["--verbose"];
    let opt = args
        .to_picker()
        .pick(&arg![verbose: bool])
        .or_default()
        .to_option();
    assert_eq!(opt, Some(true));
}

// Chain with_route passthrough

#[test]
fn test_with_route_chain() {
    let args = vec!["--flag"];
    let parsed = args
        .with_route::<String>()
        .pick(&arg![flag: bool])
        .or_default()
        .unwrap();
    assert_eq!(parsed, true);
}

// Unrelated flag should not match

#[test]
fn test_unrelated_flag_does_not_match() {
    let args = vec!["--other"];
    let parsed = args
        .to_picker()
        .pick(&arg![verbose: bool])
        .or_default()
        .unwrap();
    assert_eq!(parsed, false);
}
