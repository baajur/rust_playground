use crossbeam::channel::Receiver;
use std::io::Result;
use std::time::{Duration, Instant};

pub fn stats_loop(silent: bool, stats_rx: Receiver<usize>) -> Result<()> {
    let mut total_bytes = 0;
    let start = Instant::now();
    let mut timer = Timer::new();
    loop {
        let num_bytes = stats_rx.recv().unwrap();
        timer.update();
        let rate_per_second = num_bytes as f64 / timer.delta.as_secs_f64();
        total_bytes += num_bytes;

        if !silent && timer.ready {
            timer.ready = false;
            eprint!(
                "\r[after: {} total bytes: {} speed: {:.0}B/s]",
                start.elapsed().as_secs().as_time(),
                total_bytes,
                rate_per_second
            );
        }

        if num_bytes == 0 {
            break;
        }
    }
    if !silent {
        eprintln!();
    }
    Ok(())
}

trait TimeOutput {
    fn as_time(&self) -> String;
}

impl TimeOutput for u64 {
    fn as_time(&self) -> String {
        let (hours, left) = (*self / 3600, *self % 3600);
        let (minutes, seconds) = (left / 60, left % 60);
        format!("{}:{:02}:{:02}", hours, minutes, seconds)
    }
}

struct Timer {
    /// Last update moment.
    last_instant: Instant,
    /// Time duration between current and last update moments.
    delta: Duration,
    /// How often we want the timer to go off, or be ready.
    period: Duration,
    /// To keep time of how much time until the timer goes off.
    countdown: Duration,
    /// It tells if the timer is ready.
    ready: bool,
}

impl Timer {
    fn new() -> Self {
        let now = Instant::now();
        Self {
            last_instant: now,
            delta: Duration::default(),
            period: Duration::from_millis(1000),
            countdown: Duration::default(),
            ready: true,
        }
    }

    fn update(&mut self) {
        let now = Instant::now();
        self.delta = now - self.last_instant;
        self.last_instant = now;
        self.countdown = self.countdown.checked_sub(self.delta).unwrap_or_else(|| {
            self.ready = true;
            self.period
        });
    }
}