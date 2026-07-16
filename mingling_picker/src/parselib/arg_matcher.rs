use crate::matcher_needed::*;

pub struct ArgMatcher;

impl Matcher for ArgMatcher {
    fn on_match_one(
        _args: &[MaskedArg],
        _style: &ParserStyle,
        _arg_info: &PickerArgInfo,
    ) -> Option<usize> {
        todo!()
    }

    fn on_match_all(
        _args: &[MaskedArg],
        _style: &ParserStyle,
        _arg_info: &PickerArgInfo,
    ) -> Vec<usize> {
        todo!()
    }
}
