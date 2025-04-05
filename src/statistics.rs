use std::{sync::{atomic::{AtomicU64, Ordering}, Arc, LazyLock}, time::Duration};

pub static STATISTICS: LazyLock<Statistics> = LazyLock::new(|| Statistics {
    data: [StatisticsData::default(), StatisticsData::default()],
});

/// Handles statistics for multiple worker threads at once
/// Maintains for each strategy:
///  - number of tries
///  - number of false positives
///  - number of successes
///  - accumulated time taken (for average time per try) for try & check
///  - average tries/s
pub struct Statistics {
    pub data: [StatisticsData; 2],
}

#[derive(Default)]
pub struct StatisticsData {
    /// Value is increased by worker threads.
    tries: AtomicU64,

    /// Value is increased by checker thread after verification.
    false_positives: AtomicU64,

    /// Value is increased by checker thread after verification.
    successes: AtomicU64,

    try_time_taken_ns: AtomicU64,
    check_time_taken_ns: AtomicU64,
}

impl StatisticsData {
    pub fn add_try(&self, time: Duration) {
        self.tries.fetch_add(1, Ordering::Relaxed);
        let ns = time.as_nanos();
        self.try_time_taken_ns.fetch_add(ns as u64, Ordering::Relaxed);
    }

    pub fn add_check(&self, found: bool, time: Duration) {
        let ns = time.as_nanos();
        self.check_time_taken_ns.fetch_add(ns as u64, Ordering::Relaxed);
        if found {
            self.successes.fetch_add(1, Ordering::Relaxed);
        } else {
            self.false_positives.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn tries(&self) -> u64 {
        self.tries.load(Ordering::Relaxed)
    }

    pub fn successes(&self) -> u64 {
        self.successes.load(Ordering::Relaxed)
    }

    pub fn false_positives(&self) -> u64 {
        self.false_positives.load(Ordering::Relaxed)
    }

    /// Returns the average amount of tries per second.
    pub fn tries_throughput(&self) -> f64 {
        // floor it as u64
        let taken_secs = self.try_time_taken_ns.load(Ordering::Relaxed) / (1e9 as u64);
        self.tries() as f64 / taken_secs as f64
    }

    /// Returns the average amount of tries per second.
    pub fn check_throughput(&self) -> f64 {
        // floor it as u64
        let taken_secs = self.check_time_taken_ns.load(Ordering::Relaxed) / (1e9 as u64);
        (self.false_positives() + self.successes()) as f64 / taken_secs as f64
    }
}

pub enum Strategy {
    Random {
        rng_info: String,
        pk: [u8; 32],
        addr: [u8; 20],
    },

    /// Unused, was thinking of doing this but it doesn't really make that much sense
    Mnemonic {
        rng_info: String,
        mnemonic: Vec<Arc<str>>,
    },
}

impl Strategy {
    fn index(&self) -> usize {
        match self {
            Strategy::Random { .. } => 0,
            Strategy::Mnemonic { .. } => 1,
        }
    }

    pub fn statistics(&self) -> &StatisticsData {
        &STATISTICS.data[self.index()]
    }

    pub fn random_statistics() -> &'static StatisticsData {
        &STATISTICS.data[0]
    }
}
