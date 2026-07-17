use arg_picker::{IntoPicker, macros::arg};

// Route mechanism — or_route

#[test]
fn test_or_route_triggered_to_result() {
    // flag not present and no default value → route triggered → Err
    let args: Vec<&str> = vec![];
    let result: Result<bool, &'static str> = args
        .with_route::<&'static str>()
        .pick(&arg![verbose: bool])
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
        .pick(&arg![verbose: bool])
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
        .pick(&arg![verbose: bool])
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
        .pick(&arg![verbose: bool])
        .or_route(|| "missing")
        .to_option();
    assert_eq!(opt, None);
}

#[test]
#[should_panic(expected = "missing required argument")]
fn test_or_route_unwrap_panics() {
    let args: Vec<&str> = vec![];
    args.with_route::<&'static str>()
        .pick(&arg![verbose: bool])
        .or_route(|| "missing")
        .unwrap();
}

#[test]
fn test_or_route_with_string_route() {
    // Route type is String
    let args: Vec<&str> = vec![];
    let result: Result<bool, String> = args
        .with_route::<String>()
        .pick(&arg![verbose: bool])
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
        .pick(&arg![verbose: bool])
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
        .pick(&arg![flag_a: bool])
        .or_route(|| "first_missing")
        .pick(&arg![flag_b: bool])
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
        .pick(&arg![flag_a: bool])
        .or_default()
        .pick(&arg![flag_b: bool])
        .or_route(|| "second_missing")
        .to_result();
    assert_eq!(result, Err("second_missing"));
}

#[test]
fn test_two_flags_both_present_route_not_triggered() {
    // Both flags present → route not triggered → Ok((true, true))
    let args = vec!["--flag-a", "--flag-b"];
    let result: Result<(bool, bool), &'static str> = args
        .with_route::<&'static str>()
        .pick(&arg![flag_a: bool])
        .or_route(|| "first_missing")
        .pick(&arg![flag_b: bool])
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
        .pick(&arg![flag_a: bool])
        // No or_default, no or_route either
        .pick(&arg![flag_b: bool])
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
        .pick(&arg![flag_a: bool])
        .pick(&arg![flag_b: bool])
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
        .pick(&arg![verbose: bool])
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
        .pick(&arg![verbose: bool])
        .or_route(|| "string_route".to_string())
        .with_route::<i32>()
        .pick(&arg![other: bool])
        .or_route(|| -1)
        .to_result();
    // The first flag is missing, but its or_route is cleared when with_route switches (route_$ = None)
    // The second flag is missing and has or_route → Err(-1)
    assert_eq!(result, Err(-1));
}
