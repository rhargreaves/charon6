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
