use std::io::Write;
use std::os::fd::OwnedFd;

use crate::cidr::Ipv6Cidr;
use crate::codec::{DecodeError, Reassembler, decode_dst};
use crate::packet::parse_ipv6_endpoints;

pub fn open_ipv6_packet_socket() -> nix::Result<OwnedFd> {
    use nix::sys::socket::{AddressFamily, SockFlag, SockProtocol, SockType, socket};

    socket(
        AddressFamily::Packet,
        SockType::Datagram,
        SockFlag::empty(),
        SockProtocol::EthIpv6,
    )
}

pub fn capture_loop(fd: &OwnedFd, cidr: &Ipv6Cidr) -> nix::Result<()> {
    use nix::sys::socket::{MsgFlags, recv};
    use std::os::fd::AsRawFd;

    let mut buf = vec![0u8; 65536];
    let mut reassembler = Reassembler::new();
    let stdout = std::io::stdout();
    loop {
        let n = recv(fd.as_raw_fd(), &mut buf, MsgFlags::empty())?;
        let Some((src, dst)) = parse_ipv6_endpoints(&buf[..n]) else {
            eprintln!("dropped: malformed IPv6 header");
            continue;
        };

        match decode_dst(dst, cidr) {
            Ok(frame) => {
                eprintln!("src={src} -> dst={dst}");
                reassembler.push(frame);
                if let Some(message) = reassembler.take() {
                    let mut out = stdout.lock();
                    let _ = out.write_all(&message);
                    let _ = out.write_all(b"\n");
                    let _ = out.flush();
                }
            }
            Err(DecodeError::OutOfCidr) => {}
            Err(DecodeError::InvalidLen(len)) => {
                eprintln!("src={src} -> dst={dst}");
                eprintln!("dropped: invalid len={len} from {dst}");
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
