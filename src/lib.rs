mod cidr;
mod cipher;
mod codec;
mod packet;
mod receiver;
mod sender;
#[cfg(test)]
mod test_helpers;

pub use cidr::Ipv6Cidr;
pub use cipher::Cipher;
pub use receiver::{open_ipv6_packet_socket, receive_loop};
pub use sender::send_message;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Transport {
    Icmp,
    Udp(u16),
}

impl std::fmt::Display for Transport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Transport::Icmp => write!(f, "ICMPv6"),
            Transport::Udp(port) => write!(f, "UDP/{port}"),
        }
    }
}

pub(crate) fn nix_to_io(err: nix::Error) -> std::io::Error {
    std::io::Error::from_raw_os_error(err as i32)
}
