use mingling_picker::value::Flag;
use mingling_picker::{IntoPicker, macros::arg};

// Same flag name, different Pickable types
//
// PickerArgAttr priority: Flag < Single
// So Single and Multi should parse BEFORE Flag when sharing
// the same flag name.

#[test]
fn test_single_takes_flag_when_sharing_name() {
    // --name Alice: String consumes it, Flag sees nothing.
    let (name, verbose): (String, Flag) = vec!["--name", "Alice"]
        .to_picker()
        .pick(&arg![name: String]) // Single, parsed first
        .pick(&arg![name: Flag]) // Flag, parsed second, nothing left
        .unwrap();
    assert_eq!(name, "Alice");
    assert_eq!(verbose, Flag::Inactive);
}

#[test]
fn test_flag_only_triggers_when_single_missing() {
    // --verbose: only Flag matches, String gets default.
    let (name, verbose): (String, Flag) = vec!["--verbose"]
        .to_picker()
        .pick(&arg![name: String]) // Single, no match → default ""
        .or_default()
        .pick(&arg![name: Flag]) // Flag, no --name in args → Inactive
        .unwrap();
    assert_eq!(name, "");
    assert_eq!(verbose, Flag::Inactive);
}

#[test]
fn test_flag_gets_leftovers_after_single_consumes_value() {
    // --name Alice --name: String takes "--name Alice", Flag takes "--name".
    let (name, verbose): (String, Flag) = vec!["--name", "Alice", "--name"]
        .to_picker()
        .pick(&arg![name: String]) // Single: tag [0, 1], consumes --name Alice
        .pick(&arg![name: Flag]) // Flag: tags position 2 (--name)
        .unwrap();
    assert_eq!(name, "Alice");
    assert_eq!(
        verbose,
        Flag::Active,
        "Flag should see --name at position 2 after String consumed positions 0-1"
    );
}

#[test]
fn test_short_flag_sharing_same_letter() {
    // -n Alice: String takes it, Flag misses.
    let (name, verbose): (String, Flag) = vec!["-n", "Alice"]
        .to_picker()
        .pick(&arg![name: String, 'n']) // Single, parsed first
        .pick(&arg![name: Flag, 'n']) // Flag, parsed second
        .unwrap();
    assert_eq!(name, "Alice");
    assert_eq!(verbose, Flag::Inactive);
}

#[test]
fn test_flag_captures_remaining_after_single_partial_consume() {
    // --name Alice --verbose: String takes --name Alice, Flag takes --verbose.
    let (name, verbose): (String, Flag) = vec!["--name", "Alice", "--verbose"]
        .to_picker()
        .pick(&arg![name: String]) // Single: tag [0, 1]
        .pick(&arg![verbose: Flag]) // Flag: tag [2]
        .unwrap();
    assert_eq!(name, "Alice");
    assert_eq!(verbose, Flag::Active);
}

#[test]
fn test_single_skips_already_claimed_positions() {
    // --verbose --name Alice: Flag takes --verbose, String takes --name Alice.
    let (verbose, name): (Flag, String) = vec!["--verbose", "--name", "Alice"]
        .to_picker()
        .pick(&arg![verbose: Flag]) // Flag: tag [0]
        .pick(&arg![name: String]) // Single: tag [1, 2]
        .unwrap();
    assert_eq!(verbose, Flag::Active);
    assert_eq!(name, "Alice");
}
