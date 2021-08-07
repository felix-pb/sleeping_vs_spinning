use benchmark::{Benchmark, ITERATIONS};
use std::io::ErrorKind;
use std::net::UdpSocket;
use std::time::{Duration, SystemTime};

const TX_ADDR: &str = "127.0.0.1:12345";
const RX_ADDR: &str = "127.0.0.1:12346";

pub fn sleeping() {
    let mut benchmark = Benchmark::new("std – udp – sleeping");
    let udp_socket = UdpSocket::bind(TX_ADDR).unwrap();
    std::thread::spawn(move || {
        udp_socket.connect(RX_ADDR).unwrap();
        for _ in 0..ITERATIONS {
            std::thread::sleep(Duration::from_millis(100));
            let t0 = SystemTime::now();
            let d0 = t0.duration_since(SystemTime::UNIX_EPOCH).unwrap();
            let buffer = d0.as_secs_f64().to_be_bytes();
            let n = udp_socket.send(&buffer).unwrap();
            assert_eq!(n, 8);
        }
    });
    let udp_socket = UdpSocket::bind(RX_ADDR).unwrap();
    let mut buffer = [0; 8];
    for _ in 0..ITERATIONS {
        match udp_socket.recv(&mut buffer) {
            Ok(n) if n == 8 => {
                let t1 = SystemTime::now();
                let d0 = Duration::from_secs_f64(f64::from_be_bytes(buffer));
                let t0 = SystemTime::UNIX_EPOCH + d0;
                benchmark.add(t1.duration_since(t0).unwrap());
            }
            _ => panic!(),
        }
    }
    benchmark.print();
}

pub fn spinning() {
    let mut benchmark = Benchmark::new("std – udp – spinning");
    let udp_socket = UdpSocket::bind(TX_ADDR).unwrap();
    std::thread::spawn(move || {
        udp_socket.connect(RX_ADDR).unwrap();
        for _ in 0..ITERATIONS {
            std::thread::sleep(Duration::from_millis(100));
            let t0 = SystemTime::now();
            let d0 = t0.duration_since(SystemTime::UNIX_EPOCH).unwrap();
            let buffer = d0.as_secs_f64().to_be_bytes();
            let n = udp_socket.send(&buffer).unwrap();
            assert_eq!(n, 8);
        }
    });
    let udp_socket = UdpSocket::bind(RX_ADDR).unwrap();
    udp_socket.set_nonblocking(true).unwrap();
    let mut buffer = [0; 8];
    let mut iterations = 0;
    loop {
        match udp_socket.recv(&mut buffer) {
            Ok(n) if n == 8 => {
                let t1 = SystemTime::now();
                let d0 = Duration::from_secs_f64(f64::from_be_bytes(buffer));
                let t0 = SystemTime::UNIX_EPOCH + d0;
                benchmark.add(t1.duration_since(t0).unwrap());
                iterations += 1;
                if iterations == ITERATIONS {
                    break;
                }
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => continue,
            _ => panic!(),
        }
    }
    benchmark.print();
}
