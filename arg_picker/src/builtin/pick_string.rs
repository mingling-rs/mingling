// Doc Not Optimize
use crate::PickerArgResult::NotFound;
use crate::{SinglePickable, pickable_needed::*};

impl SinglePickable for String {
    fn pick_single(str: Option<&str>) -> PickerArgResult<Self> {
        str.map_or(NotFound, |str| PickerArgResult::Parsed(str.to_string()))
    }
}
