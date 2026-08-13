// Doc Not Optimize
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use crate::SinglePickable;

impl SinglePickable for IpAddr {
    fn pick_single(str: Option<&str>) -> crate::PickerArgResult<Self> {
        str.map_or(crate::PickerArgResult::NotFound, |s| {
            s.parse::<Self>()
                .map_or(crate::PickerArgResult::NotFound, |addr| {
                    crate::PickerArgResult::Parsed(addr)
                })
        })
    }
}

impl SinglePickable for Ipv4Addr {
    fn pick_single(str: Option<&str>) -> crate::PickerArgResult<Self> {
        str.map_or(crate::PickerArgResult::NotFound, |s| {
            s.parse::<Self>()
                .map_or(crate::PickerArgResult::NotFound, |addr| {
                    crate::PickerArgResult::Parsed(addr)
                })
        })
    }
}

impl SinglePickable for Ipv6Addr {
    fn pick_single(str: Option<&str>) -> crate::PickerArgResult<Self> {
        str.map_or(crate::PickerArgResult::NotFound, |s| {
            s.parse::<Self>()
                .map_or(crate::PickerArgResult::NotFound, |addr| {
                    crate::PickerArgResult::Parsed(addr)
                })
        })
    }
}
