use std::net::Ipv6Addr;

use crate::cidr::Ipv6Cidr;

pub fn cidr(s: &str) -> Ipv6Cidr {
    s.parse().unwrap()
}

pub fn addr(s: &str) -> Ipv6Addr {
    s.parse().unwrap()
}
