use std::io::Read;
use std::net::{Ipv6Addr, SocketAddrV6, UdpSocket};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

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
    assert!(
        has_net_raw(),
        "missing CAP_NET_RAW: run via `make test` (uses sudo) or `make ci`"
    );

    let mut child = Command::new(env!("CARGO_BIN_EXE_charon6"))
        .arg("lo")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn charon6");

    thread::sleep(Duration::from_millis(500));

    let socket = UdpSocket::bind("[::1]:0").expect("failed to bind loopback UDP socket");
    let dst = SocketAddrV6::new(Ipv6Addr::LOCALHOST, 9999, 0, 0);
    for _ in 0..5 {
        let _ = socket.send_to(b"smoke", dst);
        thread::sleep(Duration::from_millis(50));
    }

    thread::sleep(Duration::from_millis(200));
    child.kill().expect("failed to kill charon6");

    let mut output = String::new();
    child
        .stdout
        .take()
        .expect("missing stdout pipe")
        .read_to_string(&mut output)
        .expect("failed to read charon6 stdout");

    child.wait().expect("failed to reap charon6");

    assert!(
        output.contains("src=::1 -> dst=::1"),
        "expected loopback capture in output, got:\n{output}"
    );
}
