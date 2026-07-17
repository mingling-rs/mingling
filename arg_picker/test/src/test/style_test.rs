use arg_picker::parselib::{
    build_possible_flags, FlagMatcher, Matcher, ParserStyle, ParserStyleNamingCase,
    POWERSHELL_STYLE, UNIX_STYLE, WINDOWS_STYLE,
};
use arg_picker::PickerArgInfo;

use crate::make_masked;

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
    assert_eq!(flags, vec!["/Verbose"]);
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

#[test]
fn test_windows_style_match() {
    // Windows style: /verbose (case insensitive)
    let mut info = PickerArgInfo::new();
    info.set_long("verbose");

    let args = vec![make_masked("/verbose", 0)];
    let result = FlagMatcher::on_match_one(&args, &WINDOWS_STYLE, &info);
    assert_eq!(result, Some(0));
}

#[test]
fn test_windows_style_match_case_insensitive() {
    // Windows style is case-insensitive: /VERBOSE should match "verbose"
    let mut info = PickerArgInfo::new();
    info.set_long("verbose");

    let args = vec![make_masked("/VERBOSE", 0)];
    let result = FlagMatcher::on_match_one(&args, &WINDOWS_STYLE, &info);
    assert_eq!(result, Some(0));
}

#[test]
fn test_windows_style_no_match_on_unrelated_flag() {
    // Different flag should not match
    let mut info = PickerArgInfo::new();
    info.set_long("verbose");

    let args = vec![make_masked("/output", 0)];
    let result = FlagMatcher::on_match_one(&args, &WINDOWS_STYLE, &info);
    assert_eq!(result, None);
}

#[test]
fn test_powershell_style_match() {
    // PowerShell style: -Verbose (case insensitive)
    let mut info = PickerArgInfo::new();
    info.set_long("Verbose");

    let args = vec![make_masked("-Verbose", 0)];
    let result = FlagMatcher::on_match_one(&args, &POWERSHELL_STYLE, &info);
    assert_eq!(result, Some(0));
}

#[test]
fn test_powershell_style_match_case_insensitive() {
    let mut info = PickerArgInfo::new();
    info.set_long("Verbose");

    let args = vec![make_masked("-VERBOSE", 0)];
    let result = FlagMatcher::on_match_one(&args, &POWERSHELL_STYLE, &info);
    assert_eq!(result, Some(0));
}

#[test]
fn test_unix_style_case_sensitive_no_match() {
    // UNIX style is case-sensitive: --VERBOSE should NOT match --verbose
    let mut info = PickerArgInfo::new();
    info.set_long("verbose");

    let args = vec![make_masked("--VERBOSE", 0)];
    let result = FlagMatcher::on_match_one(&args, &UNIX_STYLE, &info);
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
    let result = FlagMatcher::on_match_all(&args, &WINDOWS_STYLE, &info);
    assert_eq!(result, vec![0, 2]);
}

#[test]
fn test_windows_style_match_after_end_of_options() {
    // end_of_options is always "--" regardless of style
    // Flags after -- should not match
    let mut info = PickerArgInfo::new();
    info.set_long("verbose");

    let args = vec![
        make_masked("/verbose", 0),
        make_masked("--", 1),
        make_masked("/verbose", 2),
    ];
    let result = FlagMatcher::on_match_all(&args, &WINDOWS_STYLE, &info);
    // end_of_options is always "--" regardless of style
    assert_eq!(result, vec![0]);
}

// Naming case conversion

/// A Unix-like style with kebab-case naming.
const KEBAB_STYLE: ParserStyle<'static> = ParserStyle {
    naming_case: ParserStyleNamingCase::Kebab,
    ..UNIX_STYLE
};

#[test]
fn test_kebab_case_naming_for_multiword_flag() {
    // flag name `flag_a` → Kebab → `flag-a` → `--flag-a`
    let mut info = PickerArgInfo::new();
    info.set_long("flag_a");

    let args = vec![make_masked("--flag-a", 0)];
    let result = FlagMatcher::on_match_one(&args, &KEBAB_STYLE, &info);
    assert_eq!(
        result,
        Some(0),
        "--flag-a should match flag_a via kebab-case conversion"
    );
}

#[test]
fn test_snake_case_should_not_match_as_long_flag() {
    // With kebab naming, `--flag_a` (snake) should NOT match `flag_a`
    let mut info = PickerArgInfo::new();
    info.set_long("flag_a");

    let args = vec![make_masked("--flag_a", 0)];
    let result = FlagMatcher::on_match_one(&args, &KEBAB_STYLE, &info);
    assert_eq!(
        result, None,
        "--flag_a should NOT match in kebab-style context"
    );
}

#[test]
fn test_kebab_case_naming_for_unix_style() {
    // UNIX_STYLE now uses Kebab case: `flag_a` → `flag-a` → `--flag-a`
    let mut info = PickerArgInfo::new();
    info.set_long("flag_a");

    let args = vec![make_masked("--flag-a", 0)];
    let result = FlagMatcher::on_match_one(&args, &UNIX_STYLE, &info);
    assert_eq!(
        result,
        Some(0),
        "--flag-a should match with kebab-style UNIX_STYLE"
    );
}

#[test]
fn test_snake_case_rejected_by_unix_style() {
    // UNIX_STYLE uses Kebab: `--flag_a` (snake) should NOT match
    let mut info = PickerArgInfo::new();
    info.set_long("flag_a");

    let args = vec![make_masked("--flag_a", 0)];
    let result = FlagMatcher::on_match_one(&args, &UNIX_STYLE, &info);
    assert_eq!(
        result, None,
        "--flag_a should NOT match under kebab-style UNIX_STYLE"
    );
}

#[test]
fn test_powershell_pascal_case_naming() {
    // POWERSHELL_STYLE uses Pascal case: `verbose` → `Verbose` → `-Verbose`
    let mut info = PickerArgInfo::new();
    info.set_long("verbose");

    let args = vec![make_masked("-Verbose", 0)];
    let result = FlagMatcher::on_match_one(&args, &POWERSHELL_STYLE, &info);
    assert_eq!(
        result,
        Some(0),
        "-Verbose should match verbose via Pascal case"
    );
}

#[test]
fn test_flag_naming_kebab_matches_my_name() {
    // UNIX_STYLE now uses Kebab: `my_name` → `my-name` → `--my-name`
    let mut info = PickerArgInfo::new();
    info.set_long("my_name");

    let args = vec![make_masked("--my-name", 0)];
    let result = FlagMatcher::on_match_one(&args, &UNIX_STYLE, &info);
    assert_eq!(
        result,
        Some(0),
        "--my-name should match via kebab conversion"
    );
}

#[test]
fn test_flag_naming_kebab_rejects_my_name_underscore() {
    // Kebab style: `--my_name` (snake) should NOT match
    let mut info = PickerArgInfo::new();
    info.set_long("my_name");

    let args = vec![make_masked("--my_name", 0)];
    let result = FlagMatcher::on_match_one(&args, &UNIX_STYLE, &info);
    assert_eq!(
        result, None,
        "--my_name should NOT match under kebab-style UNIX_STYLE"
    );
}

#[test]
fn test_flag_naming_pascal_matches_my_name() {
    // `my_name` under Pascal → `MyName` → `-MyName`
    let mut info = PickerArgInfo::new();
    info.set_long("my_name");

    let args = vec![make_masked("-MyName", 0)];
    let result = FlagMatcher::on_match_one(&args, &POWERSHELL_STYLE, &info);
    assert_eq!(
        result,
        Some(0),
        "-MyName should match via Pascal conversion"
    );
}

#[test]
fn test_flag_naming_pascal_matches_lowercase() {
    // PowerShell is case-insensitive: `-myname` should also match
    let mut info = PickerArgInfo::new();
    info.set_long("my_name");

    let args = vec![make_masked("-myname", 0)];
    let result = FlagMatcher::on_match_one(&args, &POWERSHELL_STYLE, &info);
    assert_eq!(
        result,
        Some(0),
        "-myname (lowercase) should match via case-insensitive Pascal"
    );
}

#[test]
fn test_flag_naming_pascal_rejects_my_name_underscore() {
    // Pascal style: `-my_name` (underscore) should NOT match
    let mut info = PickerArgInfo::new();
    info.set_long("my_name");

    let args = vec![make_masked("-my_name", 0)];
    let result = FlagMatcher::on_match_one(&args, &POWERSHELL_STYLE, &info);
    assert_eq!(
        result, None,
        "-my_name should NOT match under Pascal-style naming"
    );
}
