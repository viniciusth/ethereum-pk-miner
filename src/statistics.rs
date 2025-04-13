use std::{
    collections::HashMap,
    sync::{
        Arc, LazyLock, RwLock,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

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
///  - any other named timing average operation /s
pub struct Statistics {
    pub data: [StatisticsData; 2],
}

pub struct StatisticsData {
    /// Value is increased by worker threads.
    tries: AtomicU64,

    program_start: Instant,

    /// Value is increased by checker thread after verification.
    false_positives: AtomicU64,

    /// Value is increased by checker thread after verification.
    successes: AtomicU64,

    try_time_taken_ns: AtomicU64,
    check_time_taken_ns: AtomicU64,

    /// Write only on insertion, all other operations can be done on read.
    /// Map<name, (count, total_time)>
    others: RwLock<HashMap<String, (AtomicU64, AtomicU64)>>,
}

impl Default for StatisticsData {
    fn default() -> Self {
        Self {
            program_start: Instant::now(),
            tries: 0.into(),
            false_positives: 0.into(),
            successes: 0.into(),
            try_time_taken_ns: 0.into(),
            check_time_taken_ns: 0.into(),
            others: RwLock::new(HashMap::new()),
        }
    }
}

impl StatisticsData {
    pub fn add_try(&self, time: Duration) {
        self.tries.fetch_add(1, Ordering::Relaxed);
        let ns = time.as_nanos();
        self.try_time_taken_ns
            .fetch_add(ns as u64, Ordering::Relaxed);
    }

    pub fn add_check(&self, found: bool, time: Duration) {
        let ns = time.as_nanos();
        self.check_time_taken_ns
            .fetch_add(ns as u64, Ordering::Relaxed);
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

    /// Returns the average amount of tries per second per thread.
    pub fn tries_throughput(&self) -> f64 {
        let taken_secs = self.try_time_taken_ns.load(Ordering::Relaxed) as f64 / 1e9;
        self.tries() as f64 / taken_secs
    }

    /// Overall average amount of tries per second since execution start.
    pub fn overall_tries_throughput(&self) -> f64 {
        let taken_secs = self.program_start.elapsed().as_nanos() as f64 / 1e9;
        self.tries() as f64 / taken_secs
    }

    /// Returns the average amount of tries per second.
    pub fn check_throughput(&self) -> f64 {
        let taken_secs = self.check_time_taken_ns.load(Ordering::Relaxed) as f64 / 1e9;
        (self.false_positives() + self.successes()) as f64 / taken_secs
    }

    /// Adds a named timing to the structure, all values can be fetched through [get_throughputs]
    pub fn add_timing(&self, name: &str, time: Duration) {
        if let Some(x) = self.others.read().unwrap().get(name) {
            let ns = time.as_nanos() as u64;
            x.0.fetch_add(1, Ordering::Relaxed);
            x.1.fetch_add(ns, Ordering::Relaxed);
        } else {
            self.others
                .write()
                .unwrap()
                .entry(name.to_owned())
                .or_default();
            self.add_timing(name, time);
        }
    }

    /// Returns the throughput of all named timings, as operations/s
    pub fn get_throughputs(&self) -> Vec<(String, f64)> {
        self.others
            .read()
            .unwrap()
            .iter()
            .map(|(k, v)| {
                let taken_secs = v.1.load(Ordering::Relaxed) as f64 / 1e9;
                let count = v.0.load(Ordering::Relaxed);
                (k.to_owned(), count as f64 / taken_secs)
            })
            .collect()
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

#[macro_export]
macro_rules! measure {
    ($name:literal $code:block) => {{
        let _private_now = std::time::Instant::now();
        let res = $code;
        crate::statistics::Strategy::random_statistics().add_timing($name, _private_now.elapsed());
        res
    }};
}
