use std::fmt;
use std::net::Ipv6Addr;
use std::str::FromStr;

const IPV6_BITS: u8 = 128;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ipv6Cidr {
    network: Ipv6Addr,
    prefix_len: u8,
}

impl Ipv6Cidr {
    pub fn network(&self) -> Ipv6Addr {
        self.network
    }

    pub fn contains(&self, address: Ipv6Addr) -> bool {
        let mask = self.mask();
        (u128::from(address) & mask) == (u128::from(self.network) & mask)
    }

    fn mask(&self) -> u128 {
        if self.prefix_len == 0 {
            0
        } else {
            u128::MAX << (IPV6_BITS - self.prefix_len)
        }
    }
}

impl fmt::Display for Ipv6Cidr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.network, self.prefix_len)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseCidrError;

impl fmt::Display for ParseCidrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid IPv6 CIDR (expected <address>/<prefix>)")
    }
}

impl std::error::Error for ParseCidrError {}

impl FromStr for Ipv6Cidr {
    type Err = ParseCidrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (address, prefix) = s.split_once('/').ok_or(ParseCidrError)?;
        let network = address.parse::<Ipv6Addr>().map_err(|_| ParseCidrError)?;
        let prefix_len = prefix.parse::<u8>().map_err(|_| ParseCidrError)?;
        if prefix_len > IPV6_BITS {
            return Err(ParseCidrError);
        }
        Ok(Ipv6Cidr {
            network,
            prefix_len,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::{addr, cidr};

    #[test]
    fn contains_address_within_prefix() {
        let c = cidr("2001:db8::/32");
        assert!(c.contains(addr("2001:db8::1")));
        assert!(!c.contains(addr("2001:db9::1")));
    }

    #[test]
    fn rejects_invalid_cidr() {
        assert!("2001:db8::".parse::<Ipv6Cidr>().is_err());
        assert!("not_an_addr/32".parse::<Ipv6Cidr>().is_err());
        assert!("2001:db8::/129".parse::<Ipv6Cidr>().is_err());
    }

    #[test]
    fn prefix_zero_matches_all_addresses() {
        let c = cidr("::/0");
        assert!(c.contains(addr("::1")));
        assert!(c.contains(addr("fe80::1")));
        assert!(c.contains(addr("2001:db8::1")));
    }

    #[test]
    fn prefix_128_matches_only_exact_address() {
        let c = cidr("2001:db8::1/128");
        assert!(c.contains(addr("2001:db8::1")));
        assert!(!c.contains(addr("2001:db8::2")));
    }
}
