use crate::PickerArgResult::NotFound;
use crate::{SinglePickable, pickable_needed::*};

impl SinglePickable for String {
    fn pick_single(str: Option<&str>) -> PickerArgResult<Self> {
        match str {
            Some(str) => PickerArgResult::Parsed(str.to_string()),
            None => NotFound,
        }
    }
}
