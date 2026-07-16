use mingling_picker::PickerArgInfo;
use mingling_picker::parselib::{ArgMatcher, Matcher, POWERSHELL_STYLE, UNIX_STYLE};

use crate::make_args;

// on_match_one — Named

#[test]
fn test_match_one_named_basic() {
    let mut info = PickerArgInfo::new();
    info.set_long("name");
    let args = make_args(&[("--name", 0), ("Alice", 1)]);
    let result = ArgMatcher::on_match_one(&args, &UNIX_STYLE, &info);
    assert_eq!(result, Some(0));
}

#[test]
fn test_match_one_named_eq_mode() {
    let mut info = PickerArgInfo::new();
    info.set_long("name");
    let args = make_args(&[("--name=Alice", 0)]);
    let result = ArgMatcher::on_match_one(&args, &UNIX_STYLE, &info);
    assert_eq!(result, Some(0));
}

#[test]
fn test_match_one_named_no_match() {
    let mut info = PickerArgInfo::new();
    info.set_long("name");
    let args = make_args(&[("--other", 0), ("Alice", 1)]);
    let result = ArgMatcher::on_match_one(&args, &UNIX_STYLE, &info);
    assert_eq!(result, None);
}

#[test]
fn test_match_one_named_short_flag() {
    let mut info = PickerArgInfo::new();
    info.set_long("name");
    info.set_short('n');
    let args = make_args(&[("-n", 0), ("Alice", 1)]);
    let result = ArgMatcher::on_match_one(&args, &UNIX_STYLE, &info);
    assert_eq!(result, Some(0));
}

#[test]
fn test_match_one_named_after_end_of_options() {
    // Flags after `--` should not be matched.
    let mut info = PickerArgInfo::new();
    info.set_long("name");
    let args = make_args(&[("--", 0), ("--name", 1), ("Alice", 2)]);
    let result = ArgMatcher::on_match_one(&args, &UNIX_STYLE, &info);
    assert_eq!(result, None);
}

// on_match_one — Positional

#[test]
fn test_match_one_positional_basic() {
    let mut info = PickerArgInfo::new();
    info.set_positional(true);
    let args = make_args(&[("file.txt", 0)]);
    let result = ArgMatcher::on_match_one(&args, &UNIX_STYLE, &info);
    assert_eq!(result, Some(0));
}

#[test]
fn test_match_one_positional_takes_first() {
    let mut info = PickerArgInfo::new();
    info.set_positional(true);
    let args = make_args(&[("a.txt", 0), ("b.txt", 1)]);
    let result = ArgMatcher::on_match_one(&args, &UNIX_STYLE, &info);
    assert_eq!(result, Some(0));
}

// on_match_all — Named, single occurrence

#[test]
fn test_match_all_named_flag_plus_value() {
    let mut info = PickerArgInfo::new();
    info.set_long("name");
    let args = make_args(&[("--name", 0), ("Alice", 1)]);
    let result = ArgMatcher::on_match_all(&args, &UNIX_STYLE, &info);
    assert_eq!(result, vec![0, 1]);
}

#[test]
fn test_match_all_named_eq_mode() {
    // --name=Alice: value is inline, only tag the flag position.
    let mut info = PickerArgInfo::new();
    info.set_long("name");
    let args = make_args(&[("--name=Alice", 0)]);
    let result = ArgMatcher::on_match_all(&args, &UNIX_STYLE, &info);
    assert_eq!(result, vec![0]);
}

#[test]
fn test_match_all_named_no_value() {
    // Flag at end with no following arg: only tag the flag.
    let mut info = PickerArgInfo::new();
    info.set_long("name");
    let args = make_args(&[("--name", 0)]);
    let result = ArgMatcher::on_match_all(&args, &UNIX_STYLE, &info);
    assert_eq!(result, vec![0]);
}

#[test]
fn test_match_all_named_value_looks_like_flag() {
    // The next arg looks like a flag — still tag it.
    // Validation is the Pickable's responsibility.
    let mut info = PickerArgInfo::new();
    info.set_long("name");
    let args = make_args(&[("--name", 0), ("--other", 1)]);
    let result = ArgMatcher::on_match_all(&args, &UNIX_STYLE, &info);
    assert_eq!(result, vec![0, 1]);
}

// on_match_all — Named, multiple occurrences (Single per flag)

#[test]
fn test_match_all_named_two_occurrences() {
    // --name Alice --name Bob → each occurrence gets one value.
    let mut info = PickerArgInfo::new();
    info.set_long("name");
    let args = make_args(&[("--name", 0), ("Alice", 1), ("--name", 2), ("Bob", 3)]);
    let result = ArgMatcher::on_match_all(&args, &UNIX_STYLE, &info);
    assert_eq!(result, vec![0, 1, 2, 3]);
}

#[test]
fn test_match_all_named_skips_non_matching_args() {
    // --val a b --val d  → only pairs (0,1) and (3,4); idx 2 ("b") left free.
    let mut info = PickerArgInfo::new();
    info.set_long("val");
    let args = make_args(&[("--val", 0), ("a", 1), ("b", 2), ("--val", 3), ("d", 4)]);
    let result = ArgMatcher::on_match_all(&args, &UNIX_STYLE, &info);
    assert_eq!(result, vec![0, 1, 3, 4]);
}

// on_match_all — Named, short flag

#[test]
fn test_match_all_named_short_flag() {
    let mut info = PickerArgInfo::new();
    info.set_long("name");
    info.set_short('n');
    let args = make_args(&[("-n", 0), ("Alice", 1)]);
    let result = ArgMatcher::on_match_all(&args, &UNIX_STYLE, &info);
    assert_eq!(result, vec![0, 1]);
}

// on_match_all — Named, eq + non-eq mixed

#[test]
fn test_match_all_named_mixed_eq_and_regular() {
    let mut info = PickerArgInfo::new();
    info.set_long("name");
    let args = make_args(&[("--name=Alice", 0), ("--name", 1), ("Bob", 2)]);
    let result = ArgMatcher::on_match_all(&args, &UNIX_STYLE, &info);
    assert_eq!(result, vec![0, 1, 2]);
}

// on_match_all — Named, case insensitive (PowerShell)

#[test]
fn test_match_all_named_powershell_case_insensitive() {
    let mut info = PickerArgInfo::new();
    info.set_long("Name");
    let args = make_args(&[("-name", 0), ("Alice", 1)]);
    let result = ArgMatcher::on_match_all(&args, &POWERSHELL_STYLE, &info);
    assert_eq!(result, vec![0, 1]);
}

// on_match_all — Positional

#[test]
fn test_match_all_positional_single() {
    let mut info = PickerArgInfo::new();
    info.set_positional(true);
    let args = make_args(&[("file.txt", 0)]);
    let result = ArgMatcher::on_match_all(&args, &UNIX_STYLE, &info);
    assert_eq!(result, vec![0]);
}

#[test]
fn test_match_all_positional_multiple() {
    let mut info = PickerArgInfo::new();
    info.set_positional(true);
    let args = make_args(&[("a.txt", 0), ("b.txt", 1)]);
    let result = ArgMatcher::on_match_all(&args, &UNIX_STYLE, &info);
    assert_eq!(result, vec![0, 1]);
}

// End-of-options marker (`--`)

#[test]
fn test_match_all_named_stops_at_end_of_options() {
    // --name before `--` should match, --name after should not.
    let mut info = PickerArgInfo::new();
    info.set_long("name");
    let args = make_args(&[
        ("--name", 0),
        ("Alice", 1),
        ("--", 2),
        ("--name", 3),
        ("Bob", 4),
    ]);
    let result = ArgMatcher::on_match_all(&args, &UNIX_STYLE, &info);
    assert_eq!(result, vec![0, 1]);
}

#[test]
fn test_match_all_positional_stops_at_end_of_options() {
    let mut info = PickerArgInfo::new();
    info.set_positional(true);
    let args = make_args(&[("before", 0), ("--", 1), ("after", 2)]);
    let result = ArgMatcher::on_match_all(&args, &UNIX_STYLE, &info);
    assert_eq!(result, vec![0]);
}

// Empty args

#[test]
fn test_match_one_empty() {
    let mut info = PickerArgInfo::new();
    info.set_long("name");
    let args = vec![];
    let result = ArgMatcher::on_match_one(&args, &UNIX_STYLE, &info);
    assert_eq!(result, None);
}

#[test]
fn test_match_all_empty() {
    let mut info = PickerArgInfo::new();
    info.set_long("name");
    let args = vec![];
    let result = ArgMatcher::on_match_all(&args, &UNIX_STYLE, &info);
    assert!(result.is_empty());
}

// Verify that -- itself is never matched as a flag

#[test]
fn test_match_all_end_of_options_not_matched() {
    // `--` should neither match as a flag nor take a value.
    let mut info = PickerArgInfo::new();
    info.set_long("name");
    let args = make_args(&[("--", 0)]);
    let result = ArgMatcher::on_match_all(&args, &UNIX_STYLE, &info);
    assert!(result.is_empty());
}
