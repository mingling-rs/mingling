use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6};

use crate::SinglePickable;

impl SinglePickable for SocketAddr {
    fn pick_single(str: Option<&str>) -> crate::PickerArgResult<Self> {
        str.map_or(crate::PickerArgResult::NotFound, |s| {
            s.parse::<Self>()
                .map_or(crate::PickerArgResult::NotFound, |addr| {
                    crate::PickerArgResult::Parsed(addr)
                })
        })
    }
}

impl SinglePickable for SocketAddrV4 {
    fn pick_single(str: Option<&str>) -> crate::PickerArgResult<Self> {
        str.map_or(crate::PickerArgResult::NotFound, |s| {
            s.parse::<Self>()
                .map_or(crate::PickerArgResult::NotFound, |addr| {
                    crate::PickerArgResult::Parsed(addr)
                })
        })
    }
}

impl SinglePickable for SocketAddrV6 {
    fn pick_single(str: Option<&str>) -> crate::PickerArgResult<Self> {
        str.map_or(crate::PickerArgResult::NotFound, |s| {
            s.parse::<Self>()
                .map_or(crate::PickerArgResult::NotFound, |addr| {
                    crate::PickerArgResult::Parsed(addr)
                })
        })
    }
}
