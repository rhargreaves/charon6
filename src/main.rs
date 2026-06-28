use charon6::{Ipv6Cidr, capture_loop, open_ipv6_packet_socket, send_message};

use clap::Parser;

#[derive(Parser)]
#[command(about = "Encode/decode messages in IPv6 destination addresses")]
struct Args {
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

    #[arg(short, long, help = "Send UDP datagram rather than ICMP")]
    port: Option<u16>,
}

fn main() {
    #[cfg(not(target_os = "linux"))]
    compile_error!("charon6 only supports Linux");

    let args = Args::parse();

    if args.send && args.recv {
        eprintln!("error: cannot use --send and --recv together");
        std::process::exit(1);
    }

    if !args.send && !args.recv {
        eprintln!("error: specify --send or --recv");
        std::process::exit(1);
    }

    if args.send {
        run_send(&args.cidr, args.port);
    } else {
        run_recv(&args.cidr, args.port);
    }
}

fn run_send(cidr: &Ipv6Cidr, port: Option<u16>) {
    use std::io::Read;

    let mut input = Vec::new();
    if let Err(err) = std::io::stdin().read_to_end(&mut input) {
        eprintln!("failed to read stdin: {err}");
        std::process::exit(1);
    }

    if let Err(err) = send_message(cidr, &input, port) {
        eprintln!("send error: {err}");
        std::process::exit(1);
    }
}

fn run_recv(cidr: &Ipv6Cidr, port: Option<u16>) {
    eprintln!("charon6 started");

    let fd = match open_ipv6_packet_socket() {
        Ok(fd) => fd,
        Err(err) => {
            eprintln!("failed to open AF_PACKET socket: {err} (need root or CAP_NET_RAW?)");
            std::process::exit(1);
        }
    };

    match port {
        Some(p) => eprintln!("Listening for UDP/{p} packets, decoding {cidr}..."),
        None => eprintln!("Listening for ICMPv6 packets, decoding {cidr}..."),
    }

    if let Err(err) = capture_loop(&fd, cidr, port) {
        if err.kind() == std::io::ErrorKind::BrokenPipe {
            return;
        }
        eprintln!("capture error: {err}");
        std::process::exit(1);
    }
}
