use benchmark::{Benchmark, ITERATIONS};
use std::sync::mpsc::TryRecvError;
use std::time::{Duration, Instant};

pub fn sleeping() {
    let mut benchmark = Benchmark::new("std – mpsc – sleeping");
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        for _ in 0..ITERATIONS {
            std::thread::sleep(Duration::from_millis(100));
            tx.send(Instant::now()).unwrap();
        }
    });
    for t0 in rx {
        benchmark.add(t0.elapsed());
    }
    benchmark.print();
}

pub fn spinning() {
    let mut benchmark = Benchmark::new("std – mpsc – spinning");
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        for _ in 0..ITERATIONS {
            std::thread::sleep(Duration::from_millis(100));
            tx.send(Instant::now()).unwrap();
        }
    });
    loop {
        match rx.try_recv() {
            Ok(t0) => benchmark.add(t0.elapsed()),
            Err(TryRecvError::Empty) => continue,
            Err(TryRecvError::Disconnected) => break,
        }
    }
    benchmark.print();
}
