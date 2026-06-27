use std::io::Read;
use std::net::{Ipv6Addr, SocketAddrV6, UdpSocket};
use std::process::{Command, Stdio};
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::thread;
use std::time::Duration;

pub const TEST_CIDR: &str = "2001:db8::/64";

const STARTUP_DELAY: Duration = Duration::from_millis(500);
const SEND_INTERVAL: Duration = Duration::from_millis(80);
const DRAIN_DELAY: Duration = Duration::from_millis(300);
const DST_PORT: u16 = 9999;

pub fn loopback_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

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

pub fn encode_dst(seq: u8, payload: &[u8]) -> Ipv6Addr {
    assert!(payload.len() <= 6);
    let mut host = [0u8; 8];
    host[0] = seq;
    host[1] = payload.len() as u8;
    host[2..2 + payload.len()].copy_from_slice(payload);

    let mut bytes = [0u8; 16];
    bytes[..8].copy_from_slice(&[0x20, 0x01, 0x0d, 0xb8, 0, 0, 0, 0]);
    bytes[8..].copy_from_slice(&host);
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
pub fn capture_with(cidr: &str, send: impl FnOnce(&UdpSocket)) -> String {
    assert!(
        has_net_raw(),
        "missing CAP_NET_RAW: run via `make test` (uses sudo) or `make ci`"
    );
    let _guard = loopback_lock();

    let mut child = Command::new(env!("CARGO_BIN_EXE_charon6"))
        .args(["lo", "--cidr", cidr])
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
