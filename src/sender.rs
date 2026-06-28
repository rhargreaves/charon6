use std::io;
use std::net::Ipv6Addr;

use crate::Transport;
use crate::cidr::Ipv6Cidr;
use crate::codec::{MAX_PAYLOAD_PER_FRAME, encode_dst};

fn nix_to_io(err: nix::Error) -> io::Error {
    io::Error::from_raw_os_error(err as i32)
}

const MAX_PACKETS: usize = u8::MAX as usize + 1;

pub fn send_message(
    cidr: &Ipv6Cidr,
    message: &[u8],
    transport: &Transport,
    key: Option<crate::cipher::Cipher>,
) -> io::Result<()> {
    let destinations = encode_message(cidr, message, key.as_ref())?;
    match transport {
        Transport::Udp(port) => send_udp(&destinations, *port),
        Transport::Icmp => send_icmp(&destinations),
    }
}

fn encode_message(
    cidr: &Ipv6Cidr,
    message: &[u8],
    key: Option<&crate::cipher::Cipher>,
) -> io::Result<Vec<Ipv6Addr>> {
    let num_chunks = message.chunks(MAX_PAYLOAD_PER_FRAME).count();
    let needs_empty_terminator =
        message.is_empty() || message.len().is_multiple_of(MAX_PAYLOAD_PER_FRAME);
    let total_packets = num_chunks + needs_empty_terminator as usize;

    if total_packets > MAX_PACKETS {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("message too large: {total_packets} packets needed, max is {MAX_PACKETS}"),
        ));
    }

    let mut destinations: Vec<Ipv6Addr> = message
        .chunks(MAX_PAYLOAD_PER_FRAME)
        .enumerate()
        .map(|(seq, chunk)| encode_dst(cidr, seq as u8, chunk, key))
        .collect();

    if needs_empty_terminator {
        destinations.push(encode_dst(cidr, num_chunks as u8, &[], key));
    }

    Ok(destinations)
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
    .map_err(nix_to_io)?;

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
        sendto(fd.as_raw_fd(), &icmp_header, &sockaddr, MsgFlags::empty()).map_err(nix_to_io)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_helpers::doc_cidr;

    #[test]
    fn encode_short_message_produces_single_terminator_packet() {
        let dsts = encode_message(&doc_cidr(), b"hi!", None).unwrap();
        assert_eq!(dsts.len(), 1);
    }

    #[test]
    fn encode_exact_multiple_appends_empty_terminator() {
        let dsts = encode_message(&doc_cidr(), b"abcdef", None).unwrap();
        assert_eq!(dsts.len(), 2);
    }

    #[test]
    fn encode_empty_message_produces_single_empty_terminator() {
        let dsts = encode_message(&doc_cidr(), b"", None).unwrap();
        assert_eq!(dsts.len(), 1);
    }

    #[test]
    fn encode_12_bytes_produces_two_full_frames_plus_terminator() {
        let dsts = encode_message(&doc_cidr(), b"abcdefghijkl", None).unwrap();
        assert_eq!(dsts.len(), 3);
    }

    #[test]
    fn encode_7_bytes_produces_two_packets() {
        let dsts = encode_message(&doc_cidr(), b"abcdefg", None).unwrap();
        assert_eq!(dsts.len(), 2);
    }

    #[test]
    fn encode_oversized_message_returns_error() {
        let message = vec![0u8; MAX_PACKETS * MAX_PAYLOAD_PER_FRAME + 1];
        let result = encode_message(&doc_cidr(), &message, None);
        assert!(result.is_err());
    }

    #[test]
    fn encode_max_size_message_succeeds() {
        // 255 full frames + 1 terminator with 5 bytes = 1535 bytes, 256 packets
        let message =
            vec![0u8; (MAX_PACKETS - 1) * MAX_PAYLOAD_PER_FRAME + MAX_PAYLOAD_PER_FRAME - 1];
        let dsts = encode_message(&doc_cidr(), &message, None).unwrap();
        assert_eq!(dsts.len(), MAX_PACKETS);
    }
}
