use std::time::Duration;

pub const ITERATIONS: usize = 99;

pub struct Benchmark {
    name: &'static str,
    durations: Vec<Duration>,
}

impl Benchmark {
    pub fn new(name: &'static str) -> Benchmark {
        Benchmark {
            name,
            durations: Vec::with_capacity(ITERATIONS),
        }
    }

    pub fn add(&mut self, duration: Duration) {
        self.durations.push(duration);
    }

    pub fn print(&mut self) {
        self.durations.sort();
        let len = self.durations.len();
        let avg = self.durations.iter().sum::<Duration>() / len as u32;
        let mid = self.durations.get(len / 2).unwrap();
        let min = self.durations.first().unwrap();
        let max = self.durations.last().unwrap();
        println!("[{}]", self.name);
        println!("avg = {:?}", avg);
        println!("mid = {:?}", mid);
        println!("min = {:?}", min);
        println!("max = {:?}", max);
        println!();
    }
}
