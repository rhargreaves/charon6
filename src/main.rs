use charon6::{Ipv6Cidr, capture_loop, open_ipv6_packet_socket, send_message};

use clap::Parser;

const DEFAULT_DEVICE: &str = "lo";

#[derive(Parser)]
#[command(about = "Encode/decode messages in IPv6 destination addresses")]
struct Args {
    #[arg(default_value = DEFAULT_DEVICE, help = "Interface to capture on (recv mode)")]
    device: String,

    #[arg(
        long,
        value_name = "IPv6 CIDR",
        help = "IPv6 /64 CIDR for encoding/decoding"
    )]
    cidr: Ipv6Cidr,

    #[arg(short, long, help = "Send mode: read stdin, encode to packets")]
    send: bool,

    #[arg(short, long, help = "Receive mode: decode packets to stdout")]
    recv: bool,
}

fn main() {
    #[cfg(not(target_os = "linux"))]
    compile_error!("charon6 only supports Linux");

    let args = Args::parse();

    if args.send && args.recv {
        eprintln!("error: cannot use --send and --recv together");
        std::process::exit(1);
    }

    if args.send {
        run_send(&args.cidr);
    } else {
        run_recv(&args.device, &args.cidr);
    }
}

fn run_send(cidr: &Ipv6Cidr) {
    use std::io::Read;

    let mut input = Vec::new();
    if let Err(err) = std::io::stdin().read_to_end(&mut input) {
        eprintln!("failed to read stdin: {err}");
        std::process::exit(1);
    }

    if let Err(err) = send_message(cidr, &input) {
        eprintln!("send error: {err}");
        std::process::exit(1);
    }
}

fn run_recv(device: &str, cidr: &Ipv6Cidr) {
    eprintln!("charon6 started");
    eprintln!("Opening AF_PACKET socket for device: {device}");

    let fd = match open_ipv6_packet_socket(device) {
        Ok(fd) => fd,
        Err(err) => {
            eprintln!(
                "Failed to open AF_PACKET socket for '{device}': {err} \
                 (need root or CAP_NET_RAW)",
            );
            std::process::exit(1);
        }
    };

    eprintln!("Listening for IPv6 packets on {device} decoding {cidr}...");

    if let Err(err) = capture_loop(&fd, cidr) {
        eprintln!("capture error: {err}");
        std::process::exit(1);
    }
}
