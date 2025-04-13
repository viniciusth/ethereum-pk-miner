use std::{
    fs::{File, OpenOptions},
    io::{BufReader, Write},
    sync::{Arc, mpsc},
    thread::{self, JoinHandle},
    time::Instant,
};

use rand::rng;
use ratatui::{
    text::Text,
    widgets::{Block, Paragraph, Widget},
};
use xorf::{BinaryFuse8, BinaryFuse16, BinaryFuse32, Filter};
use xxhash_rust::xxh3::xxh3_64;

use crate::{
    db::address_exists,
    generator::CryptoGenerator,
    measure,
    statistics::Strategy,
    utils::{addr_from_pk, encode_hex},
};

use super::Runner;

struct MinerRunner {
    threads: u8,
    pool: Vec<JoinHandle<()>>,
    checker: Option<JoinHandle<()>>,
    filter: Arc<dyn Filter<u64> + Send + Sync>,
}

impl Runner for MinerRunner {
    fn start(&mut self) -> color_eyre::Result<()> {
        let count = if self.threads > 0 {
            self.threads as usize
        } else {
            num_cpus::get().max(3) - 2
        };

        let (tx, rx) = mpsc::sync_channel(100);
        for _ in 0..count {
            let filter = self.filter.clone();
            let tx = tx.clone();
            self.pool.push(thread::spawn(|| {
                worker_thread(filter, tx);
            }));
        }

        self.checker.replace(thread::spawn(|| {
            checker_thread(rx);
        }));

        Ok(())
    }

    fn draw(&self, frame: &mut ratatui::Frame) -> color_eyre::Result<()> {
        let area = frame.area();
        let buffer = frame.buffer_mut();

        let tries = Strategy::random_statistics().tries();
        let false_positives = Strategy::random_statistics().false_positives();
        let tries_throughput = Strategy::random_statistics().tries_throughput();
        let actual_throughput = Strategy::random_statistics().overall_tries_throughput();
        let checks_throughput = Strategy::random_statistics().check_throughput();
        let mut others_throughput = Strategy::random_statistics().get_throughputs();
        others_throughput.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        let others = others_throughput
            .iter()
            .map(|(name, tp)| format!("{name}: {tp:.2}/s"));

        let lines = Text::from_iter(
            [
                format!("Active Threads: {}", self.pool.len() + 2),
                format!("Tries: {tries}, Throughput per thread: {tries_throughput:.2}/s, Total Throughput: {actual_throughput:.2}/s"),
                format!("False Positives: {false_positives}, Throughput: {checks_throughput:.2}/s"),
                "--- Other Metrics ---".to_string(),
            ]
            .into_iter()
            .chain(others),
        );
        Paragraph::new(lines)
            .block(Block::bordered().title("Application Status"))
            // .centered()
            .render(area, buffer);

        Ok(())
    }
}

pub fn new_miner_runner(threads: u8, fuse: u8, fuse_path: String) -> Box<dyn Runner> {
    let reader = BufReader::new(File::open(&fuse_path).unwrap());
    let filter: Arc<dyn Filter<u64> + Send + Sync> = match fuse {
        8 => {
            let filter: BinaryFuse8 =
                bincode::decode_from_reader(reader, bincode::config::standard()).unwrap();
            Arc::new(filter)
        }
        16 => {
            let filter: BinaryFuse16 =
                bincode::decode_from_reader(reader, bincode::config::standard()).unwrap();
            Arc::new(filter)
        }
        32 => {
            let filter: BinaryFuse32 =
                bincode::decode_from_reader(reader, bincode::config::standard()).unwrap();
            Arc::new(filter)
        }
        _ => unreachable!(),
    };
    Box::new(MinerRunner {
        pool: vec![],
        threads,
        checker: None,
        filter,
    })
}

pub fn worker_thread(filter: Arc<dyn Filter<u64>>, tx: mpsc::SyncSender<Strategy>) {
    let mut rng = rng();

    let mut iter = 0;
    let mut addr = [0; 20];
    loop {
        let start = Instant::now();
        iter += 1;
        if iter == 100_000 {
            iter = 1;
            measure! {
                "worker.reseed"
                {
                    rng.reseed().unwrap();
                }
            }
        }

        let pk = rng.generate_pk();
        addr_from_pk(&pk, &mut addr);
        let hsh = measure! {
            "worker.xxh3_64"
            {
                xxh3_64(&addr)
            }
        };

        let msg = Strategy::Random {
            rng_info: "ThreadRng".into(),
            pk,
            addr,
        };

        measure! {
            "worker.filter.contains"
            {
                if filter.contains(&hsh) {
                    tx.send(msg).expect("checker shouldn't have died");
                }
            }
        }

        Strategy::random_statistics().add_try(start.elapsed());
    }
}

pub fn checker_thread(rx: mpsc::Receiver<Strategy>) {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("./data/to_check")
        .unwrap();
    while let Ok(msg) = rx.recv() {
        let start = Instant::now();
        match &msg {
            Strategy::Random { rng_info, pk, addr } => {
                let addr = encode_hex(addr);
                if address_exists(&format!("0x{addr}")) {
                    let pk = encode_hex(pk);
                    let msg = format!("pk: {pk}, addr: {addr}, info: {rng_info}");
                    let err_msg = format!("failed to write: {msg}");
                    writeln!(file, "{msg}").expect(&err_msg);
                    file.flush().expect(&err_msg);
                }
            }
            _ => unreachable!(),
        };
        msg.statistics().add_check(false, start.elapsed());
    }
}
