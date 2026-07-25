use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6};

use crate::SinglePickable;

impl SinglePickable for SocketAddr {
    fn pick_single(str: Option<&str>) -> crate::PickerArgResult<Self> {
        match str {
            Some(s) => match s.parse::<SocketAddr>() {
                Ok(addr) => crate::PickerArgResult::Parsed(addr),
                Err(_) => crate::PickerArgResult::NotFound,
            },
            None => crate::PickerArgResult::NotFound,
        }
    }
}

impl SinglePickable for SocketAddrV4 {
    fn pick_single(str: Option<&str>) -> crate::PickerArgResult<Self> {
        match str {
            Some(s) => match s.parse::<SocketAddrV4>() {
                Ok(addr) => crate::PickerArgResult::Parsed(addr),
                Err(_) => crate::PickerArgResult::NotFound,
            },
            None => crate::PickerArgResult::NotFound,
        }
    }
}

impl SinglePickable for SocketAddrV6 {
    fn pick_single(str: Option<&str>) -> crate::PickerArgResult<Self> {
        match str {
            Some(s) => match s.parse::<SocketAddrV6>() {
                Ok(addr) => crate::PickerArgResult::Parsed(addr),
                Err(_) => crate::PickerArgResult::NotFound,
            },
            None => crate::PickerArgResult::NotFound,
        }
    }
}
