use std::net::Ipv6Addr;

use crate::cidr::Ipv6Cidr;

#[derive(Debug, PartialEq, Eq)]
pub struct Frame {
    pub payload: Vec<u8>,
    pub is_last: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum DecodeError {
    OutOfCidr,
    InvalidLen(u8),
}

const HOST_BYTES: usize = 8;
const SEQ_OFFSET: usize = 0;
const LEN_OFFSET: usize = 1;
const PAYLOAD_OFFSET: usize = 2;
const MAX_PAYLOAD: u8 = (HOST_BYTES - PAYLOAD_OFFSET) as u8;

pub fn decode_dst(addr: Ipv6Addr, cidr: &Ipv6Cidr) -> Result<Frame, DecodeError> {
    if !cidr.contains(addr) {
        return Err(DecodeError::OutOfCidr);
    }
    let bytes = addr.octets();
    let host = &bytes[bytes.len() - HOST_BYTES..];
    let _seq = host[SEQ_OFFSET];
    let len = host[LEN_OFFSET];
    if len > MAX_PAYLOAD {
        return Err(DecodeError::InvalidLen(len));
    }
    let len_usize = len as usize;
    let payload = host[PAYLOAD_OFFSET..PAYLOAD_OFFSET + len_usize].to_vec();
    Ok(Frame {
        payload,
        is_last: len < MAX_PAYLOAD,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_mid_message_frame_when_payload_full() {
        let cidr: Ipv6Cidr = "2001:db8::/64".parse().unwrap();
        let addr: Ipv6Addr = "2001:db8::0006:6865:6c6c:6f20".parse().unwrap();

        let frame = decode_dst(addr, &cidr).expect("expected Ok");
        assert_eq!(
            frame,
            Frame {
                payload: b"hello ".to_vec(),
                is_last: false,
            }
        );
    }

    #[test]
    fn decodes_terminator_frame() {
        let cidr: Ipv6Cidr = "2001:db8::/64".parse().unwrap();
        let addr: Ipv6Addr = "2001:db8::9903:6869:2100:0".parse().unwrap();

        let frame = decode_dst(addr, &cidr).expect("expected Ok");
        assert_eq!(
            frame,
            Frame {
                payload: vec![b'h', b'i', b'!'],
                is_last: true,
            }
        );
    }
}
