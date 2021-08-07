use benchmark::{Benchmark, ITERATIONS};
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

pub async fn sleeping() {
    let mut benchmark = Benchmark::new("tokio – mpsc – sleeping");
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    tokio::spawn(async move {
        for _ in 0..ITERATIONS {
            tokio::time::sleep(Duration::from_millis(100)).await;
            tx.send(Instant::now()).unwrap();
        }
    });
    while let Some(t0) = rx.recv().await {
        benchmark.add(t0.elapsed());
    }
    benchmark.print();
}

pub async fn spinning() {
    let mut benchmark = Benchmark::new("tokio – mpsc – spinning");
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    tokio::spawn(async move {
        for _ in 0..ITERATIONS {
            tokio::time::sleep(Duration::from_millis(100)).await;
            tx.send(Instant::now()).unwrap();
        }
    });
    let waker = futures::task::noop_waker();
    let mut context = Context::from_waker(&waker);
    loop {
        match rx.poll_recv(&mut context) {
            Poll::Ready(Some(t0)) => benchmark.add(t0.elapsed()),
            Poll::Pending => continue,
            Poll::Ready(None) => break,
        }
    }
    benchmark.print();
}
