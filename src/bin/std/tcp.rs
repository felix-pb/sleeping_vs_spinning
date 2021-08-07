use benchmark::{Benchmark, ITERATIONS};
use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::{Duration, SystemTime};

const ADDR: &str = "127.0.0.1:12345";

pub fn sleeping() {
    let mut benchmark = Benchmark::new("std – tcp – sleeping");
    let tcp_listener = TcpListener::bind(ADDR).unwrap();
    std::thread::spawn(move || {
        let mut tcp_stream = tcp_listener.accept().unwrap().0;
        tcp_stream.set_nodelay(true).unwrap();
        for _ in 0..ITERATIONS {
            std::thread::sleep(Duration::from_millis(100));
            let t0 = SystemTime::now();
            let d0 = t0.duration_since(SystemTime::UNIX_EPOCH).unwrap();
            let buffer = d0.as_secs_f64().to_be_bytes();
            let n = tcp_stream.write(&buffer).unwrap();
            assert_eq!(n, 8);
        }
    });
    let mut tcp_stream = TcpStream::connect(ADDR).unwrap();
    tcp_stream.set_nodelay(true).unwrap();
    let mut buffer = [0; 8];
    loop {
        match tcp_stream.read(&mut buffer) {
            Ok(n) if n == 0 => break,
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
    let mut benchmark = Benchmark::new("std – tcp – spinning");
    let tcp_listener = TcpListener::bind(ADDR).unwrap();
    std::thread::spawn(move || {
        let mut tcp_stream = tcp_listener.accept().unwrap().0;
        tcp_stream.set_nodelay(true).unwrap();
        for _ in 0..ITERATIONS {
            std::thread::sleep(Duration::from_millis(100));
            let t0 = SystemTime::now();
            let d0 = t0.duration_since(SystemTime::UNIX_EPOCH).unwrap();
            let buffer = d0.as_secs_f64().to_be_bytes();
            let n = tcp_stream.write(&buffer).unwrap();
            assert_eq!(n, 8);
        }
    });
    let mut tcp_stream = TcpStream::connect(ADDR).unwrap();
    tcp_stream.set_nodelay(true).unwrap();
    tcp_stream.set_nonblocking(true).unwrap();
    let mut buffer = [0; 8];
    loop {
        match tcp_stream.read(&mut buffer) {
            Ok(n) if n == 0 => break,
            Ok(n) if n == 8 => {
                let t1 = SystemTime::now();
                let d0 = Duration::from_secs_f64(f64::from_be_bytes(buffer));
                let t0 = SystemTime::UNIX_EPOCH + d0;
                benchmark.add(t1.duration_since(t0).unwrap());
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => continue,
            _ => panic!(),
        }
    }
    benchmark.print();
}
