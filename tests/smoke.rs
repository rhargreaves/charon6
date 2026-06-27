//! End-to-end smoke test: spawn the built binary, generate IPv6 loopback
//! traffic, and assert that it reports the captured source/destination.
//!
//! Requires CAP_NET_RAW (run via `sudo -E $(which cargo) test`). When the
//! capability is absent the test skips so the suite stays green unprivileged.

use std::io::Read;
use std::net::{Ipv6Addr, SocketAddrV6, UdpSocket};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

/// Returns true if the current process holds CAP_NET_RAW in its effective set.
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

#[test]
fn reports_ipv6_traffic_on_loopback() {
    if !has_net_raw() {
        eprintln!("skipping reports_ipv6_traffic_on_loopback: missing CAP_NET_RAW");
        return;
    }

    let mut child = Command::new(env!("CARGO_BIN_EXE_charon6"))
        .arg("lo")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn charon6");

    // Give the binary time to open and bind the socket before sending traffic.
    thread::sleep(Duration::from_millis(500));

    // Emit a few IPv6 packets on the loopback. No listener is needed: the
    // AF_PACKET socket observes the outgoing datagrams regardless.
    let socket = UdpSocket::bind("[::1]:0").expect("failed to bind loopback UDP socket");
    let dst = SocketAddrV6::new(Ipv6Addr::LOCALHOST, 9999, 0, 0);
    for _ in 0..5 {
        let _ = socket.send_to(b"smoke", dst);
        thread::sleep(Duration::from_millis(50));
    }

    // Allow the capture loop to drain, then stop the binary.
    thread::sleep(Duration::from_millis(200));
    child.kill().expect("failed to kill charon6");

    let mut output = String::new();
    child
        .stdout
        .take()
        .expect("missing stdout pipe")
        .read_to_string(&mut output)
        .expect("failed to read charon6 stdout");

    assert!(
        output.contains("src=::1 -> dst=::1"),
        "expected loopback capture in output, got:\n{output}"
    );
}
