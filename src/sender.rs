use std::io;
use std::net::{SocketAddrV6, UdpSocket};

use crate::cidr::Ipv6Cidr;
use crate::codec::encode_dst;

const MAX_PAYLOAD_PER_PACKET: usize = 6;
const DST_PORT: u16 = 9999;

pub fn send_message(cidr: &Ipv6Cidr, message: &[u8]) -> io::Result<()> {
    let socket = UdpSocket::bind("[::]:0")?;

    let chunks: Vec<&[u8]> = message.chunks(MAX_PAYLOAD_PER_PACKET).collect();
    let total = chunks.len().max(1);

    for (seq, chunk) in chunks.iter().enumerate() {
        let dst = encode_dst(cidr, seq as u8, chunk);
        let addr = SocketAddrV6::new(dst, DST_PORT, 0, 0);
        socket.send_to(b"x", addr)?;
    }

    if message.is_empty() || message.len().is_multiple_of(MAX_PAYLOAD_PER_PACKET) {
        let dst = encode_dst(cidr, total as u8, &[]);
        let addr = SocketAddrV6::new(dst, DST_PORT, 0, 0);
        socket.send_to(b"x", addr)?;
    }

    Ok(())
}
