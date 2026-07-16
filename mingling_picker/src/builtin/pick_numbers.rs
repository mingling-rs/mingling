use crate::{SinglePickable, pickable_needed::*};

macro_rules! impl_single_pickable_num {
    ($($t:ty),+ $(,)?) => {
        $(impl SinglePickable for $t {
            fn pick_single(str: Option<&str>) -> PickerArgResult<Self> {
                match str {
                    Some(s) => s.parse::<$t>().map(PickerArgResult::Parsed).unwrap_or(PickerArgResult::NotFound),
                    None => PickerArgResult::NotFound,
                }
            }
        })+
    };
}

impl_single_pickable_num! {
    i8, i16, i32, i64, i128, isize,
    u8, u16, u32, u64, u128, usize,
    f32, f64,
}
