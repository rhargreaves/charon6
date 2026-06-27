mod common;

use std::net::Ipv6Addr;

use common::{capture_with, encode_dst, send_to};

#[test]
fn decodes_single_packet_message() {
    const CIDR: &str = "2001:db8:1::/64";

    let text = capture_with(CIDR, |socket| {
        send_to(socket, encode_dst(CIDR, 0, b"hi!"));
    });

    assert!(
        text.contains("hi!\n"),
        "expected decoded message on stdout, got: {text:?}"
    );
}

#[test]
fn decodes_two_packet_message() {
    const CIDR: &str = "2001:db8:2::/64";

    let text = capture_with(CIDR, |socket| {
        for dst in [
            "2001:db8:2::6:6865:6c6c:6f20",
            "2001:db8:2::105:776f:726c:6400",
        ] {
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
    const CIDR: &str = "2001:db8:3::/64";
    const TOTAL_PACKETS: usize = 10;
    const PAYLOAD_PER_PACKET: usize = 6;
    // 9 full packets + a 1-byte terminator = 10 packets total.
    const MESSAGE_LEN: usize = (TOTAL_PACKETS - 1) * PAYLOAD_PER_PACKET + 1;

    let message: Vec<u8> = (0..MESSAGE_LEN).map(|i| b'a' + (i as u8 % 26)).collect();
    let chunks: Vec<&[u8]> = message.chunks(PAYLOAD_PER_PACKET).collect();
    assert_eq!(chunks.len(), TOTAL_PACKETS);

    let text = capture_with(CIDR, |socket| {
        for (seq, chunk) in chunks.iter().enumerate() {
            send_to(socket, encode_dst(CIDR, seq as u8, chunk));
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

#[test]
fn decodes_out_of_order_packets() {
    const CIDR: &str = "2001:db8:4::/64";

    let text = capture_with(CIDR, |socket| {
        // Send packets out of order: seq 2, seq 0, seq 1
        send_to(socket, encode_dst(CIDR, 2, b"world")); // terminator (len < 6)
        send_to(socket, encode_dst(CIDR, 0, b"hello "));
        send_to(socket, encode_dst(CIDR, 1, b"cruel "));
    });

    assert!(
        text.contains("hello cruel world\n"),
        "expected reordered message on stdout, got: {text:?}"
    );
}

#[test]
fn incomplete_message_produces_no_output() {
    const CIDR: &str = "2001:db8:5::/64";

    let text = capture_with(CIDR, |socket| {
        // Send terminator (seq 2) but missing seq 1 - message incomplete
        send_to(socket, encode_dst(CIDR, 2, b"end")); // terminator
        send_to(socket, encode_dst(CIDR, 0, b"start "));
        // seq 1 never sent
    });

    assert!(
        text.is_empty(),
        "expected no output for incomplete message, got: {text:?}"
    );
}

#[test]
fn send_mode_encodes_stdin_to_packets() {
    use std::io::Write;
    use std::process::{Command, Stdio};

    const CIDR: &str = "2001:db8:6::/64";
    const MESSAGE: &[u8] = b"hello";

    // Start receiver
    let mut receiver = Command::new(env!("CARGO_BIN_EXE_charon6"))
        .args(["--recv", "--cidr", CIDR])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn receiver");

    std::thread::sleep(std::time::Duration::from_millis(500));

    // Start sender with stdin
    let mut sender = Command::new(env!("CARGO_BIN_EXE_charon6"))
        .args(["--send", "--cidr", CIDR])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn sender");

    sender
        .stdin
        .take()
        .unwrap()
        .write_all(MESSAGE)
        .expect("failed to write to sender stdin");

    sender.wait().expect("sender failed");

    std::thread::sleep(std::time::Duration::from_millis(300));
    receiver.kill().expect("failed to kill receiver");

    let mut output = Vec::new();
    std::io::Read::read_to_end(&mut receiver.stdout.take().unwrap(), &mut output)
        .expect("failed to read receiver stdout");
    receiver.wait().expect("failed to reap receiver");

    let text = String::from_utf8_lossy(&output);
    assert!(
        text.contains("hello\n"),
        "expected decoded message, got: {text:?}"
    );
}
