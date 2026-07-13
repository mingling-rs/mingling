use crate::{Pickable, PickerResult};
use std::path::PathBuf;

macro_rules! impl_pickable {
    ($($t:ty),*) => {
        $(
            impl Pickable for $t {
                fn pick(raw_str: &str) -> PickerResult<Self> {
                    raw_str.parse::<$t>().into()
                }
            }
        )*
    };
}

impl_pickable!(
    i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, usize, f32, f64, isize
);

impl Pickable for String {
    fn pick(raw_str: &str) -> PickerResult<Self> {
        PickerResult::Parsed(raw_str.to_string())
    }
}

impl Pickable for PathBuf {
    fn pick(raw_str: &str) -> PickerResult<Self> {
        PickerResult::Parsed(PathBuf::from(raw_str))
    }
}

impl<T: Pickable> Pickable for Option<T> {
    fn pick(raw_str: &str) -> PickerResult<Self> {
        if raw_str.is_empty() {
            PickerResult::Parsed(None)
        } else {
            match T::pick(raw_str) {
                PickerResult::Parsed(v) => PickerResult::Parsed(Some(v)),
                _ => PickerResult::Parsed(None),
            }
        }
    }
}
