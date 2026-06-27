mod common;

use std::net::Ipv6Addr;

use common::{capture_with, encode_dst, send_raw, send_recv};

#[test]
fn decodes_single_packet_message() {
    const CIDR: &str = "2001:db8:1::/64";

    let text = capture_with(CIDR, || {
        send_raw(&[encode_dst(CIDR, 0, b"hi!")]);
    });

    assert!(
        text.contains("hi!\n"),
        "expected decoded message on stdout, got: {text:?}"
    );
}

#[test]
fn decodes_two_packet_message() {
    const CIDR: &str = "2001:db8:2::/64";

    let text = capture_with(CIDR, || {
        send_raw(&[
            "2001:db8:2::6:6865:6c6c:6f20".parse().unwrap(),
            "2001:db8:2::105:776f:726c:6400".parse().unwrap(),
        ]);
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
    let dsts: Vec<Ipv6Addr> = message
        .chunks(PAYLOAD_PER_PACKET)
        .enumerate()
        .map(|(seq, chunk)| encode_dst(CIDR, seq as u8, chunk))
        .collect();
    assert_eq!(dsts.len(), TOTAL_PACKETS);

    let text = capture_with(CIDR, || {
        send_raw(&dsts);
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

    let text = capture_with(CIDR, || {
        send_raw(&[
            encode_dst(CIDR, 2, b"world"), // terminator (len < 6)
            encode_dst(CIDR, 0, b"hello "),
            encode_dst(CIDR, 1, b"cruel "),
        ]);
    });

    assert!(
        text.contains("hello cruel world\n"),
        "expected reordered message on stdout, got: {text:?}"
    );
}

#[test]
fn incomplete_message_produces_no_output() {
    const CIDR: &str = "2001:db8:5::/64";

    let text = capture_with(CIDR, || {
        // Send terminator (seq 2) but missing seq 1 - message incomplete
        send_raw(&[encode_dst(CIDR, 2, b"end"), encode_dst(CIDR, 0, b"start ")]);
    });

    assert!(
        text.is_empty(),
        "expected no output for incomplete message, got: {text:?}"
    );
}

#[test]
fn send_mode_encodes_stdin_to_packets() {
    const CIDR: &str = "2001:db8:6::/64";

    let text = send_recv(CIDR, b"hello", &[]);
    assert!(
        text.contains("hello\n"),
        "expected decoded message, got: {text:?}"
    );
}

#[test]
fn send_mode_uses_custom_port() {
    const CIDR: &str = "2001:db8:7::/64";

    let text = send_recv(CIDR, b"port!", &["--port", "7777"]);
    assert!(
        text.contains("port!\n"),
        "expected decoded message with custom port, got: {text:?}"
    );
}
