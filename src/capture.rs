use std::io::Write;
use std::os::fd::OwnedFd;

use crate::cidr::Ipv6Cidr;
use crate::codec::{DecodeError, Reassembler, decode_dst};
use crate::packet::{PROTO_ICMPV6, PROTO_UDP, parse_ipv6_packet};

pub fn open_ipv6_packet_socket() -> nix::Result<OwnedFd> {
    use nix::sys::socket::{AddressFamily, SockFlag, SockProtocol, SockType, socket};

    socket(
        AddressFamily::Packet,
        SockType::Datagram,
        SockFlag::empty(),
        SockProtocol::EthIpv6,
    )
}

pub fn capture_loop(
    fd: &OwnedFd,
    cidr: &Ipv6Cidr,
    port: Option<u16>,
    key: Option<[u32; 4]>,
) -> std::io::Result<()> {
    use nix::sys::socket::{MsgFlags, recv};
    use std::os::fd::AsRawFd;

    let mut buf = vec![0u8; 65536];
    let mut reassembler = Reassembler::new();
    let stdout = std::io::stdout();
    loop {
        let n = recv(fd.as_raw_fd(), &mut buf, MsgFlags::empty())
            .map_err(|e| std::io::Error::from_raw_os_error(e as i32))?;
        let Some(info) = parse_ipv6_packet(&buf[..n]) else {
            eprintln!("dropped: malformed IPv6 header");
            continue;
        };

        match port {
            None => {
                if info.next_header != PROTO_ICMPV6 {
                    continue;
                }
            }
            Some(p) => {
                if info.next_header != PROTO_UDP || info.udp_dst_port != Some(p) {
                    continue;
                }
            }
        }

        match decode_dst(info.dst, cidr, key.as_ref()) {
            Ok(frame) => {
                eprintln!("src={} -> dst={}", info.src, info.dst);
                reassembler.push(frame);
                if let Some(message) = reassembler.take() {
                    let mut out = stdout.lock();
                    out.write_all(&message)?;
                    out.write_all(b"\n")?;
                    out.flush()?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opens_packet_socket() {
        assert!(open_ipv6_packet_socket().is_ok());
    }
}
