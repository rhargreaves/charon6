use std::env;
use std::net::Ipv6Addr;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};

fn main() {
    #[cfg(not(target_os = "linux"))]
    compile_error!("charon6 only supports Linux");

    println!("charon6 started");
    let device_name = env::args().nth(1).unwrap_or_else(|| "lo".to_string());
    run(device_name);
}

fn run(device_name: String) {
    println!("Opening AF_PACKET socket for device: {device_name}");

    let fd = unsafe {
        libc::socket(
            libc::AF_PACKET,
            libc::SOCK_DGRAM,
            (libc::ETH_P_IPV6 as u16).to_be() as i32,
        )
    };

    if fd < 0 {
        let err = std::io::Error::last_os_error();
        eprintln!("Failed to create AF_PACKET socket: {err} (need root or CAP_NET_RAW)");
        std::process::exit(1);
    }

    // Wrap fd in OwnedFd so it closes on drop
    let socket_fd = unsafe { OwnedFd::from_raw_fd(fd) };

    // Get interface index
    let iface_name = match std::ffi::CString::new(device_name.clone()) {
        Ok(name) => name,
        Err(e) => {
            eprintln!("Invalid device name: {e}");
            std::process::exit(1);
        }
    };

    let ifindex = unsafe { libc::if_nametoindex(iface_name.as_ptr()) };
    if ifindex == 0 {
        let err = std::io::Error::last_os_error();
        eprintln!("Failed to find interface index for '{device_name}': {err}");
        std::process::exit(1);
    }

    // Bind socket to interface
    let mut sll: libc::sockaddr_ll = unsafe { std::mem::zeroed() };
    sll.sll_family = libc::AF_PACKET as u16;
    sll.sll_ifindex = ifindex as i32;
    sll.sll_protocol = (libc::ETH_P_IPV6 as u16).to_be();

    let bind_res = unsafe {
        libc::bind(
            socket_fd.as_raw_fd(),
            &sll as *const libc::sockaddr_ll as *const libc::sockaddr,
            std::mem::size_of::<libc::sockaddr_ll>() as libc::socklen_t,
        )
    };

    if bind_res < 0 {
        let err = std::io::Error::last_os_error();
        eprintln!("Failed to bind socket to '{device_name}': {err}");
        std::process::exit(1);
    }

    println!("Listening for IPv6 packets on {device_name}...");

    let mut buf = vec![0u8; 65536];
    loop {
        let n = unsafe {
            libc::recv(
                socket_fd.as_raw_fd(),
                buf.as_mut_ptr() as *mut libc::c_void,
                buf.len(),
                0,
            )
        };

        if n < 0 {
            let err = std::io::Error::last_os_error();
            eprintln!("recv error: {err}");
            continue;
        }

        let n = n as usize;
        if n >= 40 {
            let ip_header = &buf[0..40];
            if (ip_header[0] & 0xF0) == 0x60 {
                let src_bytes: [u8; 16] = ip_header[8..24].try_into().unwrap();
                let dst_bytes: [u8; 16] = ip_header[24..40].try_into().unwrap();
                let src = Ipv6Addr::from(src_bytes);
                let dst = Ipv6Addr::from(dst_bytes);
                println!("src={src} -> dst={dst}");
            }
        }
    }
}
