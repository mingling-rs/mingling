use std::path::PathBuf;

use crate::{
    PickerArgResult::{NotFound, Parsed},
    SinglePickable,
};

impl SinglePickable for PathBuf {
    fn pick_single(str: Option<&str>) -> crate::PickerArgResult<Self> {
        str.map_or(NotFound, |str| {
            just_fmt::fmt_path_str(str).map_or(NotFound, |formated| Parsed(Self::from(formated)))
        })
    }
}
