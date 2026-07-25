use std::path::PathBuf;

use crate::{
    PickerArgResult::{NotFound, Parsed},
    SinglePickable,
};

impl SinglePickable for PathBuf {
    fn pick_single(str: Option<&str>) -> crate::PickerArgResult<Self> {
        match str {
            Some(str) => match just_fmt::fmt_path_str(str) {
                Ok(formated) => Parsed(PathBuf::from(formated)),
                Err(_) => NotFound,
            },
            None => NotFound,
        }
    }
}
