// Doc Not Optimize
use crate::parser::picker::bools::{True, Yes};
use crate::parser::{Argument, Pick1, Picker};

#[test]
fn test_argument_from_static_str() {
    let arg: Argument = "hello".into();
    assert_eq!(arg.len(), 1);
    assert_eq!(arg[0], "hello");
}

#[test]
fn test_argument_from_slice() {
    let arg: Argument = (&["--name", "value"][..]).into();
    assert_eq!(arg.len(), 2);
    assert_eq!(arg[0], "--name");
    assert_eq!(arg[1], "value");
}

#[test]
fn test_argument_from_array() {
    let arg: Argument = ["--file", "test.txt"].into();
    assert_eq!(arg.len(), 2);
}

#[test]
fn test_argument_from_vec() {
    let arg: Argument = vec!["a".to_string(), "b".to_string()].into();
    assert_eq!(arg.len(), 2);
}

#[test]
fn test_argument_default_is_empty() {
    let arg = Argument::default();
    assert!(arg.is_empty());
}

#[test]
fn test_pick_argument_with_flag() {
    let mut arg: Argument = vec!["--name", "Alice", "--verbose"].into();
    let value = arg.pick_argument("--name");
    assert_eq!(value, Some("Alice".to_string()));
    // After picking, the flag and its value are removed
    assert_eq!(arg.as_ref(), &["--verbose"]);
}

#[test]
fn test_pick_argument_flag_not_found() {
    let mut arg: Argument = vec!["--name", "Alice"].into();
    let value = arg.pick_argument("--missing");
    assert_eq!(value, None);
    // Original args unchanged
    assert_eq!(arg.as_ref(), &["--name", "Alice"]);
}

#[test]
fn test_pick_argument_empty() {
    let mut arg: Argument = Argument::default();
    let value = arg.pick_argument("--flag");
    assert_eq!(value, None);
}

#[test]
fn test_pick_argument_flag_at_end_no_value() {
    let mut arg: Argument = vec!["--name"].into();
    let value = arg.pick_argument("--name");
    assert_eq!(value, None);
    assert!(arg.is_empty());
}

#[test]
fn test_pick_argument_no_flag_positional() {
    let mut arg: Argument = vec!["first", "second", "--flag", "val"].into();
    let value = arg.pick_argument(());
    assert_eq!(value, Some("first".to_string()));
    assert_eq!(arg.as_ref(), &["second", "--flag", "val"]);
}

#[test]
fn test_pick_argument_positional_all() {
    let mut arg: Argument = vec!["one", "two", "three"].into();
    let v1 = arg.pick_argument(());
    let v2 = arg.pick_argument(());
    let v3 = arg.pick_argument(());
    let v4 = arg.pick_argument(());
    assert_eq!(v1, Some("one".to_string()));
    assert_eq!(v2, Some("two".to_string()));
    assert_eq!(v3, Some("three".to_string()));
    assert_eq!(v4, None);
}

#[test]
fn test_pick_argument_empty_args_no_flag() {
    let mut arg: Argument = Argument::default();
    let value = arg.pick_argument(());
    assert_eq!(value, None);
}

#[test]
fn test_pick_argument_with_flag_from_iter() {
    let mut arg: Argument = vec!["-f", "data.txt", "--other"].into();
    let value = arg.pick_argument(&["-f", "--file"][..]);
    assert_eq!(value, Some("data.txt".to_string()));
    assert_eq!(arg.as_ref(), &["--other"]);
}

#[test]
fn test_pick_arguments_multiple_values() {
    let mut arg: Argument = vec!["--files", "a.txt", "b.txt", "c.txt", "--other"].into();
    let values = arg.pick_arguments("--files");
    assert_eq!(values, vec!["a.txt", "b.txt", "c.txt"]);
    assert_eq!(arg.as_ref(), &["--other"]);
}

#[test]
fn test_pick_arguments_single_value() {
    let mut arg: Argument = vec!["--name", "Alice", "--verbose"].into();
    let values = arg.pick_arguments("--name");
    assert_eq!(values, vec!["Alice"]);
    assert_eq!(arg.as_ref(), &["--verbose"]);
}

#[test]
fn test_pick_arguments_no_values() {
    let mut arg: Argument = vec!["--flag", "--other", "val"].into();
    let values = arg.pick_arguments("--flag");
    assert!(values.is_empty());
    assert_eq!(arg.as_ref(), &["--other", "val"]);
}

#[test]
fn test_pick_arguments_flag_not_found() {
    let mut arg: Argument = vec!["--name", "Alice"].into();
    let values = arg.pick_arguments("--missing");
    assert!(values.is_empty());
    assert_eq!(arg.as_ref(), &["--name", "Alice"]);
}

#[test]
fn test_pick_arguments_stops_at_next_flag() {
    let mut arg: Argument = vec!["--list", "a", "b", "-c", "d", "e"].into();
    let values = arg.pick_arguments("--list");
    assert_eq!(values, vec!["a", "b"]);
    assert_eq!(arg.as_ref(), &["-c", "d", "e"]);
}

#[test]
fn test_pick_arguments_empty_flag_positional() {
    let mut arg: Argument = vec!["pos1", "pos2", "--flag", "val"].into();
    let values = arg.pick_arguments(());
    assert_eq!(values, vec!["pos1", "pos2"]);
    assert_eq!(arg.as_ref(), &["--flag", "val"]);
}

#[test]
fn test_pick_arguments_empty_args() {
    let mut arg: Argument = Argument::default();
    let values = arg.pick_arguments("--flag");
    assert!(values.is_empty());
}

#[test]
fn test_pick_flag_found() {
    let mut arg: Argument = vec!["--verbose", "--name", "Alice"].into();
    let result = arg.pick_flag("--verbose");
    assert!(result);
    assert_eq!(arg.as_ref(), &["--name", "Alice"]);
}

#[test]
fn test_pick_flag_not_found() {
    let mut arg: Argument = vec!["--name", "Alice"].into();
    let result = arg.pick_flag("--verbose");
    assert!(!result);
    assert_eq!(arg.as_ref(), &["--name", "Alice"]);
}

#[test]
fn test_pick_flag_empty_args() {
    let mut arg: Argument = Argument::default();
    let result = arg.pick_flag("--flag");
    assert!(!result);
}

#[test]
fn test_pick_flag_with_flag_iter() {
    let mut arg: Argument = vec!["-h", "--name", "Alice"].into();
    let result = arg.pick_flag(&["-h", "--help"][..]);
    assert!(result);
}

#[test]
fn test_pick_flag_second_not_first() {
    let mut arg: Argument = vec!["--name", "Alice"].into();
    let result = arg.pick_flag(&["-h", "--help"][..]);
    assert!(!result);
}

#[test]
fn test_pick_flag_positional_yes() {
    let mut arg: Argument = vec!["yes"].into();
    let result = arg.pick_flag(());
    assert!(result);
    assert!(arg.is_empty());
}

#[test]
fn test_pick_flag_positional_no() {
    let mut arg: Argument = vec!["no"].into();
    let result = arg.pick_flag(());
    assert!(!result);
}

#[test]
fn test_pick_flag_positional_true() {
    let mut arg: Argument = vec!["true"].into();
    let result = arg.pick_flag(());
    assert!(result);
}

#[test]
fn test_pick_flag_positional_false() {
    let mut arg: Argument = vec!["false"].into();
    let result = arg.pick_flag(());
    assert!(!result);
}

#[test]
fn test_pick_flag_positional_1() {
    let mut arg: Argument = vec!["1"].into();
    let result = arg.pick_flag(());
    assert!(result);
}

#[test]
fn test_pick_flag_positional_0() {
    let mut arg: Argument = vec!["0"].into();
    let result = arg.pick_flag(());
    assert!(!result);
}

#[test]
fn test_pick_flag_positional_unknown() {
    let mut arg: Argument = vec!["unknown_value"].into();
    let result = arg.pick_flag(());
    assert!(!result);
}

#[test]
fn test_pick_flag_positional_case_insensitive_yes() {
    let mut arg: Argument = vec!["YeS"].into();
    let result = arg.pick_flag(());
    assert!(result);
}

#[test]
fn test_dump_remains() {
    let mut arg: Argument = vec!["a", "b", "c"].into();
    let remains = arg.dump_remains();
    assert_eq!(remains, vec!["a", "b", "c"]);
    assert!(arg.is_empty());
}

#[test]
fn test_dump_remains_empty() {
    let mut arg: Argument = Argument::default();
    let remains = arg.dump_remains();
    assert!(remains.is_empty());
}

#[test]
fn test_dump_remains_after_pick() {
    let mut arg: Argument = vec!["--flag", "value", "extra"].into();
    let _ = arg.pick_argument("--flag");
    let remains = arg.dump_remains();
    assert_eq!(remains, vec!["extra"]);
}

#[test]
fn test_strip_all_flags() {
    let arg: Argument = vec!["--verbose", "file.txt", "--format", "json"].into();
    let result = arg.strip_all_flags();
    assert_eq!(result.as_ref(), &["file.txt", "json"]);
}

#[test]
fn test_strip_all_flags_no_flags() {
    let arg: Argument = vec!["just", "positional", "args"].into();
    let result = arg.strip_all_flags();
    assert_eq!(result.as_ref(), &["just", "positional", "args"]);
}

#[test]
fn test_strip_all_flags_all_flags() {
    let arg: Argument = vec!["--a", "-b", "--c"].into();
    let result = arg.strip_all_flags();
    assert!(result.is_empty());
}

#[test]
fn test_strip_all_flags_empty() {
    let arg: Argument = Argument::default();
    let result = arg.strip_all_flags();
    assert!(result.is_empty());
}

#[test]
fn test_picker_new() {
    let picker = Picker::new(vec!["--name", "Alice"]);
    assert_eq!(picker.args.len(), 2);
}

#[test]
fn test_picker_from_trait() {
    let picker: Picker = vec!["--name", "Alice"].into();
    assert_eq!(picker.args.len(), 2);
}

#[test]
fn test_picker_pick_string() {
    let result: String = Picker::new(vec!["--name", "Alice"]).pick("--name").unpack();
    assert_eq!(result, "Alice");
}

#[test]
fn test_picker_pick_string_default_when_missing() {
    let result: String = Picker::new(vec!["--other", "val"])
        .pick::<String>("--name")
        .unpack();
    assert_eq!(result, "");
}

#[test]
fn test_picker_pick_string_default_when_missing_with_or() {
    let result: String = Picker::new(vec!["--other", "val"])
        .pick_or("--name", "default_name")
        .unpack();
    assert_eq!(result, "default_name");
}

#[test]
fn test_picker_pick_bool_flag_present() {
    let result: bool = Picker::new(vec!["--verbose", "--name", "Alice"])
        .pick::<bool>("--verbose")
        .unpack();
    assert!(result);
}

#[test]
fn test_picker_pick_bool_flag_absent() {
    let result: bool = Picker::new(vec!["--name", "Alice"])
        .pick::<bool>("--verbose")
        .unpack();
    assert!(!result);
}

#[test]
fn test_picker_pick_i32() {
    let result: i32 = Picker::new(vec!["--count", "42"]).pick("--count").unpack();
    assert_eq!(result, 42);
}

#[test]
fn test_picker_pick_i32_default_zero() {
    let result: i32 = Picker::new(vec!["--other"]).pick::<i32>("--count").unpack();
    assert_eq!(result, 0);
}

#[test]
fn test_picker_pick_f64() {
    let result: f64 = Picker::new(vec!["--ratio", "5.16"])
        .pick("--ratio")
        .unpack();
    let expected: f64 = 5.16;
    assert!((result - expected).abs() < 1e-10);
}

#[test]
fn test_picker_pick_u64() {
    let result: u64 = Picker::new(vec!["--size", "100"]).pick("--size").unpack();
    assert_eq!(result, 100);
}

#[test]
fn test_picker_pick_i32_parse_failure_returns_default() {
    let result: i32 = Picker::new(vec!["--count", "not-a-number"])
        .pick::<i32>("--count")
        .unpack();
    assert_eq!(result, 0);
}

#[test]
fn test_picker_pick_usize_bytes() {
    let result: usize = Picker::new(vec!["--limit", "1024"])
        .pick("--limit")
        .unpack();
    assert_eq!(result, 1024);
}

#[test]
fn test_picker_pick_usize_kib() {
    let result: usize = Picker::new(vec!["--limit", "1KiB"])
        .pick("--limit")
        .unpack();
    assert_eq!(result, 1024);
}

#[test]
fn test_picker_pick_usize_mib() {
    let result: usize = Picker::new(vec!["--limit", "2MiB"])
        .pick("--limit")
        .unpack();
    assert_eq!(result, 2 * 1024 * 1024);
}

#[test]
fn test_picker_pick_usize_parse_failure_returns_default() {
    let result: usize = Picker::new(vec!["--limit", "invalid"])
        .pick::<usize>("--limit")
        .unpack();
    assert_eq!(result, 0);
}

#[test]
fn test_picker_pick_vec_string() {
    let result: Vec<String> = Picker::new(vec!["--files", "a.txt", "b.txt", "c.txt"])
        .pick("--files")
        .unpack();
    assert_eq!(result, vec!["a.txt", "b.txt", "c.txt"]);
}

#[test]
fn test_picker_pick_vec_string_missing() {
    let result: Vec<String> = Picker::new(vec!["--other", "val"])
        .pick::<Vec<String>>("--files")
        .unpack();
    assert!(result.is_empty());
}

#[test]
fn test_picker_pick_vec_usize() {
    let result: Vec<usize> = Picker::new(vec!["--sizes", "100", "1KiB", "2MiB"])
        .pick("--sizes")
        .unpack();
    assert_eq!(result, vec![100, 1024, 2 * 1024 * 1024]);
}

#[test]
fn test_picker_pick_vec_i32() {
    let result: Vec<i32> = Picker::new(vec!["--nums", "10", "20", "30"])
        .pick("--nums")
        .unpack();
    assert_eq!(result, vec![10, 20, 30]);
}

#[test]
fn test_picker_pick_yes_yes() {
    let result: Yes = Picker::new(vec!["--flag", "y"]).pick("--flag").unpack();
    assert!(result.is_yes());
    assert!(*result);
}

#[test]
fn test_picker_pick_yes_no() {
    let result: Yes = Picker::new(vec!["--flag", "no"]).pick("--flag").unpack();
    assert!(result.is_no());
    assert!(!*result);
}

#[test]
fn test_picker_pick_yes_default_no() {
    let result: Yes = Picker::new(vec!["--other"]).pick::<Yes>("--flag").unpack();
    assert!(result.is_no());
}

#[test]
fn test_picker_pick_true_true() {
    let result: True = Picker::new(vec!["--flag", "true"]).pick("--flag").unpack();
    assert!(result.is_true());
    assert!(*result);
}

#[test]
fn test_picker_pick_true_false() {
    let result: True = Picker::new(vec!["--flag", "anything"])
        .pick("--flag")
        .unpack();
    assert!(result.is_false());
    assert!(!*result);
}

#[test]
fn test_picker_pick_true_default_false() {
    let result: True = Picker::new(vec!["--other"]).pick::<True>("--flag").unpack();
    assert!(result.is_false());
}

#[test]
fn test_picker_pick_or_fallback() {
    let result: String = Picker::new(vec!["--other", "val"])
        .pick_or("--name", "fallback")
        .unpack();
    assert_eq!(result, "fallback");
}

#[test]
fn test_picker_pick_or_existing() {
    let result: String = Picker::new(vec!["--name", "Alice"])
        .pick_or("--name", "fallback")
        .unpack();
    assert_eq!(result, "Alice");
}

#[test]
fn test_picker_pick_or_numeric_fallback() {
    let result: i32 = Picker::new(vec!["--other"]).pick_or("--count", 99).unpack();
    assert_eq!(result, 99);
}

#[test]
fn test_picker_pick_or_route_present() {
    let result = Picker::new(vec!["--name", "Alice"])
        .pick_or_route::<String, _>("--name", "missing_name")
        .unpack();
    assert_eq!(result, Ok("Alice".to_string()));
}

#[test]
fn test_picker_pick_or_route_missing() {
    let result = Picker::new(vec!["--other"])
        .pick_or_route::<String, _>("--name", "missing_name")
        .unpack();
    assert_eq!(result, Err("missing_name"));
}

#[test]
fn test_picker_require_present() {
    let result: Option<String> = Picker::new(vec!["--name", "Alice"])
        .require::<String>("--name")
        .map(super::picker::Pick1::unpack);
    assert_eq!(result, Some("Alice".to_string()));
}

#[test]
fn test_picker_require_missing() {
    let result: Option<Pick1<String>> = Picker::new(vec!["--other"]).require::<String>("--name");
    assert!(result.is_none());
}

#[test]
fn test_picker_chaining_two_values() {
    let (name, count): (String, i32) = Picker::new(vec!["--name", "Alice", "--count", "42"])
        .pick::<String>("--name")
        .pick::<i32>("--count")
        .unpack();
    assert_eq!(name, "Alice");
    assert_eq!(count, 42);
}

#[test]
fn test_picker_chaining_three_values() {
    let (_name, _verbose, count): (String, bool, i32) =
        Picker::new(vec!["--name", "Alice", "--count", "42", "--verbose"])
            .pick::<String>("--name")
            .pick::<bool>("--verbose")
            .pick::<i32>("--count")
            .unpack();
    assert_eq!(count, 42);
}

#[test]
fn test_picker_chaining_with_pick_or() {
    let (name, count): (String, i32) = Picker::new(vec!["--name", "Alice"])
        .pick::<String>("--name")
        .pick_or("--count", 10)
        .unpack();
    assert_eq!(name, "Alice");
    assert_eq!(count, 10);
}

#[test]
fn test_picker_chaining_with_mixed_flag_styles() {
    let (name, verbose): (String, bool) = Picker::new(vec!["-n", "Bob", "--verbose"])
        .pick::<String>("-n")
        .pick::<bool>("--verbose")
        .unpack();
    assert_eq!(name, "Bob");
    assert!(verbose);
}

#[test]
fn test_pick_after_modification() {
    let result: String = Picker::new(vec!["--name", "  Alice  "])
        .pick::<String>("--name")
        .after(|s| s.trim().to_string())
        .unpack();
    assert_eq!(result, "Alice");
}

#[test]
fn test_pick_after_chained() {
    let (name, count): (String, i32) = Picker::new(vec!["--name", "alice", "--count", "7"])
        .pick::<String>("--name")
        .after(|s| s.to_uppercase())
        .pick::<i32>("--count")
        .after(|n| n * 2)
        .unpack();
    assert_eq!(name, "ALICE");
    assert_eq!(count, 14);
}

#[test]
fn test_pick_after_or_route_ok() {
    let result = Picker::new(vec!["--name", "Alice"])
        .pick::<String>("--name")
        .after_or_route(|s| {
            if s.len() > 3 {
                Ok(s.clone())
            } else {
                Err("too_short")
            }
        })
        .unpack();
    assert_eq!(result, Ok("Alice".to_string()));
}

#[test]
fn test_pick_after_or_route_err() {
    let result = Picker::new(vec!["--name", "Ab"])
        .pick::<String>("--name")
        .after_or_route(|s| {
            if s.len() > 3 {
                Ok(s.clone())
            } else {
                Err("too_short")
            }
        })
        .unpack();
    assert_eq!(result, Err("too_short"));
}

#[test]
fn test_pick_with_route_unpack_ok() {
    let result = Picker::new(vec!["--name", "Alice"])
        .pick_or_route::<String, _>("--name", "error")
        .unpack();
    assert_eq!(result, Ok("Alice".to_string()));
}

#[test]
fn test_pick_with_route_unpack_err() {
    let result: Result<String, &str> = Picker::new(vec!["--other"])
        .pick_or_route::<String, _>("--name", "missing")
        .unpack();
    assert_eq!(result, Err("missing"));
}

#[test]
fn test_pick_with_route_unpack_directly() {
    let result: String = Picker::new(vec!["--other"])
        .pick_or_route::<String, _>("--name", "fallback_in_route")
        .unpack_directly();
    // When route is set, unpack_directly returns the default value (empty string for String)
    assert_eq!(result, "");
}

#[test]
fn test_pick_with_route_chaining_present() {
    let result = Picker::new(vec!["--name", "Alice", "--count", "42"])
        .pick_or_route::<String, _>("--name", "err_name")
        .pick::<i32>("--count")
        .unpack();
    assert_eq!(result, Ok(("Alice".to_string(), 42)));
}

#[test]
fn test_pick_with_route_chaining_missing_first_route_propagates() {
    let result = Picker::new(vec!["--count", "42"])
        .pick_or_route::<String, _>("--name", "err_name")
        .pick::<i32>("--count")
        .unpack();
    assert_eq!(result, Err("err_name"));
}

#[test]
fn test_pick_with_route_chaining_pick_or_route_second_missing() {
    let result = Picker::new(vec!["--name", "Alice"])
        .pick_or_route::<String, _>("--name", "err_name")
        .pick_or_route::<i32>("--count", "err_count")
        .unpack();
    assert_eq!(result, Err("err_count"));
}

#[test]
fn test_pick_with_route_after_or_route_preserves_existing_route() {
    let result = Picker::new(vec!["--other"])
        .pick_or_route::<String, _>("--name", "missing_name")
        .after_or_route(|_s: &String| {
            // This won't be called because route is already set, but let's see behavior
            Ok("should_not_matter".to_string())
        })
        .unpack();
    assert_eq!(result, Err("missing_name"));
}

#[test]
fn test_picker_operate_args_filter() {
    let result: String = Picker::new(vec!["--name", "Alice", "--verbose"])
        .operate_args(Argument::strip_all_flags)
        .pick_or("--name", "fallback_name")
        .unpack();
    // After stripping flags, "--name" and "--verbose" are gone, "Alice" is a positional arg.
    // But --name with a value won't be present as a flag, so it falls back to positional.
    // Actually, strip_all_flags removes anything starting with '-'.
    // So "--name" is removed, and "Alice" remains as a positional argument.
    // When we try to pick "--name", it won't find it, so we get the fallback.
    assert_eq!(result, "fallback_name");
}

#[test]
fn test_picker_operate_args_transform() {
    let result: Vec<String> = Picker::new(vec!["--files", "a.txt", "b.txt", "c.txt"])
        .operate_args(|mut args| {
            // Add an extra file
            args.push("d.txt".to_string());
            args
        })
        .pick::<Vec<String>>("--files")
        .unpack();
    assert_eq!(result, vec!["a.txt", "b.txt", "c.txt", "d.txt"]);
}
