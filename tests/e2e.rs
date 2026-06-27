use std::io::Read;
use std::net::{Ipv6Addr, SocketAddrV6, UdpSocket};
use std::process::{Command, Stdio};
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::thread;
use std::time::Duration;

const TEST_CIDR: &str = "2001:db8::/64";

/// e2e tests share the loopback interface; serialize so captures don't interleave.
fn loopback_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn has_net_raw() -> bool {
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

fn encode_dst(seq: u8, payload: &[u8]) -> Ipv6Addr {
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

#[test]
fn decodes_hello_world_from_literal_destinations() {
    assert!(
        has_net_raw(),
        "missing CAP_NET_RAW: run via `make test` (uses sudo) or `make ci`"
    );
    let _guard = loopback_lock();

    let mut child = Command::new(env!("CARGO_BIN_EXE_charon6"))
        .args(["lo", "--cidr", TEST_CIDR])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn charon6");

    thread::sleep(Duration::from_millis(500));

    let socket = UdpSocket::bind("[::1]:0").expect("failed to bind loopback UDP socket");
    for dst in ["2001:db8::6:6865:6c6c:6f20", "2001:db8::105:776f:726c:6400"] {
        let addr: Ipv6Addr = dst.parse().expect("invalid destination address");
        let target = SocketAddrV6::new(addr, 9999, 0, 0);
        socket
            .send_to(b"x", target)
            .unwrap_or_else(|e| panic!("send_to {dst}: {e}"));
        thread::sleep(Duration::from_millis(80));
    }

    thread::sleep(Duration::from_millis(300));
    child.kill().expect("failed to kill charon6");

    let mut output = Vec::new();
    child
        .stdout
        .take()
        .expect("missing stdout pipe")
        .read_to_end(&mut output)
        .expect("failed to read charon6 stdout");

    child.wait().expect("failed to reap charon6");

    let text = String::from_utf8_lossy(&output);
    assert!(
        text.contains("hello world\n"),
        "expected decoded message on stdout, got: {text:?}"
    );
}

#[test]
fn decodes_message_from_destination_addresses() {
    assert!(
        has_net_raw(),
        "missing CAP_NET_RAW: run via `make test` (uses sudo) or `make ci`"
    );
    let _guard = loopback_lock();

    let mut child = Command::new(env!("CARGO_BIN_EXE_charon6"))
        .args(["lo", "--cidr", TEST_CIDR])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn charon6");

    thread::sleep(Duration::from_millis(500));

    let socket = UdpSocket::bind("[::1]:0").expect("failed to bind loopback UDP socket");
    for (seq, chunk) in [(0u8, b"hello " as &[u8]), (1, b"world!"), (2, b"")]
        .into_iter()
        .enumerate()
        .map(|(i, (_, c))| (i as u8, c))
    {
        let dst = encode_dst(seq, chunk);
        let target = SocketAddrV6::new(dst, 9999, 0, 0);
        socket
            .send_to(b"x", target)
            .unwrap_or_else(|e| panic!("send_to {dst}: {e}"));
        thread::sleep(Duration::from_millis(80));
    }

    thread::sleep(Duration::from_millis(300));
    child.kill().expect("failed to kill charon6");

    let mut output = Vec::new();
    child
        .stdout
        .take()
        .expect("missing stdout pipe")
        .read_to_end(&mut output)
        .expect("failed to read charon6 stdout");

    child.wait().expect("failed to reap charon6");

    let text = String::from_utf8_lossy(&output);
    assert!(
        text.contains("hello world!\n"),
        "expected decoded message on stdout, got: {text:?}"
    );
}
