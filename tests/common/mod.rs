use std::io::{Read, Write};
use std::net::{Ipv6Addr, SocketAddrV6, UdpSocket};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

const STARTUP_DELAY: Duration = Duration::from_millis(500);
const SEND_INTERVAL: Duration = Duration::from_millis(80);
const DRAIN_DELAY: Duration = Duration::from_millis(300);
const DST_PORT: u16 = 9999;
const PREFIX_BYTES: usize = 8;

pub fn has_net_raw() -> bool {
    const CAP_NET_RAW: u32 = 13;
    let status = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    for line in status.lines() {
        if let Some(hex) = line.strip_prefix("CapEff:")
            && let Ok(caps) = u64::from_str_radix(hex.trim(), 16)
        {
            return caps & (1 << CAP_NET_RAW) != 0;
        }
    }
    false
}

pub fn encode_dst(cidr: &str, seq: u8, payload: &[u8]) -> Ipv6Addr {
    assert!(payload.len() <= 6);
    let network = cidr
        .split('/')
        .next()
        .and_then(|s| s.parse::<Ipv6Addr>().ok())
        .unwrap_or_else(|| panic!("invalid cidr: {cidr}"));

    let mut bytes = network.octets();
    bytes[PREFIX_BYTES] = seq;
    bytes[PREFIX_BYTES + 1] = payload.len() as u8;
    bytes[PREFIX_BYTES + 2..PREFIX_BYTES + 2 + payload.len()].copy_from_slice(payload);
    Ipv6Addr::from(bytes)
}

pub fn send_to(socket: &UdpSocket, dst: Ipv6Addr) {
    socket
        .send_to(b"x", SocketAddrV6::new(dst, DST_PORT, 0, 0))
        .unwrap_or_else(|e| panic!("send_to {dst}: {e}"));
    thread::sleep(SEND_INTERVAL);
}

/// Run charon6 against `lo` decoding `cidr`, invoke `send` to emit packets,
/// then return whatever the binary wrote to stdout.
///
/// Tests should pass distinct CIDRs so they can run in parallel without
/// capturing each other's traffic.
pub fn capture_with(cidr: &str, send: impl FnOnce(&UdpSocket)) -> String {
    assert!(
        has_net_raw(),
        "missing CAP_NET_RAW: run via `make test` (uses sudo) or `make ci`"
    );

    let mut child = Command::new(env!("CARGO_BIN_EXE_charon6"))
        .args(["--cidr", cidr])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn charon6");

    thread::sleep(STARTUP_DELAY);

    let socket = UdpSocket::bind("[::1]:0").expect("failed to bind loopback UDP socket");
    send(&socket);

    thread::sleep(DRAIN_DELAY);
    child.kill().expect("failed to kill charon6");

    let mut output = Vec::new();
    child
        .stdout
        .take()
        .expect("missing stdout pipe")
        .read_to_end(&mut output)
        .expect("failed to read charon6 stdout");
    child.wait().expect("failed to reap charon6");

    String::from_utf8_lossy(&output).into_owned()
}

/// Spawn a receiver and sender pair, pipe `message` into the sender,
/// and return whatever the receiver wrote to stdout.
pub fn send_recv(cidr: &str, message: &[u8], extra_send_args: &[&str]) -> String {
    assert!(
        has_net_raw(),
        "missing CAP_NET_RAW: run via `make test` (uses sudo) or `make ci`"
    );

    let mut receiver = Command::new(env!("CARGO_BIN_EXE_charon6"))
        .args(["--recv", "--cidr", cidr])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn receiver");

    thread::sleep(STARTUP_DELAY);

    let mut send_args = vec!["--send", "--cidr", cidr];
    send_args.extend_from_slice(extra_send_args);

    let mut sender = Command::new(env!("CARGO_BIN_EXE_charon6"))
        .args(&send_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn sender");

    sender
        .stdin
        .take()
        .unwrap()
        .write_all(message)
        .expect("failed to write to sender stdin");

    sender.wait().expect("sender failed");

    thread::sleep(DRAIN_DELAY);
    receiver.kill().expect("failed to kill receiver");

    let mut output = Vec::new();
    receiver
        .stdout
        .take()
        .expect("missing stdout pipe")
        .read_to_end(&mut output)
        .expect("failed to read receiver stdout");
    receiver.wait().expect("failed to reap receiver");

    String::from_utf8_lossy(&output).into_owned()
}
