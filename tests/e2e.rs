use std::io::Read;
use std::net::{Ipv6Addr, SocketAddrV6, UdpSocket};
use std::process::{Command, Stdio};
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::thread;
use std::time::Duration;

const TEST_CIDR: &str = "2001:db8::/64";
const STARTUP_DELAY: Duration = Duration::from_millis(500);
const SEND_INTERVAL: Duration = Duration::from_millis(80);
const DRAIN_DELAY: Duration = Duration::from_millis(300);
const DST_PORT: u16 = 9999;

fn loopback_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
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

fn send_to(socket: &UdpSocket, dst: Ipv6Addr) {
    socket
        .send_to(b"x", SocketAddrV6::new(dst, DST_PORT, 0, 0))
        .unwrap_or_else(|e| panic!("send_to {dst}: {e}"));
    thread::sleep(SEND_INTERVAL);
}

/// Run charon6 against `lo` decoding `cidr`, invoke `send` to emit packets,
/// then return whatever the binary wrote to stdout.
fn capture_with(cidr: &str, send: impl FnOnce(&UdpSocket)) -> String {
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

#[test]
fn decodes_single_packet_message() {
    let text = capture_with(TEST_CIDR, |socket| {
        send_to(socket, encode_dst(0, b"hi!"));
    });

    assert!(
        text.contains("hi!\n"),
        "expected decoded message on stdout, got: {text:?}"
    );
}

#[test]
fn decodes_two_packet_message() {
    let text = capture_with(TEST_CIDR, |socket| {
        for dst in ["2001:db8::6:6865:6c6c:6f20", "2001:db8::105:776f:726c:6400"] {
            let addr: Ipv6Addr = dst.parse().expect("invalid destination address");
            send_to(socket, addr);
        }
    });

    assert!(
        text.contains("hello world\n"),
        "expected decoded message on stdout, got: {text:?}"
    );
}

#[test]
fn decodes_ten_packet_message() {
    const TOTAL_PACKETS: usize = 10;
    const PAYLOAD_PER_PACKET: usize = 6;
    // 9 full packets + a 1-byte terminator = 10 packets total.
    const MESSAGE_LEN: usize = (TOTAL_PACKETS - 1) * PAYLOAD_PER_PACKET + 1;

    let message: Vec<u8> = (0..MESSAGE_LEN).map(|i| b'a' + (i as u8 % 26)).collect();
    let chunks: Vec<&[u8]> = message.chunks(PAYLOAD_PER_PACKET).collect();
    assert_eq!(chunks.len(), TOTAL_PACKETS);

    let text = capture_with(TEST_CIDR, |socket| {
        for (seq, chunk) in chunks.iter().enumerate() {
            send_to(socket, encode_dst(seq as u8, chunk));
        }
    });

    let expected = format!(
        "{}\n",
        std::str::from_utf8(&message).expect("message is ASCII")
    );
    assert!(
        text.contains(&expected),
        "expected decoded message on stdout, got: {text:?}"
    );
}
