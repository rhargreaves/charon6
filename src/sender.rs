use std::io;
use std::net::Ipv6Addr;

use crate::Transport;
use crate::cidr::Ipv6Cidr;
use crate::codec::{MAX_PAYLOAD_PER_FRAME, encode_dst};

pub fn send_message(
    cidr: &Ipv6Cidr,
    message: &[u8],
    transport: &Transport,
    key: Option<crate::xtea::XteaKey>,
) -> io::Result<()> {
    let destinations = encode_message(cidr, message, key.as_ref());
    match transport {
        Transport::Udp(port) => send_udp(&destinations, *port),
        Transport::Icmp => send_icmp(&destinations),
    }
}

fn encode_message(
    cidr: &Ipv6Cidr,
    message: &[u8],
    key: Option<&crate::xtea::XteaKey>,
) -> Vec<Ipv6Addr> {
    let chunks: Vec<&[u8]> = message.chunks(MAX_PAYLOAD_PER_FRAME).collect();
    let total = chunks.len().max(1);

    let mut destinations: Vec<Ipv6Addr> = chunks
        .iter()
        .enumerate()
        .map(|(seq, chunk)| encode_dst(cidr, seq as u8, chunk, key))
        .collect();

    if message.is_empty() || message.len().is_multiple_of(MAX_PAYLOAD_PER_FRAME) {
        destinations.push(encode_dst(cidr, total as u8, &[], key));
    }

    destinations
}

fn send_udp(destinations: &[Ipv6Addr], port: u16) -> io::Result<()> {
    use std::net::{SocketAddrV6, UdpSocket};

    let socket = UdpSocket::bind("[::]:0")?;
    for dst in destinations {
        socket.send_to(b"", SocketAddrV6::new(*dst, port, 0, 0))?;
    }
    Ok(())
}

fn send_icmp(destinations: &[Ipv6Addr]) -> io::Result<()> {
    use nix::sys::socket::{
        AddressFamily, MsgFlags, SockFlag, SockProtocol, SockType, sendto, socket,
    };
    use std::os::fd::AsRawFd;

    let fd = socket(
        AddressFamily::Inet6,
        SockType::Raw,
        SockFlag::empty(),
        SockProtocol::IcmpV6,
    )
    .map_err(|e| io::Error::from_raw_os_error(e as i32))?;

    const ICMPV6_ECHO_REQUEST: u8 = 128;
    let icmp_header: [u8; 8] = [
        ICMPV6_ECHO_REQUEST, // type
        0,                   // code
        0,
        0, // checksum (kernel computes)
        0,
        0, // identifier
        0,
        0, // sequence number
    ];

    for dst in destinations {
        let sockaddr =
            nix::sys::socket::SockaddrIn6::from(std::net::SocketAddrV6::new(*dst, 0, 0, 0));
        sendto(fd.as_raw_fd(), &icmp_header, &sockaddr, MsgFlags::empty())
            .map_err(|e| io::Error::from_raw_os_error(e as i32))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cidr() -> Ipv6Cidr {
        "2001:db8::/64".parse().unwrap()
    }

    #[test]
    fn encode_short_message_produces_single_terminator_packet() {
        let dsts = encode_message(&cidr(), b"hi!", None);
        assert_eq!(dsts.len(), 1);
    }

    #[test]
    fn encode_exact_multiple_appends_empty_terminator() {
        let dsts = encode_message(&cidr(), b"abcdef", None);
        assert_eq!(dsts.len(), 2);
    }

    #[test]
    fn encode_empty_message_produces_single_empty_terminator() {
        let dsts = encode_message(&cidr(), b"", None);
        assert_eq!(dsts.len(), 1);
    }

    #[test]
    fn encode_12_bytes_produces_two_full_frames_plus_terminator() {
        let dsts = encode_message(&cidr(), b"abcdefghijkl", None);
        assert_eq!(dsts.len(), 3);
    }

    #[test]
    fn encode_7_bytes_produces_two_packets() {
        let dsts = encode_message(&cidr(), b"abcdefg", None);
        assert_eq!(dsts.len(), 2);
    }
}
