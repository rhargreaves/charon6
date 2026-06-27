use std::net::Ipv6Addr;
use std::os::fd::OwnedFd;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ipv6Cidr {
    network: Ipv6Addr,
    prefix_len: u8,
}

const IPV6_BITS: u8 = 128;

impl Ipv6Cidr {
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

impl FromStr for Ipv6Cidr {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (address, prefix) = s.split_once('/').ok_or(())?;
        let network = address.parse::<Ipv6Addr>().map_err(|_| ())?;
        let prefix_len = prefix.parse::<u8>().map_err(|_| ())?;
        if prefix_len > IPV6_BITS {
            return Err(());
        }
        Ok(Ipv6Cidr {
            network,
            prefix_len,
        })
    }
}

pub fn open_ipv6_packet_socket(device: &str) -> nix::Result<OwnedFd> {
    use nix::sys::socket::{
        AddressFamily, SockFlag, SockProtocol, SockType, setsockopt, socket, sockopt::BindToDevice,
    };

    let fd = socket(
        AddressFamily::Packet,
        SockType::Datagram,
        SockFlag::empty(),
        SockProtocol::EthIpv6,
    )?;
    setsockopt(&fd, BindToDevice, &std::ffi::OsString::from(device))?;
    Ok(fd)
}

pub fn capture_loop(fd: &OwnedFd) -> nix::Result<()> {
    use nix::sys::socket::{MsgFlags, recv};
    use std::os::fd::AsRawFd;

    let mut buf = vec![0u8; 65536];
    loop {
        let n = recv(fd.as_raw_fd(), &mut buf, MsgFlags::empty())?;
        if let Some((src, dst)) = parse_ipv6_endpoints(&buf[..n]) {
            println!("src={src} -> dst={dst}");
        }
    }
}

pub fn endpoint_in_filter(filter: Option<&Ipv6Cidr>, _src: Ipv6Addr, _dst: Ipv6Addr) -> bool {
    filter.is_none()
}

pub fn parse_ipv6_endpoints(packet: &[u8]) -> Option<(Ipv6Addr, Ipv6Addr)> {
    const IPV6_HEADER_LEN: usize = 40;
    const IP_VERSION_MASK: u8 = 0xF0;
    const IPV6_VERSION: u8 = 0x60;
    const SRC_OFFSET: usize = 8;
    const DST_OFFSET: usize = 24;
    const ADDR_LEN: usize = 16;

    let header: &[u8; IPV6_HEADER_LEN] = packet.get(..IPV6_HEADER_LEN)?.try_into().ok()?;
    if header[0] & IP_VERSION_MASK != IPV6_VERSION {
        return None;
    }
    let src_bytes: [u8; ADDR_LEN] = header[SRC_OFFSET..SRC_OFFSET + ADDR_LEN].try_into().ok()?;
    let dst_bytes: [u8; ADDR_LEN] = header[DST_OFFSET..DST_OFFSET + ADDR_LEN].try_into().ok()?;
    Some((Ipv6Addr::from(src_bytes), Ipv6Addr::from(dst_bytes)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cidr_contains_address_within_prefix() {
        let cidr: Ipv6Cidr = "2001:db8::/32".parse().unwrap();
        assert!(cidr.contains("2001:db8::1".parse().unwrap()));
        assert!(!cidr.contains("2001:db9::1".parse().unwrap()));
    }

    #[test]
    fn filter_matches_all_when_no_cidr() {
        let src = "2001:db8::1".parse().unwrap();
        let dst = "fe80::1".parse().unwrap();
        assert!(endpoint_in_filter(None, src, dst));
    }

    #[test]
    fn rejects_invalid_cidr() {
        assert!("2001:db8::".parse::<Ipv6Cidr>().is_err());
        assert!("not_an_addr/32".parse::<Ipv6Cidr>().is_err());
        assert!("2001:db8::/129".parse::<Ipv6Cidr>().is_err());
    }

    #[test]
    fn parses_src_and_dst_from_ipv6_header() {
        let mut packet = vec![0u8; 40];
        packet[0] = 0x60;
        let src = Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1);
        packet[8..24].copy_from_slice(&src.octets());
        let dst = Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 2);
        packet[24..40].copy_from_slice(&dst.octets());

        assert_eq!(parse_ipv6_endpoints(&packet), Some((src, dst)));
    }

    #[test]
    fn rejects_non_ipv6_version() {
        let mut packet = vec![0u8; 40];
        packet[0] = 0x40;

        assert_eq!(parse_ipv6_endpoints(&packet), None);
    }

    #[test]
    fn opens_packet_socket_on_loopback() {
        assert!(open_ipv6_packet_socket("lo").is_ok());
    }

    #[test]
    fn errors_on_nonexistent_device() {
        assert!(open_ipv6_packet_socket("no_such_dev0").is_err());
    }
}
