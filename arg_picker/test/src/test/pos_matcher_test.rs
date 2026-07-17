use arg_picker::parselib::{MaskedArg, Matcher, PositionalMatcher, UNIX_STYLE, WINDOWS_STYLE};
use arg_picker::PickerArgInfo;

fn make_args<'a>(pairs: &'a [(&'a str, usize)]) -> Vec<MaskedArg<'a>> {
    pairs
        .iter()
        .map(|&(raw, idx)| MaskedArg { raw, raw_idx: idx })
        .collect()
}

// on_match_one — basic

#[test]
fn test_pos_one_takes_first_non_flag() {
    let info = PickerArgInfo::new();
    let args = make_args(&[("--verbose", 0), ("file.txt", 1)]);
    let result = PositionalMatcher::on_match_one(&args, &UNIX_STYLE, &info);
    assert_eq!(result, Some(1));
}

#[test]
fn test_pos_one_takes_first_if_no_flag() {
    let info = PickerArgInfo::new();
    let args = make_args(&[("file.txt", 0)]);
    let result = PositionalMatcher::on_match_one(&args, &UNIX_STYLE, &info);
    assert_eq!(result, Some(0));
}

#[test]
fn test_pos_one_all_flags_returns_none() {
    let info = PickerArgInfo::new();
    let args = make_args(&[("--verbose", 0), ("--name", 1)]);
    let result = PositionalMatcher::on_match_one(&args, &UNIX_STYLE, &info);
    assert_eq!(result, None);
}

#[test]
fn test_pos_one_empty_returns_none() {
    let info = PickerArgInfo::new();
    let args = vec![];
    let result = PositionalMatcher::on_match_one(&args, &UNIX_STYLE, &info);
    assert_eq!(result, None);
}

// on_match_one — after `--`

#[test]
fn test_pos_one_after_end_takes_even_flag_like() {
    // After `--`, accept everything including `--verbose`.
    let info = PickerArgInfo::new();
    let args = make_args(&[("--", 0), ("--verbose", 1), ("file.txt", 2)]);
    let result = PositionalMatcher::on_match_one(&args, &UNIX_STYLE, &info);
    assert_eq!(result, Some(1));
}

#[test]
fn test_pos_one_only_end_returns_none() {
    let info = PickerArgInfo::new();
    let args = make_args(&[("--", 0)]);
    let result = PositionalMatcher::on_match_one(&args, &UNIX_STYLE, &info);
    assert_eq!(result, None);
}

// on_match_one — Windows style prefix

#[test]
fn test_pos_one_windows_skips_slash_prefix() {
    let info = PickerArgInfo::new();
    let args = make_args(&[("/Verbose", 0), ("file.txt", 1)]);
    let result = PositionalMatcher::on_match_one(&args, &WINDOWS_STYLE, &info);
    assert_eq!(result, Some(1));
}

// on_match_all — basic

#[test]
fn test_pos_all_collects_non_flags() {
    let info = PickerArgInfo::new();
    let args = make_args(&[("--verbose", 0), ("a.txt", 1), ("--name", 2), ("b.txt", 3)]);
    let result = PositionalMatcher::on_match_all(&args, &UNIX_STYLE, &info);
    assert_eq!(result, vec![1, 3]);
}

#[test]
fn test_pos_all_only_flags_returns_empty() {
    let info = PickerArgInfo::new();
    let args = make_args(&[("--a", 0), ("--b", 1)]);
    let result = PositionalMatcher::on_match_all(&args, &UNIX_STYLE, &info);
    assert!(result.is_empty());
}

#[test]
fn test_pos_all_empty() {
    let info = PickerArgInfo::new();
    let args = vec![];
    let result = PositionalMatcher::on_match_all(&args, &UNIX_STYLE, &info);
    assert!(result.is_empty());
}

// on_match_all — after `--`

#[test]
fn test_pos_all_after_end_accepts_everything() {
    // After `--`, even `--verbose` is accepted as positional.
    let info = PickerArgInfo::new();
    let args = make_args(&[
        ("--verbose", 0),
        ("a.txt", 1),
        ("--", 2),
        ("--flag", 3),
        ("b.txt", 4),
    ]);
    let result = PositionalMatcher::on_match_all(&args, &UNIX_STYLE, &info);
    assert_eq!(result, vec![1, 3, 4]);
}

#[test]
fn test_pos_all_only_after_end() {
    let info = PickerArgInfo::new();
    let args = make_args(&[("--", 0), ("arg1", 1), ("--arg2", 2)]);
    let result = PositionalMatcher::on_match_all(&args, &UNIX_STYLE, &info);
    assert_eq!(result, vec![1, 2]);
}

// on_match_all — mixed before/after `--`

#[test]
fn test_pos_all_mixed() {
    let info = PickerArgInfo::new();
    let args = make_args(&[
        ("infile", 0),
        ("--verbose", 1),
        ("--", 2),
        ("outfile", 3),
        ("--extra", 4),
    ]);
    let result = PositionalMatcher::on_match_all(&args, &UNIX_STYLE, &info);
    // Before `--`: "infile" (skip --verbose).
    // After `--`: everything — "outfile", "--extra".
    assert_eq!(result, vec![0, 3, 4]);
}
