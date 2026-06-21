use std::io::Read;
use std::net::Ipv6Addr;

fn main() {
    #[cfg(not(target_os = "linux"))]
    compile_error!("charon6 only supports Linux");
    println!("charon6 started");

    const TUN_NAME: &str = "tun0";

    let mut config = tun::Configuration::default();
    config
        .tun_name(TUN_NAME)
        .up()
        .mtu(1500)
        .layer(tun::Layer::L3);

    let mut dev = tun::Device::new(&config)
        .expect("failed to create TUN device (need root or CAP_NET_ADMIN)");

    println!("listening for IPv6 packets on {TUN_NAME}...");

    let mut buf = vec![0u8; 2000];
    loop {
        let n = dev.read(&mut buf).expect("read error");
        if n >= 40 {
            let src_bytes: [u8; 16] = buf[8..24].try_into().unwrap();
            let src = Ipv6Addr::from(src_bytes);
            println!("{src}");
        }
    }
}
