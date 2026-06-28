use std::collections::BTreeMap;
use std::net::Ipv6Addr;

use crate::cidr::Ipv6Cidr;

#[derive(Debug, PartialEq, Eq)]
pub struct Frame {
    pub seq: u8,
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
pub const MAX_PAYLOAD_PER_FRAME: usize = HOST_BYTES - PAYLOAD_OFFSET;

#[derive(Default)]
pub struct Reassembler {
    frames: BTreeMap<u8, Frame>,
    last_seq: Option<u8>,
}

impl Reassembler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, frame: Frame) {
        if frame.is_last {
            self.last_seq = Some(frame.seq);
        }
        self.frames.insert(frame.seq, frame);
    }

    pub fn is_complete(&self) -> bool {
        match self.last_seq {
            Some(last) => {
                let expected_count = last as usize + 1;
                self.frames.len() == expected_count
            }
            None => false,
        }
    }
    pub fn take(&mut self) -> Option<Vec<u8>> {
        if !self.is_complete() {
            return None;
        }
        let message = self
            .frames
            .values()
            .flat_map(|f| f.payload.iter().copied())
            .collect();
        self.frames.clear();
        self.last_seq = None;
        Some(message)
    }
}

pub fn encode_dst(cidr: &Ipv6Cidr, seq: u8, payload: &[u8]) -> Ipv6Addr {
    debug_assert!(payload.len() <= MAX_PAYLOAD_PER_FRAME);
    let mut bytes = cidr.network().octets();
    bytes[HOST_BYTES + SEQ_OFFSET] = seq;
    bytes[HOST_BYTES + LEN_OFFSET] = payload.len() as u8;
    bytes[HOST_BYTES + PAYLOAD_OFFSET..HOST_BYTES + PAYLOAD_OFFSET + payload.len()]
        .copy_from_slice(payload);
    Ipv6Addr::from(bytes)
}

pub fn decode_dst(addr: Ipv6Addr, cidr: &Ipv6Cidr) -> Result<Frame, DecodeError> {
    if !cidr.contains(addr) {
        return Err(DecodeError::OutOfCidr);
    }
    let bytes = addr.octets();
    let host = &bytes[bytes.len() - HOST_BYTES..];
    let seq = host[SEQ_OFFSET];
    let len = host[LEN_OFFSET];
    if len as usize > MAX_PAYLOAD_PER_FRAME {
        return Err(DecodeError::InvalidLen(len));
    }
    let len_usize = len as usize;
    let payload = host[PAYLOAD_OFFSET..PAYLOAD_OFFSET + len_usize].to_vec();
    Ok(Frame {
        seq,
        payload,
        is_last: (len as usize) < MAX_PAYLOAD_PER_FRAME,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_len_exceeding_payload_capacity() {
        let cidr: Ipv6Cidr = "2001:db8::/64".parse().unwrap();
        let addr: Ipv6Addr = "2001:db8::0007:0000:0000:0000".parse().unwrap();

        assert_eq!(decode_dst(addr, &cidr), Err(DecodeError::InvalidLen(7)));
    }

    #[test]
    fn rejects_address_outside_cidr() {
        let cidr: Ipv6Cidr = "2001:db8::/64".parse().unwrap();
        let addr: Ipv6Addr = "fe80::1".parse().unwrap();

        assert_eq!(decode_dst(addr, &cidr), Err(DecodeError::OutOfCidr));
    }

    #[test]
    fn decodes_mid_message_frame_when_payload_full() {
        let cidr: Ipv6Cidr = "2001:db8::/64".parse().unwrap();
        let addr: Ipv6Addr = "2001:db8::0006:6865:6c6c:6f20".parse().unwrap();

        let frame = decode_dst(addr, &cidr).expect("expected Ok");
        assert_eq!(
            frame,
            Frame {
                seq: 0,
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
                seq: 0x99,
                payload: vec![b'h', b'i', b'!'],
                is_last: true,
            }
        );
    }

    #[test]
    fn reassembler_takes_complete_message_in_order() {
        let mut r = Reassembler::new();
        r.push(Frame {
            seq: 0,
            payload: b"hello ".to_vec(),
            is_last: false,
        });
        assert!(r.take().is_none());
        r.push(Frame {
            seq: 1,
            payload: b"world".to_vec(),
            is_last: true,
        });
        assert_eq!(r.take(), Some(b"hello world".to_vec()));
    }

    #[test]
    fn reassembler_takes_complete_message_out_of_order() {
        let mut r = Reassembler::new();
        r.push(Frame {
            seq: 1,
            payload: b"world".to_vec(),
            is_last: true,
        });
        assert!(r.take().is_none());
        r.push(Frame {
            seq: 0,
            payload: b"hello ".to_vec(),
            is_last: false,
        });
        assert_eq!(r.take(), Some(b"hello world".to_vec()));
    }

    #[test]
    fn reassembler_incomplete_returns_none() {
        let mut r = Reassembler::new();
        r.push(Frame {
            seq: 0,
            payload: b"a".to_vec(),
            is_last: false,
        });
        r.push(Frame {
            seq: 2,
            payload: b"c".to_vec(),
            is_last: true,
        });
        // seq 1 missing
        assert!(r.take().is_none());
    }

    #[test]
    fn reassembler_duplicate_frames_are_idempotent() {
        let mut r = Reassembler::new();
        r.push(Frame {
            seq: 0,
            payload: b"hello ".to_vec(),
            is_last: false,
        });
        r.push(Frame {
            seq: 0,
            payload: b"hello ".to_vec(),
            is_last: false,
        });
        r.push(Frame {
            seq: 1,
            payload: b"world".to_vec(),
            is_last: true,
        });
        assert_eq!(r.take(), Some(b"hello world".to_vec()));
    }

    #[test]
    fn reassembler_resets_after_take() {
        let mut r = Reassembler::new();
        r.push(Frame {
            seq: 0,
            payload: b"first".to_vec(),
            is_last: true,
        });
        assert!(r.take().is_some());
        assert!(r.take().is_none());
        // Can accept a new message
        r.push(Frame {
            seq: 0,
            payload: b"second".to_vec(),
            is_last: true,
        });
        assert_eq!(r.take(), Some(b"second".to_vec()));
    }

    #[test]
    fn reassembler_single_terminator_frame() {
        let mut r = Reassembler::new();
        r.push(Frame {
            seq: 0,
            payload: b"hi".to_vec(),
            is_last: true,
        });
        assert_eq!(r.take(), Some(b"hi".to_vec()));
    }
}
