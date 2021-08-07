# Sleeping vs. Spinning

Spinning or ["busy-waiting"](https://en.wikipedia.org/wiki/Busy_waiting) is almost always described as a bad thing. Why waste an entire CPU core to essentially do nothing? Why not let the operating system put your thread to sleep and wake it up when it's ready to work? Unfortunately, "waking up" a thread takes some time. In addition, CPUs automatically reduce their clock speed when not busy, and it takes some time to "warm them up" again.

In this repository, I want to measure the cost of sleeping. I'll do so by comparing the latencies of 3 common "blocking/sleeping" abstractions with their busy-waiting equivalents:

* `std::sync::mpsc::channel` [1]
* `std::net::TcpStream` [2]
* `std::net::UdpSocket` [3] – I won't talk about it here, but the code is in `src/bin/std/udp.rs`

[1] Measured as the latency between `tx.send()` and the corresponding `rx.recv()`.<br/>
[2] Measured as the latency between `tx_stream.write(&buf)` and `rx_stream.read(&mut buf)`.<br/>
[3] Measured as the latency between `tx_socket.send(&buf)` and `rx_socket.recv(&mut buf)`.<br/>

To compare the results, I'm using the simple `Benchmark` struct defined in `src/lib.rs`. Each benchmark is measured for 99 iterations, after which I print a summary of the average, median, minimum, and maximum latencies.

## Channel Benchmark

For the mpsc channel, I'm measuring the latency it takes for the `Receiver` to receive the message sent by the `Sender`. Here's the relevant code from `src/bin/std/mpsc.rs`:

```rust
use benchmark::{Benchmark, ITERATIONS};
use std::sync::mpsc::TryRecvError;
use std::time::{Duration, Instant};

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
```

In the code above, the `for t0 in rx` iteration blocks the thread until the next message is ready. To busy-wait instead, we need to loop on the `try_recv` method of the `Receiver` struct. The code is also in `src/bin/std/mpsc.rs`, but here are the changes we need to make:

```diff
-   let mut benchmark = Benchmark::new("std – mpsc – sleeping");
+   let mut benchmark = Benchmark::new("std – mpsc – spinning");
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        for _ in 0..ITERATIONS {
            std::thread::sleep(Duration::from_millis(100));
            tx.send(Instant::now()).unwrap();
        }
    });
-   for t0 in rx {
-       benchmark.add(t0.elapsed());
-   }
+   loop {
+       match rx.try_recv() {
+           Ok(t0) => benchmark.add(t0.elapsed()),
+           Err(TryRecvError::Empty) => continue,
+           Err(TryRecvError::Disconnected) => break,
+       }
+   }
    benchmark.print();
```

On my macOS laptop, I get the following results:

```
[std – mpsc – sleeping]
avg = 81.087µs
mid = 91.04µs
min = 13.765µs
max = 210.085µs

[std – mpsc – spinning]
avg = 1.679µs
mid = 308ns
min = 250ns
max = 34.432µs
```

In this case, spinning gives us a 48x speedup for the average latency and a 295x speedup for the median latency.

## TCP Benchmark

For TCP, I'm measuring the latency it takes for the client socket to receive the message sent by the server socket. AFAIK, the `Instant` struct cannot be serialized in any meaningful way, so I'm using the `SystemTime` struct instead. Here's the relevant code from `src/bin/std/tcp.rs`:

```rust
use benchmark::{Benchmark, ITERATIONS};
use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::{Duration, SystemTime};

const ADDR: &str = "127.0.0.1:12345";

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
```

In the code above, `tcp_stream.read(&mut buffer)` blocks the thread until some data is ready. To busy-wait instead, we need to use the `set_nonblocking` method of the `TcpStream` struct. The code is also in `src/bin/std/tcp.rs`, but here are the changes we need to make:

```diff
-   let mut benchmark = Benchmark::new("std – tcp – sleeping");
+   let mut benchmark = Benchmark::new("std – tcp – spinning");
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
+   tcp_stream.set_nonblocking(true).unwrap();
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
+           Err(e) if e.kind() == ErrorKind::WouldBlock => continue,
            _ => panic!(),
        }
    }
    benchmark.print();
```

On my macOS laptop, I get the following results:

```
[std – tcp – sleeping]
avg = 219.046µs
mid = 237.144µs
min = 70.168µs
max = 275.96µs

[std – tcp – spinning]
avg = 43.055µs
mid = 39.152µs
min = 22.024µs
max = 89.96µs
```

In this case, spinning gives us a 5x speedup for the average latency and a 6x speedup for the median latency.

## What About Async?

Asynchronous Rust is often described as non-blocking, i.e. when we `.await` on an I/O operation, the underlying OS thread can start working on another task rather than going to sleep. So maybe that could be a faster alternative? Let's find out! I've also added benchmarks for the 3 equivalent abstractions in the [tokio](https://tokio.rs/) library:

* `tokio::sync::mpsc::unbounded_channel`
* `tokio::net::TcpStream`
* `tokio::net::UdpSocket`

For the sake of brevity, I'll only talk about TCP here since async/await is mostly used for network I/O. Here's the relevant code from `src/bin/tokio/tcp.rs`:

```rust
use benchmark::{Benchmark, ITERATIONS};
use std::io::ErrorKind;
use std::time::{Duration, SystemTime};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

const ADDR: &str = "127.0.0.1:12345";

let mut benchmark = Benchmark::new("tokio – tcp – sleeping");
let tcp_listener = TcpListener::bind(ADDR).await.unwrap();
tokio::spawn(async move {
    let mut tcp_stream = tcp_listener.accept().await.unwrap().0;
    tcp_stream.set_nodelay(true).unwrap();
    for _ in 0..ITERATIONS {
        tokio::time::sleep(Duration::from_millis(100)).await;
        let t0 = SystemTime::now();
        let d0 = t0.duration_since(SystemTime::UNIX_EPOCH).unwrap();
        let buffer = d0.as_secs_f64().to_be_bytes();
        let n = tcp_stream.write(&buffer).await.unwrap();
        assert_eq!(n, 8);
    }
});
let mut tcp_stream = TcpStream::connect(ADDR).await.unwrap();
tcp_stream.set_nodelay(true).unwrap();
let mut buffer = [0; 8];
loop {
    match tcp_stream.read(&mut buffer).await {
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
```

On my macOS laptop, I get the following results:

```
[tokio – tcp – sleeping]
avg = 233.562µs
mid = 237.128µs
min = 88.016µs
max = 322.784µs
```

Both the average and the median latencies are very similar to their `std` equivalents. This should not come as a surprise! The main benefit of async/await is that if a task needs to wait on an I/O event, the underlying OS thread can work on some other task instead of going to sleep. In short, `tokio` is great for dealing with a _large_ number of I/O events (e.g. for web servers). However, in the example above, we are dealing with a _small_ number of I/O events and we just want to respond as fast as possible. In this case, there is no other work to do, so the underlying OS thread will go to sleep just the same.

## Conclusion

It's true that spinning/busy-waiting is almost always a bad thing. However, sometimes you don't need to handle tons of events. Instead, when an event _does_ occur, you want the lowest latency response possible. This is a common scenario in low-latency trading where participants are willing to sacrifice as many CPU cycles as necessary if it buys them a few microseconds. For those niche use cases, spinning might be useful!

## Appendix: Full Benchmark Output

* macOS v11.2.3:

```
[std – mpsc – sleeping]        [std – mpsc – spinning]      [speedup]
avg = 81.087µs                 avg = 1.679µs                48.3x
mid = 91.04µs                  mid = 0.308µs                295.6x
min = 13.765µs                 min = 0.250µs
max = 210.085µs                max = 34.432µs

[std – tcp – sleeping]         [std – tcp – spinning]       [speedup]
avg = 219.046µs                avg = 43.055µs               5.1x
mid = 237.144µs                mid = 39.152µs               6.1x
min = 70.168µs                 min = 22.024µs
max = 275.96µs                 max = 89.96µs

[std – udp – sleeping]         [std – udp – spinning]       [speedup]
avg = 182.356µs                avg = 38.29µs                4.8x
mid = 209.832µs                mid = 36.992µs               5.7x
min = 47.152µs                 min = 22.088µs
max = 239.944µs                max = 71.024µs

[tokio – mpsc – sleeping]      [tokio – mpsc – spinning]    [speedup]
avg = 55.702µs                 avg = 0.895µs                62.2x
mid = 60.325µs                 mid = 0.350µs                172.4x
min = 11.327µs                 min = 0.252µs
max = 146.688µs                max = 20.651µs

[tokio – tcp – sleeping]       [tokio – tcp – spinning]     [speedup]
avg = 233.562µs                avg = 38.82µs                6.0x
mid = 237.128µs                mid = 37.048µs               6.4x
min = 88.016µs                 min = 23.928µs
max = 322.784µs                max = 90.944µs

[tokio – udp – sleeping]       [tokio – udp – spinning]     [speedup]
avg = 239.136µs                avg = 37.733µs               6.3x
mid = 241.008µs                mid = 37.104µs               6.5x
min = 184.008µs                min = 23.976µs
max = 320.816µs                max = 86.152µs
```

* ubuntu v18.04.5

```
[std – mpsc – sleeping]        [std – mpsc – spinning]      [speedup]
avg = 4.248µs                  avg = 1.257µs                3.4x
mid = 3.974µs                  mid = 1.225µs                3.2x
min = 3.706µs                  min = 0.782µs
max = 12.487µs                 max = 9.24µs

[std – tcp – sleeping]         [std – tcp – spinning]       [speedup]
avg = 8.913µs                  avg = 5.866µs                1.5x
mid = 8.954µs                  mid = 6.027µs                1.5x
min = 6.54µs                   min = 2.813µs
max = 16.397µs                 max = 6.908µs

[std – udp – sleeping]         [std – udp – spinning]       [speedup]
avg = 6.14µs                   avg = 3.438µs                1.8x
mid = 5.693µs                  mid = 3.33µs                 1.7x
min = 5.072µs                  min = 2.808µs
max = 34.322µs                 max = 11.138µs

[tokio – mpsc – sleeping]      [tokio – mpsc – spinning]    [speedup]
avg = 4.167µs                  avg = 0.494µs                8.4x
mid = 4.059µs                  mid = 0.443µs                9.2x
min = 3.706µs                  min = 0.284µs
max = 9.131µs                  max = 5.179µs

[tokio – tcp – sleeping]       [tokio – tcp – spinning]     [speedup]
avg = 12.512µs                 avg = 7.712µs                1.6x
mid = 11.692µs                 mid = 7.429µs                1.6x
min = 8.935µs                  min = 4.688µs
max = 24.183µs                 max = 16.496µs

[tokio – udp – sleeping]       [tokio – udp – spinning]     [speedup]
avg = 9.597µs                  avg = 5.63µs                 1.7x
mid = 9.232µs                  mid = 5.461µs                1.7x
min = 8.758µs                  min = 4.006µs
max = 22.58µs                  max = 20.265µs
```
