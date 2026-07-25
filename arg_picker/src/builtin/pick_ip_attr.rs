use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use crate::SinglePickable;

impl SinglePickable for IpAddr {
    fn pick_single(str: Option<&str>) -> crate::PickerArgResult<Self> {
        match str {
            Some(s) => match s.parse::<IpAddr>() {
                Ok(addr) => crate::PickerArgResult::Parsed(addr),
                Err(_) => crate::PickerArgResult::NotFound,
            },
            None => crate::PickerArgResult::NotFound,
        }
    }
}

impl SinglePickable for Ipv4Addr {
    fn pick_single(str: Option<&str>) -> crate::PickerArgResult<Self> {
        match str {
            Some(s) => match s.parse::<Ipv4Addr>() {
                Ok(addr) => crate::PickerArgResult::Parsed(addr),
                Err(_) => crate::PickerArgResult::NotFound,
            },
            None => crate::PickerArgResult::NotFound,
        }
    }
}

impl SinglePickable for Ipv6Addr {
    fn pick_single(str: Option<&str>) -> crate::PickerArgResult<Self> {
        match str {
            Some(s) => match s.parse::<Ipv6Addr>() {
                Ok(addr) => crate::PickerArgResult::Parsed(addr),
                Err(_) => crate::PickerArgResult::NotFound,
            },
            None => crate::PickerArgResult::NotFound,
        }
    }
}
