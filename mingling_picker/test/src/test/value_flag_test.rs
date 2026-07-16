use mingling_picker::value::Flag;
use mingling_picker::{IntoPicker, macros::arg};

// Basic Flag — present / absent

#[test]
fn test_flag_present() {
    let flag: Flag = vec!["--verbose"]
        .to_picker()
        .pick(&arg![verbose: Flag])
        .unwrap();
    assert_eq!(flag, Flag::Active);
}

#[test]
fn test_flag_absent_returns_inactive() {
    // Unlike bool, Flag::pick returns Parsed(Inactive) when no match is found,
    // so or_default() is NOT required — unwrap() works directly.
    let flag: Flag = Vec::<&str>::new()
        .to_picker()
        .pick(&arg![verbose: Flag])
        .unwrap();
    assert_eq!(flag, Flag::Inactive);
}

// Short Flag

#[test]
fn test_flag_short_present() {
    let flag: Flag = vec!["-v"]
        .to_picker()
        .pick(&arg![verbose: Flag, 'v'])
        .unwrap();
    assert_eq!(flag, Flag::Active);
}

// Multiple Flags

#[test]
fn test_two_flags_both_present() {
    let (a, b): (Flag, Flag) = vec!["--flag-a", "--flag-b"]
        .to_picker()
        .pick(&arg![flag_a: Flag])
        .pick(&arg![flag_b: Flag])
        .unwrap();
    assert_eq!(a, Flag::Active);
    assert_eq!(b, Flag::Active);
}

#[test]
fn test_two_flags_one_present() {
    let (a, b): (Flag, Flag) = vec!["--flag-a"]
        .to_picker()
        .pick(&arg![flag_a: Flag])
        .pick(&arg![flag_b: Flag])
        .unwrap();
    assert_eq!(a, Flag::Active);
    assert_eq!(b, Flag::Inactive);
}

#[test]
fn test_two_flags_neither_present() {
    let (a, b): (Flag, Flag) = Vec::<&str>::new()
        .to_picker()
        .pick(&arg![flag_a: Flag])
        .pick(&arg![flag_b: Flag])
        .unwrap();
    assert_eq!(a, Flag::Inactive);
    assert_eq!(b, Flag::Inactive);
}

// After `--` (end-of-options)

#[test]
fn test_flag_after_end_of_options() {
    let flag: Flag = vec!["--", "--verbose"]
        .to_picker()
        .pick(&arg![verbose: Flag])
        .unwrap();
    assert_eq!(flag, Flag::Inactive);
}

// Alias

#[test]
fn test_flag_with_alias() {
    let flag: Flag = vec!["--cfg"]
        .to_picker()
        .pick(&arg![config: Flag, "cfg"])
        .unwrap();
    assert_eq!(flag, Flag::Active);
}

#[test]
fn test_flag_primary_name() {
    let flag: Flag = vec!["--config"]
        .to_picker()
        .pick(&arg![config: Flag, "cfg"])
        .unwrap();
    assert_eq!(flag, Flag::Active);
}

// Unrelated flag should not match

#[test]
fn test_unrelated_flag_does_not_match() {
    let flag: Flag = vec!["--other"]
        .to_picker()
        .pick(&arg![verbose: Flag])
        .unwrap();
    assert_eq!(flag, Flag::Inactive);
}

// to_result / to_option

#[test]
fn test_flag_to_result() {
    let result: Result<Flag, ()> = vec!["--verbose"]
        .to_picker()
        .pick(&arg![verbose: Flag])
        .to_result();
    assert_eq!(result, Ok(Flag::Active));
}

#[test]
fn test_flag_to_option() {
    let opt: Option<Flag> = vec!["--verbose"]
        .to_picker()
        .pick(&arg![verbose: Flag])
        .to_option();
    assert_eq!(opt, Some(Flag::Active));
}

// Bool conversions

#[test]
fn test_flag_converts_to_bool() {
    let flag = Flag::Active;
    assert!(bool::from(flag));

    let flag = Flag::Inactive;
    assert!(!bool::from(flag));
}

#[test]
fn test_flag_from_bool() {
    assert_eq!(Flag::from(true), Flag::Active);
    assert_eq!(Flag::from(false), Flag::Inactive);
}

#[test]
fn test_flag_deref_to_bool() {
    let active = Flag::Active;
    assert!(*active);

    let inactive = Flag::Inactive;
    assert!(!*inactive);
}

// Flag never triggers route (unlike bool)
//
// Flag::pick always returns Parsed, so the fallback chain
// (default → route) is never entered.

#[test]
fn test_flag_absent_does_not_trigger_route() {
    // Even without or_default / or_route, absent flag returns Inactive, not a route
    let result: Result<Flag, &str> = Vec::<&str>::new()
        .with_route::<&str>()
        .pick(&arg![verbose: Flag])
        .or_route(|| "should_not_fire")
        .to_result();
    assert_eq!(result, Ok(Flag::Inactive));
}
