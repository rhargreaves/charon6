use std::net::Ipv6Addr;

use crate::cidr::Ipv6Cidr;

pub const IPV6_DOC_CIDR: &str = "2001:db8::/64";

pub fn cidr(s: &str) -> Ipv6Cidr {
    s.parse().unwrap()
}

pub fn addr(s: &str) -> Ipv6Addr {
    s.parse().unwrap()
}

pub fn doc_cidr() -> Ipv6Cidr {
    cidr(IPV6_DOC_CIDR)
}
