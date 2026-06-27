use charon6::{capture_loop, open_ipv6_packet_socket};

use std::env;

fn main() {
    #[cfg(not(target_os = "linux"))]
    compile_error!("charon6 only supports Linux");

    println!("charon6 started");

    let device_name = env::args().nth(1).unwrap_or_else(|| "lo".to_string());
    println!("Opening AF_PACKET socket for device: {device_name}");

    let fd = match open_ipv6_packet_socket(&device_name) {
        Ok(fd) => fd,
        Err(err) => {
            eprintln!(
                "Failed to open AF_PACKET socket for '{device_name}': {err} \
                 (need root or CAP_NET_RAW)"
            );
            std::process::exit(1);
        }
    };

    println!("Listening for IPv6 packets on {device_name}...");

    if let Err(err) = capture_loop(&fd) {
        eprintln!("capture error: {err}");
        std::process::exit(1);
    }
}
