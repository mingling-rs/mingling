use mingling_picker::parselib::{
    MaskedArg, Matcher, POWERSHELL_STYLE, UNIX_STYLE, WINDOWS_STYLE, build_possible_flags,
};
use mingling_picker::{IntoPicker, PickerArgInfo, macros::flag};

// Basic bool flag — present / absent

#[test]
fn test_bool_flag_present() {
    let args = vec!["--verbose"];
    let parsed = args
        .to_picker()
        .pick(&flag![verbose: bool])
        .or_default()
        .unwrap();
    assert_eq!(parsed, true);
}

#[test]
fn test_bool_flag_absent() {
    let args: Vec<&str> = vec![];
    let parsed = args
        .to_picker()
        .pick(&flag![verbose: bool])
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
        .pick(&flag![verbose: bool, 'v'])
        .or_default()
        .unwrap();
    assert_eq!(parsed, true);
}

// Multiple bool flags at once

#[test]
fn test_two_bool_flags_both_present() {
    // The `flag!` macro expands the identifier `flag_a` into the string "flag_a" → --flag_a
    let args = vec!["--flag_a", "--flag_b"];
    let (a, b) = args
        .to_picker()
        .pick(&flag![flag_a: bool])
        .or_default()
        .pick(&flag![flag_b: bool])
        .or_default()
        .unwrap();
    assert_eq!(a, true);
    assert_eq!(b, true);
}

#[test]
fn test_two_bool_flags_one_present() {
    let args = vec!["--flag_a"];
    let (a, b) = args
        .to_picker()
        .pick(&flag![flag_a: bool])
        .or_default()
        .pick(&flag![flag_b: bool])
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
        .pick(&flag![flag_a: bool])
        .or_default()
        .pick(&flag![flag_b: bool])
        .or_default()
        .unwrap();
    assert_eq!(a, false);
    assert_eq!(b, false);
}

// Mixed short and long flags

#[test]
fn test_short_and_long_flags() {
    let args = vec!["-a", "--long_b"];
    let (a, b) = args
        .to_picker()
        .pick(&flag![flag_a: bool, 'a'])
        .or_default()
        .pick(&flag![long_b: bool])
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
        .pick(&flag![verbose: bool])
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
        .pick(&flag![config: bool, "cfg"])
        .or_default()
        .unwrap();
    assert_eq!(parsed, true);
}

#[test]
fn test_bool_flag_primary_name() {
    let args = vec!["--config"];
    let parsed = args
        .to_picker()
        .pick(&flag![config: bool, "cfg"])
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
        .pick(&flag![verbose: bool, 'v', "cfg"])
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
        .pick(&flag![verbose: bool])
        .or_default()
        .unwrap();
    assert_eq!(parsed, false);
}

#[test]
fn test_or_custom_default() {
    let args: Vec<&str> = vec![];
    let parsed = args
        .to_picker()
        .pick(&flag![verbose: bool])
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
        .pick(&flag![verbose: bool])
        .or_default()
        .to_result();
    assert_eq!(result, Ok(true));
}

#[test]
fn test_to_option_some() {
    let args = vec!["--verbose"];
    let opt = args
        .to_picker()
        .pick(&flag![verbose: bool])
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
        .pick(&flag![flag: bool])
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
        .pick(&flag![verbose: bool])
        .or_default()
        .unwrap();
    assert_eq!(parsed, false);
}

// Route mechanism — or_route

#[test]
fn test_or_route_triggered_to_result() {
    // flag not present and no default value → route triggered → Err
    let args: Vec<&str> = vec![];
    let result: Result<bool, &'static str> = args
        .with_route::<&'static str>()
        .pick(&flag![verbose: bool])
        .or_route(|| "missing_verbose")
        .to_result();
    assert_eq!(result, Err("missing_verbose"));
}

#[test]
fn test_or_route_not_triggered_when_flag_present() {
    // flag present → route not triggered → Ok
    let args = vec!["--verbose"];
    let result: Result<bool, &'static str> = args
        .with_route::<&'static str>()
        .pick(&flag![verbose: bool])
        .or_route(|| "missing_verbose")
        .to_result();
    assert_eq!(result, Ok(true));
}

#[test]
fn test_or_default_priority_over_or_route() {
    // or_default takes priority over or_route: even if the flag is absent,
    // having a default prevents the route from being triggered
    let args: Vec<&str> = vec![];
    let result: Result<bool, &'static str> = args
        .with_route::<&'static str>()
        .pick(&flag![verbose: bool])
        .or_default()
        .or_route(|| "should_not_reach")
        .to_result();
    assert_eq!(result, Ok(false));
}

#[test]
fn test_or_route_to_option_returns_none() {
    // When route is triggered, to_option returns None
    let args: Vec<&str> = vec![];
    let opt: Option<bool> = args
        .with_route::<&'static str>()
        .pick(&flag![verbose: bool])
        .or_route(|| "missing")
        .to_option();
    assert_eq!(opt, None);
}

#[test]
#[should_panic(expected = "called `Option::unwrap()` on a `None` value")]
fn test_or_route_unwrap_panics() {
    let args: Vec<&str> = vec![];
    args.with_route::<&'static str>()
        .pick(&flag![verbose: bool])
        .or_route(|| "missing")
        .unwrap();
}

#[test]
fn test_or_route_with_string_route() {
    // Route type is String
    let args: Vec<&str> = vec![];
    let result: Result<bool, String> = args
        .with_route::<String>()
        .pick(&flag![verbose: bool])
        .or_route(|| "route_hit".to_string())
        .to_result();
    assert_eq!(result, Err("route_hit".to_string()));
}

#[test]
fn test_or_route_with_custom_enum() {
    // Route type is a custom enum
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[allow(dead_code)]
    enum Redirect {
        Help,
        Version,
    }

    let args: Vec<&str> = vec![];
    let result: Result<bool, Redirect> = args
        .with_route::<Redirect>()
        .pick(&flag![verbose: bool])
        .or_route(|| Redirect::Help)
        .to_result();
    assert_eq!(result, Err(Redirect::Help));
}

// Route mechanism — multiple flags

#[test]
fn test_two_flags_first_missing_triggers_route() {
    // Both flags missing; first has or_route → Err("first");
    // first route wins (first-checked, first-triggered)
    let args: Vec<&str> = vec![];
    let result: Result<(bool, bool), &'static str> = args
        .with_route::<&'static str>()
        .pick(&flag![flag_a: bool])
        .or_route(|| "first_missing")
        .pick(&flag![flag_b: bool])
        .or_route(|| "second_missing")
        .to_result();
    assert_eq!(result, Err("first_missing"));
}

#[test]
fn test_two_flags_second_missing_triggers_route() {
    // First has or_default (no route triggered), second missing with or_route → Err("second")
    let args: Vec<&str> = vec![];
    let result: Result<(bool, bool), &'static str> = args
        .with_route::<&'static str>()
        .pick(&flag![flag_a: bool])
        .or_default()
        .pick(&flag![flag_b: bool])
        .or_route(|| "second_missing")
        .to_result();
    assert_eq!(result, Err("second_missing"));
}

#[test]
fn test_two_flags_both_present_route_not_triggered() {
    // Both flags present → route not triggered → Ok((true, true))
    let args = vec!["--flag_a", "--flag_b"];
    let result: Result<(bool, bool), &'static str> = args
        .with_route::<&'static str>()
        .pick(&flag![flag_a: bool])
        .or_route(|| "first_missing")
        .pick(&flag![flag_b: bool])
        .or_route(|| "second_missing")
        .to_result();
    assert_eq!(result, Ok((true, true)));
}

#[test]
fn test_two_flags_first_missing_no_route_second_has_route() {
    // First missing but has no or_route, second missing with or_route → Err("second")
    // Note: the first has neither route nor default, so pick failure does not modify error_route
    let args: Vec<&str> = vec![];
    let result: Result<(bool, bool), &'static str> = args
        .with_route::<&'static str>()
        .pick(&flag![flag_a: bool])
        // No or_default, no or_route either
        .pick(&flag![flag_b: bool])
        .or_route(|| "second_missing")
        .to_result();
    assert_eq!(result, Err("second_missing"));
}

#[test]
fn test_two_flags_only_second_has_default() {
    // First is missing with no default/route (v1=NotFound), second is missing but has or_default
    // Use unpack to check both results
    let args: Vec<&str> = vec![];
    let (a, b) = args
        .to_picker()
        .pick(&flag![flag_a: bool])
        .pick(&flag![flag_b: bool])
        .or_default()
        .unpack();
    assert_eq!(a, None);
    assert_eq!(b, Some(false));
}

// Route with with_route

#[test]
fn test_with_route_and_or_route() {
    // with_route::&lt;String&gt;() sets the Route type + or_route triggers
    let args: Vec<&str> = vec![];
    let result: Result<bool, String> = args
        .with_route::<String>()
        .pick(&flag![verbose: bool])
        .or_route(|| "redirected".to_string())
        .to_result();
    assert_eq!(result, Err("redirected".to_string()));
}

#[test]
fn test_with_route_type_switch() {
    // with_route switching Route type clears old route_$ and error_route
    let args: Vec<&str> = vec![];
    let result: Result<(bool, bool), i32> = args
        .with_route::<String>()
        .pick(&flag![verbose: bool])
        .or_route(|| "string_route".to_string())
        .with_route::<i32>()
        .pick(&flag![other: bool])
        .or_route(|| -1)
        .to_result();
    // The first flag is missing, but its or_route is cleared when with_route switches (route_$ = None)
    // The second flag is missing and has or_route → Err(-1)
    assert_eq!(result, Err(-1));
}

// Style: formatting utilities

#[test]
fn test_unix_style_flag_string() {
    assert_eq!(UNIX_STYLE.flag_string('v'), "-v");
    assert_eq!(UNIX_STYLE.flag_string("verbose"), "--verbose");
}

#[test]
fn test_windows_style_flag_string() {
    assert_eq!(WINDOWS_STYLE.flag_string('v'), "/v");
    assert_eq!(WINDOWS_STYLE.flag_string("verbose"), "/verbose");
}

#[test]
fn test_powershell_style_flag_string() {
    assert_eq!(POWERSHELL_STYLE.flag_string('v'), "-v");
    assert_eq!(POWERSHELL_STYLE.flag_string("Verbose"), "-Verbose");
}

#[test]
fn test_build_possible_flags_windows() {
    // Build PickerArgInfo from a flag definition: `verbose: bool`
    let mut info = PickerArgInfo::new();
    info.set_long("verbose");
    let flags = build_possible_flags(&WINDOWS_STYLE, &info);
    assert_eq!(flags, vec!["/verbose"]);
}

#[test]
fn test_build_possible_flags_with_short_and_alias() {
    let mut info = PickerArgInfo::new();
    info.set_short('n');
    info.set_long("name");
    info.set_alias(vec!["nickname"]);
    let flags = build_possible_flags(&UNIX_STYLE, &info);
    assert_eq!(flags, vec!["-n", "--name", "--nickname"]);
}

// Style: matching with different styles via Matcher trait

fn make_masked(raw: &str, idx: usize) -> MaskedArg<'_> {
    MaskedArg { raw, raw_idx: idx }
}

#[test]
fn test_windows_style_match() {
    // Windows style: /verbose (case insensitive)
    let mut info = PickerArgInfo::new();
    info.set_long("verbose");

    let args = vec![make_masked("/verbose", 0)];
    let result = <bool as Matcher>::on_match_one(&args, &WINDOWS_STYLE, &info);
    assert_eq!(result, Some(0));
}

#[test]
fn test_windows_style_match_case_insensitive() {
    // Windows style is case-insensitive: /VERBOSE should match "verbose"
    let mut info = PickerArgInfo::new();
    info.set_long("verbose");

    let args = vec![make_masked("/VERBOSE", 0)];
    let result = <bool as Matcher>::on_match_one(&args, &WINDOWS_STYLE, &info);
    assert_eq!(result, Some(0));
}

#[test]
fn test_windows_style_no_match_on_unrelated_flag() {
    // Different flag should not match
    let mut info = PickerArgInfo::new();
    info.set_long("verbose");

    let args = vec![make_masked("/output", 0)];
    let result = <bool as Matcher>::on_match_one(&args, &WINDOWS_STYLE, &info);
    assert_eq!(result, None);
}

#[test]
fn test_powershell_style_match() {
    // PowerShell style: -Verbose (case insensitive)
    let mut info = PickerArgInfo::new();
    info.set_long("Verbose");

    let args = vec![make_masked("-Verbose", 0)];
    let result = <bool as Matcher>::on_match_one(&args, &POWERSHELL_STYLE, &info);
    assert_eq!(result, Some(0));
}

#[test]
fn test_powershell_style_match_case_insensitive() {
    let mut info = PickerArgInfo::new();
    info.set_long("Verbose");

    let args = vec![make_masked("-VERBOSE", 0)];
    let result = <bool as Matcher>::on_match_one(&args, &POWERSHELL_STYLE, &info);
    assert_eq!(result, Some(0));
}

#[test]
fn test_unix_style_case_sensitive_no_match() {
    // UNIX style is case-sensitive: --VERBOSE should NOT match --verbose
    let mut info = PickerArgInfo::new();
    info.set_long("verbose");

    let args = vec![make_masked("--VERBOSE", 0)];
    let result = <bool as Matcher>::on_match_one(&args, &UNIX_STYLE, &info);
    assert_eq!(result, None);
}

#[test]
fn test_windows_style_match_all() {
    // on_match_all should find all matching flags
    let mut info = PickerArgInfo::new();
    info.set_long("verbose");
    info.set_short('v');

    let args = vec![
        make_masked("/v", 0),
        make_masked("/output", 1),
        make_masked("/VERBOSE", 2),
    ];
    let result = <bool as Matcher>::on_match_all(&args, &WINDOWS_STYLE, &info);
    assert_eq!(result, vec![0, 2]);
}

#[test]
fn test_windows_style_match_after_end_of_options() {
    // Flags after -- should not match
    let mut info = PickerArgInfo::new();
    info.set_long("verbose");

    let args = vec![
        make_masked("/verbose", 0),
        make_masked("--", 1),
        make_masked("/verbose", 2),
    ];
    let result = <bool as Matcher>::on_match_all(&args, &WINDOWS_STYLE, &info);
    // end_of_options is always "--" regardless of style
    assert_eq!(result, vec![0]);
}
