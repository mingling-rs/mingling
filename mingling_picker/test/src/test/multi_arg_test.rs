use mingling_picker::parselib::{Matcher, MultiArgMatcher, UNIX_STYLE};
use mingling_picker::PickerArgInfo;

use crate::make_args;

// on_match_one — basic

#[test]
fn test_multi_one_finds_first_flag() {
    let mut info = PickerArgInfo::new();
    info.set_long("val");
    let args = make_args(&[("--other", 0), ("--val", 1), ("a", 2), ("b", 3)]);
    let result = MultiArgMatcher::on_match_one(&args, &UNIX_STYLE, &info);
    assert_eq!(result, Some(1));
}

#[test]
fn test_multi_one_no_match() {
    let mut info = PickerArgInfo::new();
    info.set_long("val");
    let args = make_args(&[("--other", 0)]);
    let result = MultiArgMatcher::on_match_one(&args, &UNIX_STYLE, &info);
    assert_eq!(result, None);
}

// on_match_all — named, basic multi-value

#[test]
fn test_multi_all_flag_takes_all_values_until_next_flag() {
    // --val a b c  → all three values belong to --val
    let mut info = PickerArgInfo::new();
    info.set_long("val");
    let args = make_args(&[("--val", 0), ("a", 1), ("b", 2), ("c", 3)]);
    let result = MultiArgMatcher::on_match_all(&args, &UNIX_STYLE, &info);
    assert_eq!(result, vec![0, 1, 2, 3]);
}

#[test]
fn test_multi_all_stops_at_next_flag() {
    // --val a b --other c d  → only a,b belong to --val
    let mut info = PickerArgInfo::new();
    info.set_long("val");
    let args = make_args(&[("--val", 0), ("a", 1), ("b", 2), ("--other", 3), ("c", 4)]);
    let result = MultiArgMatcher::on_match_all(&args, &UNIX_STYLE, &info);
    assert_eq!(result, vec![0, 1, 2]);
}

#[test]
fn test_multi_all_flag_no_values() {
    // --val at end with no values → just the flag
    let mut info = PickerArgInfo::new();
    info.set_long("val");
    let args = make_args(&[("--val", 0)]);
    let result = MultiArgMatcher::on_match_all(&args, &UNIX_STYLE, &info);
    assert_eq!(result, vec![0]);
}

#[test]
fn test_multi_all_empty() {
    let mut info = PickerArgInfo::new();
    info.set_long("val");
    let args = vec![];
    let result = MultiArgMatcher::on_match_all(&args, &UNIX_STYLE, &info);
    assert!(result.is_empty());
}

// on_match_all — named, multiple occurrences of same flag

#[test]
fn test_multi_all_two_occurrences() {
    // --val a b --val c d  → two groups
    let mut info = PickerArgInfo::new();
    info.set_long("val");
    let args = make_args(&[
        ("--val", 0), ("a", 1), ("b", 2),
        ("--val", 3), ("c", 4), ("d", 5),
    ]);
    let result = MultiArgMatcher::on_match_all(&args, &UNIX_STYLE, &info);
    assert_eq!(result, vec![0, 1, 2, 3, 4, 5]);
}

#[test]
fn test_multi_all_skips_non_matching_args() {
    // --val a --other b --val c  → only --val groups
    let mut info = PickerArgInfo::new();
    info.set_long("val");
    let args = make_args(&[
        ("--val", 0), ("a", 1),
        ("--other", 2), ("b", 3),
        ("--val", 4), ("c", 5),
    ]);
    let result = MultiArgMatcher::on_match_all(&args, &UNIX_STYLE, &info);
    assert_eq!(result, vec![0, 1, 4, 5]);
}

// on_match_all — named, eq mode

#[test]
fn test_multi_all_eq_mode() {
    // --val=a b → eq mode + one extra value
    let mut info = PickerArgInfo::new();
    info.set_long("val");
    let args = make_args(&[("--val=a", 0), ("b", 1)]);
    let result = MultiArgMatcher::on_match_all(&args, &UNIX_STYLE, &info);
    assert_eq!(result, vec![0, 1]);
}

#[test]
fn test_multi_all_eq_mode_no_extra() {
    // --val=a alone → just the eq flag
    let mut info = PickerArgInfo::new();
    info.set_long("val");
    let args = make_args(&[("--val=a", 0)]);
    let result = MultiArgMatcher::on_match_all(&args, &UNIX_STYLE, &info);
    assert_eq!(result, vec![0]);
}

#[test]
fn test_multi_all_mixed_eq_and_regular() {
    // --val=a b --val c d
    let mut info = PickerArgInfo::new();
    info.set_long("val");
    let args = make_args(&[
        ("--val=a", 0), ("b", 1), ("--val", 2), ("c", 3), ("d", 4),
    ]);
    let result = MultiArgMatcher::on_match_all(&args, &UNIX_STYLE, &info);
    assert_eq!(result, vec![0, 1, 2, 3, 4]);
}

// on_match_all — short flag

#[test]
fn test_multi_all_short_flag() {
    let mut info = PickerArgInfo::new();
    info.set_short('v');
    info.set_long("val");
    let args = make_args(&[("-v", 0), ("a", 1), ("b", 2)]);
    let result = MultiArgMatcher::on_match_all(&args, &UNIX_STYLE, &info);
    assert_eq!(result, vec![0, 1, 2]);
}

// on_match_all — end-of-options marker

#[test]
fn test_multi_all_stops_at_end_of_options() {
    // --val a -- b → stops before `--`
    let mut info = PickerArgInfo::new();
    info.set_long("val");
    let args = make_args(&[("--val", 0), ("a", 1), ("--", 2), ("b", 3)]);
    let result = MultiArgMatcher::on_match_all(&args, &UNIX_STYLE, &info);
    assert_eq!(result, vec![0, 1]);
}

#[test]
fn test_multi_all_flag_after_end_ignored() {
    let mut info = PickerArgInfo::new();
    info.set_long("val");
    let args = make_args(&[("--", 0), ("--val", 1), ("a", 2)]);
    let result = MultiArgMatcher::on_match_all(&args, &UNIX_STYLE, &info);
    assert!(result.is_empty());
}

// on_match_all — positional

#[test]
fn test_multi_all_positional() {
    let mut info = PickerArgInfo::new();
    info.set_positional(true);
    let args = make_args(&[("a.txt", 0), ("b.txt", 1)]);
    let result = MultiArgMatcher::on_match_all(&args, &UNIX_STYLE, &info);
    assert_eq!(result, vec![0, 1]);
}

#[test]
fn test_multi_all_positional_stops_at_end_of_options() {
    let mut info = PickerArgInfo::new();
    info.set_positional(true);
    let args = make_args(&[("a.txt", 0), ("--", 1), ("b.txt", 2)]);
    let result = MultiArgMatcher::on_match_all(&args, &UNIX_STYLE, &info);
    assert_eq!(result, vec![0]);
}
