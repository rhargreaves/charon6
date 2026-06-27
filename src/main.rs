use charon6::{Ipv6Cidr, capture_loop, open_ipv6_packet_socket};

use clap::Parser;

const DEFAULT_DEVICE: &str = "lo";

#[derive(Parser)]
#[command(about = "Capture IPv6 packets and decode messages encoded in destination addresses")]
struct Args {
    #[arg(default_value = DEFAULT_DEVICE, help = "Interface to capture on")]
    device: String,

    #[arg(
        long,
        value_name = "IPv6 CIDR",
        help = "IPv6 CIDR used to decode destination addresses"
    )]
    cidr: Ipv6Cidr,
}

fn main() {
    #[cfg(not(target_os = "linux"))]
    compile_error!("charon6 only supports Linux");

    let args = Args::parse();

    eprintln!("charon6 started");
    eprintln!("Opening AF_PACKET socket for device: {}", args.device);

    let fd = match open_ipv6_packet_socket(&args.device) {
        Ok(fd) => fd,
        Err(err) => {
            eprintln!(
                "Failed to open AF_PACKET socket for '{}': {err} \
                 (need root or CAP_NET_RAW)",
                args.device
            );
            std::process::exit(1);
        }
    };

    eprintln!(
        "Listening for IPv6 packets on {} decoding {}...",
        args.device, args.cidr
    );

    if let Err(err) = capture_loop(&fd, &args.cidr) {
        eprintln!("capture error: {err}");
        std::process::exit(1);
    }
}
