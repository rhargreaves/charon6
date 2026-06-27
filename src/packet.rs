use std::net::Ipv6Addr;

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
}
