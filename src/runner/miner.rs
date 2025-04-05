use std::{fs::{File, OpenOptions}, io::{BufReader, Write}, sync::{mpsc, Arc}, thread::{self, JoinHandle}, time::Instant};

use rand::rng;
use ratatui::{text::Text, widgets::{Block, Paragraph, Widget}};
use xorf::{BinaryFuse16, Filter};
use xxhash_rust::xxh3::xxh3_64;

use crate::{generator::CryptoGenerator, statistics::Strategy, utils::{addr_from_pk, encode_hex}};

use super::Runner;

struct MinerRunner {
    pool: Vec<JoinHandle<()>>,
    checker: Option<JoinHandle<()>>,
    filter: Arc<BinaryFuse16>,
}

impl Runner for MinerRunner {
    fn start(&mut self) -> color_eyre::Result<()> {
        let count = num_cpus::get().max(3) - 2;
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
        let checks_throughput = Strategy::random_statistics().check_throughput();

        let lines = Text::from_iter([
            format!("Active Threads: {}", self.pool.len() + 2),
            format!("Tries: {tries}, Throughput: {tries_throughput:.2}/s"),
            format!("False Positives: {false_positives}, Throughput: {checks_throughput:.2}/s"),
        ]);
        Paragraph::new(lines)
            .block(Block::bordered().title("Application Status"))
            .centered()
            .render(area, buffer);


        Ok(())
    }
}

pub fn new_miner_runner() -> Box<dyn Runner> {
    let reader = BufReader::new(File::open("./data/xorfilter16").unwrap());
    let filter: BinaryFuse16 =
        bincode::decode_from_reader(reader, bincode::config::standard()).unwrap();
    Box::new(MinerRunner {
        pool: vec![],
        checker: None,
        filter: Arc::new(filter),
    })
}

pub fn worker_thread(filter: Arc<BinaryFuse16>, tx: mpsc::SyncSender<Strategy>) {
    let mut rng = rng();

    let mut iter = 0;
    let mut addr = [0; 20];
    loop {
        let start = Instant::now();
        iter += 1;
        if iter == 100_000 {
            iter = 1;
            rng.reseed().unwrap();
        }

        let pk = rng.generate_pk();
        addr_from_pk(&pk, &mut addr);
        let hsh = xxh3_64(&addr);
        let msg = Strategy::Random {
            rng_info: "ThreadRng".into(),
            pk,
            addr: addr.clone(),
        };

        if filter.contains(&hsh) {
            tx.send(msg).expect("checker shouldn't have died");
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
                let pk = encode_hex(pk);
                let msg = format!("pk: {pk}, addr: {addr}, info: {rng_info}");
                let err_msg = format!("failed to write: {msg}");
                writeln!(file, "{msg}").expect(&err_msg);
                file.flush().expect(&err_msg);
            },
            _ => unreachable!(),
        };
        msg.statistics().add_check(false, start.elapsed());
    }
}
