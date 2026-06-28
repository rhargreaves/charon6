use std::net::Ipv6Addr;

const IPV6_HEADER_LEN: usize = 40;
const IP_VERSION_MASK: u8 = 0xF0;
const IPV6_VERSION: u8 = 0x60;
const NEXT_HEADER_OFFSET: usize = 6;
const SRC_OFFSET: usize = 8;
const DST_OFFSET: usize = 24;
const ADDR_LEN: usize = 16;
const UDP_DST_PORT_OFFSET: usize = IPV6_HEADER_LEN + 2;

pub(crate) const PROTO_UDP: u8 = 17;
pub(crate) const PROTO_ICMPV6: u8 = 58;

pub(crate) struct PacketInfo {
    pub src: Ipv6Addr,
    pub dst: Ipv6Addr,
    pub next_header: u8,
    pub udp_dst_port: Option<u16>,
}

pub(crate) fn parse_ipv6_packet(packet: &[u8]) -> Option<PacketInfo> {
    let header: &[u8; IPV6_HEADER_LEN] = packet.get(..IPV6_HEADER_LEN)?.try_into().ok()?;
    if header[0] & IP_VERSION_MASK != IPV6_VERSION {
        return None;
    }
    let src_bytes: [u8; ADDR_LEN] = header[SRC_OFFSET..SRC_OFFSET + ADDR_LEN].try_into().ok()?;
    let dst_bytes: [u8; ADDR_LEN] = header[DST_OFFSET..DST_OFFSET + ADDR_LEN].try_into().ok()?;
    let next_header = header[NEXT_HEADER_OFFSET];

    let udp_dst_port = if next_header == PROTO_UDP {
        packet
            .get(UDP_DST_PORT_OFFSET..UDP_DST_PORT_OFFSET + 2)
            .map(|b| u16::from_be_bytes([b[0], b[1]]))
    } else {
        None
    };

    Some(PacketInfo {
        src: Ipv6Addr::from(src_bytes),
        dst: Ipv6Addr::from(dst_bytes),
        next_header,
        udp_dst_port,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ipv6_packet(next_header: u8, payload: &[u8]) -> Vec<u8> {
        let mut packet = vec![0u8; IPV6_HEADER_LEN + payload.len()];
        packet[0] = IPV6_VERSION;
        packet[NEXT_HEADER_OFFSET] = next_header;
        let src = Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1);
        packet[SRC_OFFSET..SRC_OFFSET + ADDR_LEN].copy_from_slice(&src.octets());
        let dst = Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 2);
        packet[DST_OFFSET..DST_OFFSET + ADDR_LEN].copy_from_slice(&dst.octets());
        packet[IPV6_HEADER_LEN..].copy_from_slice(payload);
        packet
    }

    #[test]
    fn parses_icmpv6_packet() {
        let packet = make_ipv6_packet(PROTO_ICMPV6, &[128, 0, 0, 0, 0, 0, 0, 0]);
        let info = parse_ipv6_packet(&packet).unwrap();

        assert_eq!(info.src, Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1));
        assert_eq!(info.dst, Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 2));
        assert_eq!(info.next_header, PROTO_ICMPV6);
        assert_eq!(info.udp_dst_port, None);
    }

    #[test]
    fn parses_udp_destination_port() {
        // UDP header: src_port(2) + dst_port(2) + len(2) + checksum(2)
        let mut udp_header = [0u8; 8];
        udp_header[2..4].copy_from_slice(&9999u16.to_be_bytes());
        let packet = make_ipv6_packet(PROTO_UDP, &udp_header);
        let info = parse_ipv6_packet(&packet).unwrap();

        assert_eq!(info.next_header, PROTO_UDP);
        assert_eq!(info.udp_dst_port, Some(9999));
    }

    #[test]
    fn rejects_non_ipv6_version() {
        let mut packet = vec![0u8; 40];
        packet[0] = 0x40;

        assert!(parse_ipv6_packet(&packet).is_none());
    }

    #[test]
    fn rejects_truncated_ipv6_header() {
        let packet = vec![0x60; 20];
        assert!(parse_ipv6_packet(&packet).is_none());
    }

    #[test]
    fn udp_port_none_when_payload_too_short() {
        let packet = make_ipv6_packet(PROTO_UDP, &[0, 0]);
        let info = parse_ipv6_packet(&packet).unwrap();
        assert_eq!(info.next_header, PROTO_UDP);
        assert_eq!(info.udp_dst_port, None);
    }
}
