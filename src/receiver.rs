use std::io::Write;
use std::os::fd::OwnedFd;
use std::time::{Duration, Instant};

use crate::Transport;
use crate::cidr::Ipv6Cidr;
use crate::cipher::{Cipher, HMAC_LEN};
use crate::codec::{DecodeError, Reassembler, decode_dst};
use crate::packet::{PROTO_ICMPV6, PROTO_UDP, PacketInfo, parse_ipv6_packet};

fn nix_to_io(err: nix::Error) -> std::io::Error {
    std::io::Error::from_raw_os_error(err as i32)
}

pub fn open_ipv6_packet_socket() -> std::io::Result<OwnedFd> {
    use nix::sys::socket::{AddressFamily, SockFlag, SockProtocol, SockType, socket};

    socket(
        AddressFamily::Packet,
        SockType::Datagram,
        SockFlag::empty(),
        SockProtocol::EthIpv6,
    )
    .map_err(nix_to_io)
}

pub fn capture_loop(
    fd: &OwnedFd,
    cidr: &Ipv6Cidr,
    transport: &Transport,
    key: Option<Cipher>,
    timeout: Duration,
) -> std::io::Result<()> {
    use nix::sys::socket::{MsgFlags, recv};
    use std::os::fd::AsRawFd;

    let mut buf = vec![0u8; 65536];
    let mut reassembler = Reassembler::new();
    let mut started_at: Option<Instant> = None;
    let stdout = std::io::stdout();

    loop {
        if let Some(t) = started_at
            && t.elapsed() >= timeout
        {
            eprintln!("dropped: incomplete message timed out");
            reassembler.clear();
            started_at = None;
        }

        let n = recv(fd.as_raw_fd(), &mut buf, MsgFlags::empty()).map_err(nix_to_io)?;

        let Some(info) = parse_ipv6_packet(&buf[..n]) else {
            eprintln!("dropped: malformed IPv6 header");
            continue;
        };

        if !matches_transport(&info, transport) {
            continue;
        }

        match decode_dst(info.dst, cidr, key.as_ref()) {
            Ok(frame) => {
                eprintln!("src={} -> dst={}", info.src, info.dst);
                if reassembler.is_empty() {
                    started_at = Some(Instant::now());
                }
                reassembler.push(frame);
                if let Some(payload) = reassembler.take() {
                    started_at = None;
                    if let Some(message) = verify_payload(payload, key.as_ref()) {
                        let mut out = stdout.lock();
                        out.write_all(&message)?;
                        out.write_all(b"\n")?;
                        out.flush()?;
                    }
                }
            }
            Err(DecodeError::OutOfCidr) => {}
            Err(DecodeError::InvalidLen(len)) => {
                eprintln!("src={} -> dst={}", info.src, info.dst);
                eprintln!("dropped: invalid len={len} from {}", info.dst);
            }
        }
    }
}

fn matches_transport(info: &PacketInfo, transport: &Transport) -> bool {
    match transport {
        Transport::Icmp => info.next_header == PROTO_ICMPV6,
        Transport::Udp(port) => info.next_header == PROTO_UDP && info.udp_dst_port == Some(*port),
    }
}

fn verify_payload(payload: Vec<u8>, cipher: Option<&Cipher>) -> Option<Vec<u8>> {
    match cipher {
        None => Some(payload),
        Some(c) => {
            if payload.len() < HMAC_LEN {
                eprintln!("dropped: message too short for HMAC");
                return None;
            }
            let (msg, tag) = payload.split_at(payload.len() - HMAC_LEN);
            let tag: &[u8; HMAC_LEN] = tag.try_into().unwrap();
            if !c.verify_hmac(msg, tag) {
                eprintln!("dropped: HMAC verification failed");
                return None;
            }
            Some(msg.to_vec())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opens_packet_socket() {
        assert!(open_ipv6_packet_socket().is_ok());
    }

    #[test]
    fn matches_icmp_transport() {
        let info = PacketInfo {
            src: "::1".parse().unwrap(),
            dst: "::2".parse().unwrap(),
            next_header: PROTO_ICMPV6,
            udp_dst_port: None,
        };
        assert!(matches_transport(&info, &Transport::Icmp));
        assert!(!matches_transport(&info, &Transport::Udp(9999)));
    }

    #[test]
    fn matches_udp_transport() {
        let info = PacketInfo {
            src: "::1".parse().unwrap(),
            dst: "::2".parse().unwrap(),
            next_header: PROTO_UDP,
            udp_dst_port: Some(9999),
        };
        assert!(matches_transport(&info, &Transport::Udp(9999)));
        assert!(!matches_transport(&info, &Transport::Udp(1234)));
        assert!(!matches_transport(&info, &Transport::Icmp));
    }

    #[test]
    fn verify_payload_no_cipher_passthrough() {
        let payload = b"hello".to_vec();
        assert_eq!(verify_payload(payload.clone(), None), Some(payload));
    }

    #[test]
    fn verify_payload_valid_hmac() {
        let cipher = Cipher::from_passphrase("test");
        let msg = b"hello";
        let mut payload = msg.to_vec();
        payload.extend_from_slice(&cipher.compute_hmac(msg));
        assert_eq!(verify_payload(payload, Some(&cipher)), Some(msg.to_vec()));
    }

    #[test]
    fn verify_payload_invalid_hmac() {
        let cipher = Cipher::from_passphrase("test");
        let wrong = Cipher::from_passphrase("wrong");
        let msg = b"hello";
        let mut payload = msg.to_vec();
        payload.extend_from_slice(&wrong.compute_hmac(msg));
        assert_eq!(verify_payload(payload, Some(&cipher)), None);
    }

    #[test]
    fn verify_payload_too_short_for_hmac() {
        let cipher = Cipher::from_passphrase("test");
        let payload = b"short".to_vec();
        assert_eq!(verify_payload(payload, Some(&cipher)), None);
    }
}
