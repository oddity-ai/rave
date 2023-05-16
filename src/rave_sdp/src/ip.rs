use std::net::IpAddr;

use crate::sdp::AddressType;

pub fn ip_addr_type(addr: &IpAddr) -> AddressType {
    match addr {
        IpAddr::V4(_) => AddressType::IpV4,
        IpAddr::V6(_) => AddressType::IpV6,
    }
}
