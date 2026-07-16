use mingling_picker::value::{Flag, VecUntil};
use mingling_picker::{IntoPicker, macros::arg};

#[test]
fn test_vec_until_i16_named() {
    let (nums, rest): (VecUntil<i16>, Flag) = vec!["--nums", "1", "2", "3", "abc"]
        .to_picker()
        .pick(&arg![nums: VecUntil<i16>])
        .pick(&arg![rest: Flag])
        .unwrap();
    assert_eq!(*nums, vec![1i16, 2, 3]);
    assert_eq!(rest, Flag::Inactive);
}

#[test]
fn test_vec_until_i16_stops_at_non_number() {
    let nums: VecUntil<i16> = vec!["--nums", "42", "abc", "100"]
        .to_picker()
        .pick(&arg![nums: VecUntil<i16>])
        .unwrap();
    assert_eq!(*nums, vec![42i16]);
}

#[test]
fn test_vec_until_i16_empty() {
    let nums: VecUntil<i16> = vec!["--nums"]
        .to_picker()
        .pick(&arg![nums: VecUntil<i16>])
        .unwrap();
    assert!(nums.is_empty());
}

#[test]
fn test_vec_until_i16_stops_at_next_flag() {
    let nums: VecUntil<i16> = vec!["--nums", "1", "2", "--other", "3"]
        .to_picker()
        .pick(&arg![nums: VecUntil<i16>])
        .unwrap();
    assert_eq!(*nums, vec![1i16, 2]);
}

#[test]
fn test_vec_until_i16_stops_at_end_of_options() {
    let nums: VecUntil<i16> = vec!["--nums", "1", "2", "--", "3"]
        .to_picker()
        .pick(&arg![nums: VecUntil<i16>])
        .unwrap();
    assert_eq!(*nums, vec![1i16, 2]);
}

// Two VecUntil<T> with different boundary behaviours

#[test]
fn test_vec_until_f64_and_i16_positional() {
    // Both are positional VecUntil. f64 takes all valid floats,
    // i16 takes what's left. But note: "1", "2", "3" are also valid
    // f64 values, so f64's check_boundary never fires — it consumes
    // everything, and i16 gets nothing.
    let (floats, ints): (VecUntil<f64>, VecUntil<i16>) = vec!["1.5", "2.5", "3.5", "1", "2", "3"]
        .to_picker()
        .pick(&arg![VecUntil<f64>])
        .pick(&arg![VecUntil<i16>])
        .unwrap();
    assert_eq!(*floats, vec![1.5, 2.5, 3.5]);
    assert_eq!(*ints, vec![1i16, 2, 3]);
}
