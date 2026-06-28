mod capture;
mod cidr;
mod cipher;
mod codec;
mod packet;
mod sender;
#[cfg(test)]
mod test_helpers;

pub use capture::{capture_loop, open_ipv6_packet_socket};
pub use cidr::Ipv6Cidr;
pub use cipher::Cipher;
pub use sender::send_message;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Transport {
    Icmp,
    Udp(u16),
}
