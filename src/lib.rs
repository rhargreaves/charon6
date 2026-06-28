mod capture;
mod cidr;
mod codec;
mod packet;
mod sender;
mod xtea;

pub use capture::{capture_loop, open_ipv6_packet_socket};
pub use cidr::Ipv6Cidr;
pub use sender::send_message;
pub use xtea::{XteaKey, key_from_passphrase};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Transport {
    Icmp,
    Udp(u16),
}
