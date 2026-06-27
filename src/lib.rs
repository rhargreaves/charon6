use std::net::Ipv6Addr;
use std::os::fd::OwnedFd;

/// Open an `AF_PACKET` socket that receives IPv6 packets on `device`.
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

/// Parse the source and destination IPv6 addresses from a packet buffer.
///
/// The buffer is expected to start at the IPv6 header (as delivered by an
/// `AF_PACKET`/`SOCK_DGRAM` socket, which strips the link-layer header).
pub fn parse_ipv6_endpoints(packet: &[u8]) -> Option<(Ipv6Addr, Ipv6Addr)> {
    let header: &[u8; 40] = packet.get(..40)?.try_into().ok()?;
    // High nibble of the first byte is the IP version; must be 6.
    if header[0] & 0xF0 != 0x60 {
        return None;
    }
    let src_bytes: [u8; 16] = header[8..24].try_into().ok()?;
    let dst_bytes: [u8; 16] = header[24..40].try_into().ok()?;
    Some((Ipv6Addr::from(src_bytes), Ipv6Addr::from(dst_bytes)))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Returns true if the current process holds CAP_NET_RAW in its effective set.
    fn has_net_raw() -> bool {
        const CAP_NET_RAW: u32 = 13;
        let status = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
        for line in status.lines() {
            if let Some(hex) = line.strip_prefix("CapEff:") {
                if let Ok(caps) = u64::from_str_radix(hex.trim(), 16) {
                    return caps & (1 << CAP_NET_RAW) != 0;
                }
            }
        }
        false
    }

    #[test]
    fn parses_src_and_dst_from_ipv6_header() {
        let mut packet = vec![0u8; 40];
        // Version 6 in the high nibble of byte 0.
        packet[0] = 0x60;
        // Source address occupies bytes 8..24.
        let src = Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1);
        packet[8..24].copy_from_slice(&src.octets());
        // Destination address occupies bytes 24..40.
        let dst = Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 2);
        packet[24..40].copy_from_slice(&dst.octets());

        assert_eq!(parse_ipv6_endpoints(&packet), Some((src, dst)));
    }

    #[test]
    fn rejects_non_ipv6_version() {
        let mut packet = vec![0u8; 40];
        // Version 4 in the high nibble: not an IPv6 packet.
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
}
