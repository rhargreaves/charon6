use std::net::Ipv6Addr;
use std::os::fd::OwnedFd;

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

    fn has_net_raw() -> bool {
        const CAP_NET_RAW: u32 = 13;
        let status = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
        for line in status.lines() {
            if let Some(hex) = line.strip_prefix("CapEff:")
                && let Ok(caps) = u64::from_str_radix(hex.trim(), 16)
            {
                return caps & (1 << CAP_NET_RAW) != 0;
            }
        }
        false
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
        if !has_net_raw() {
            eprintln!("skipping opens_packet_socket_on_loopback: missing CAP_NET_RAW");
            return;
        }
        assert!(open_ipv6_packet_socket("lo").is_ok());
    }

    #[test]
    fn errors_on_nonexistent_device() {
        if !has_net_raw() {
            eprintln!("skipping errors_on_nonexistent_device: missing CAP_NET_RAW");
            return;
        }
        assert!(open_ipv6_packet_socket("no_such_dev0").is_err());
    }
}
