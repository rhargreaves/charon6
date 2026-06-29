use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Transport {
    Icmp,
    Udp(u16),
}

impl fmt::Display for Transport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Transport::Icmp => write!(f, "ICMPv6"),
            Transport::Udp(port) => write!(f, "UDP/{port}"),
        }
    }
}
