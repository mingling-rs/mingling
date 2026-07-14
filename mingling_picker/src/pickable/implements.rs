use crate::{Pickable, PickerFlag, PickerResult, PickerTag};

macro_rules! impl_pickable {
    ($($t:ty),*) => {
        $(impl<'a> Pickable<'a> for $t {
            fn tag(flag: &'a PickerFlag<'a, Self>) -> PickerTag<'a> {
                flag.into()
            }

            fn pick(raw_strs: &[&str]) -> PickerResult<Self> {
                raw_strs
                    .first()
                    .and_then(|s| s.parse().ok())
                    .map(PickerResult::Parsed)
                    .unwrap_or(PickerResult::NotFound)
            }
        })*
    };
}

impl_pickable!(
    i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, usize, f32, f64, isize
);

impl<'a> Pickable<'a> for String {
    fn tag(flag: &'a PickerFlag<'a, Self>) -> PickerTag<'a> {
        let mut tag: PickerTag = flag.into();
        tag.set_optional(false);
        tag.set_multi(false);
        tag
    }

    fn pick(raw_strs: &[&str]) -> PickerResult<Self> {
        raw_strs
            .first()
            .map(|s| PickerResult::Parsed(s.to_string()))
            .unwrap_or(PickerResult::NotFound)
    }
}
