use charon6::{Ipv6Cidr, capture_loop, open_ipv6_packet_socket};

use std::env;

const DEFAULT_DEVICE: &str = "lo";

fn main() {
    #[cfg(not(target_os = "linux"))]
    compile_error!("charon6 only supports Linux");

    println!("charon6 started");

    let (device_name, filter) = parse_args(env::args().skip(1));
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

    match &filter {
        Some(cidr) => println!("Listening for IPv6 packets on {device_name} matching {cidr}..."),
        None => println!("Listening for IPv6 packets on {device_name}..."),
    }

    if let Err(err) = capture_loop(&fd, filter.as_ref()) {
        eprintln!("capture error: {err}");
        std::process::exit(1);
    }
}

fn parse_args(args: impl Iterator<Item = String>) -> (String, Option<Ipv6Cidr>) {
    let mut device_name = DEFAULT_DEVICE.to_string();
    let mut filter = None;
    let mut args = args;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--cidr" => {
                let value = args.next().unwrap_or_else(|| {
                    eprintln!("--cidr requires a value");
                    std::process::exit(1);
                });
                match value.parse::<Ipv6Cidr>() {
                    Ok(cidr) => filter = Some(cidr),
                    Err(()) => {
                        eprintln!("invalid CIDR: {value}");
                        std::process::exit(1);
                    }
                }
            }
            _ => device_name = arg,
        }
    }

    (device_name, filter)
}
