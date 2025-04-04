use std::{
    fs::File,
    io::Write,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use csv::Reader;
use ratatui::{
    style::{Style, Stylize},
    text::Text,
    widgets::{Block, Gauge, Paragraph, Widget},
};
use xxhash_rust::xxh3::xxh3_64;

use crate::utils::parse_eth_hex;

use super::Runner;

struct PrepareRunner {
    csv_path: String,
    info: Arc<Mutex<PrepareInfo>>,
    handle: Mutex<Option<thread::JoinHandle<()>>>,
}

#[derive(Clone)]
enum PrepareInfo {
    Nothing,
    Reading(u64, u64, Instant),
    Finished(u64, Duration),
}

impl Runner for PrepareRunner {
    fn start(&self) -> color_eyre::Result<()> {
        let info = self.info.clone();
        let csv_path = self.csv_path.clone();
        let handle = thread::spawn(|| run(info, csv_path));
        *self.handle.lock().unwrap() = Some(handle);
        Ok(())
    }

    fn draw(&self, frame: &mut ratatui::Frame) -> color_eyre::Result<()> {
        let info = self.info.lock().unwrap().clone();

        let area = frame.area();
        let buffer = frame.buffer_mut();
        match info {
            PrepareInfo::Nothing => {
                Paragraph::new("Setting up...")
                    .block(Block::bordered().title("Progress"))
                    .render(area, buffer);
            }
            PrepareInfo::Reading(read, total, instant) => Gauge::default()
                .block(Block::bordered().title(format!(
                    "Progress => bytes: {read}/{total} | elapsed: {}s",
                    instant.elapsed().as_secs()
                )))
                .gauge_style(Style::new().white().on_black().italic())
                .percent((read as f64 / total as f64 * 100.0).round() as u16)
                .render(area, buffer),
            PrepareInfo::Finished(total, duration) => {
                let lines = Text::from_iter([
                    format!("Time taken: {}s", duration.as_secs()),
                    format!("lines processed: {total}"),
                ]);
                Paragraph::new(lines)
                    .block(Block::bordered().title("Finished!"))
                    .render(area, buffer);
            }
        };

        Ok(())
    }
}

pub fn new_prepare_runner(csv_path: String) -> Box<dyn Runner> {
    Box::new(PrepareRunner {
        csv_path,
        info: Arc::new(Mutex::new(PrepareInfo::Nothing)),
        handle: Mutex::new(None),
    })
}

fn run(info: Arc<Mutex<PrepareInfo>>, csv_path: String) {
    let start = Instant::now();
    let file_size = File::open(&csv_path).unwrap().metadata().unwrap().len();

    let mut reader = Reader::from_path(&csv_path).unwrap().into_records();
    let mut iters = 0;
    let mut data = [0u8; 20];
    // Current amount of addresses in the csv, adjust if changed data.
    let mut filter_data = Vec::with_capacity(142849835);
    // const BITS_PER_MEGABYTE: u64 = 1024 * 1024 * 8;

    while let Some(Ok(c)) = reader.next() {
        parse_eth_hex(&c[1], &mut data);
        let hsh = xxh3_64(&data);
        filter_data.push(hsh);
        if iters % 1000 == 0 {
            *info.lock().unwrap() =
                PrepareInfo::Reading(reader.reader().position().byte(), file_size, start);
        }
        iters += 1;
    }

    let filter = xorf::BinaryFuse16::try_from(&filter_data);
    drop(filter_data);
    let mut file = File::create("./data/xorfilter16").unwrap();
    let serialized = bincode::encode_to_vec(filter, bincode::config::standard()).unwrap();
    file.write_all(&serialized).unwrap();

    *info.lock().unwrap() = PrepareInfo::Finished(iters, start.elapsed());
}
