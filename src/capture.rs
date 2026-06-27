use std::net::Ipv6Addr;
use std::os::fd::OwnedFd;

use crate::cidr::Ipv6Cidr;
use crate::packet::parse_ipv6_endpoints;

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

pub fn capture_loop(fd: &OwnedFd, filter: Option<&Ipv6Cidr>) -> nix::Result<()> {
    use nix::sys::socket::{MsgFlags, recv};
    use std::os::fd::AsRawFd;

    let mut buf = vec![0u8; 65536];
    loop {
        let n = recv(fd.as_raw_fd(), &mut buf, MsgFlags::empty())?;
        if let Some((src, dst)) = parse_ipv6_endpoints(&buf[..n])
            && endpoint_in_filter(filter, src, dst)
        {
            println!("src={src} -> dst={dst}");
        }
    }
}

pub fn endpoint_in_filter(filter: Option<&Ipv6Cidr>, src: Ipv6Addr, dst: Ipv6Addr) -> bool {
    match filter {
        None => true,
        Some(cidr) => cidr.contains(src) || cidr.contains(dst),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_matches_all_when_no_cidr() {
        let src = "2001:db8::1".parse().unwrap();
        let dst = "fe80::1".parse().unwrap();
        assert!(endpoint_in_filter(None, src, dst));
    }

    #[test]
    fn filter_matches_when_either_endpoint_in_cidr() {
        let cidr: Ipv6Cidr = "2001:db8::/32".parse().unwrap();
        let inside = "2001:db8::1".parse().unwrap();
        let outside = "fe80::1".parse().unwrap();

        assert!(endpoint_in_filter(Some(&cidr), inside, outside));
        assert!(endpoint_in_filter(Some(&cidr), outside, inside));
        assert!(!endpoint_in_filter(Some(&cidr), outside, outside));
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
