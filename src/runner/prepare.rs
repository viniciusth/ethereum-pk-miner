use std::{
    fs::File,
    io::BufWriter,
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
    handle: Option<thread::JoinHandle<()>>,
    fuse: u8,
}

#[derive(Clone)]
enum PrepareInfo {
    Nothing,
    Reading(u64, u64, Instant),
    Finished(u64, Duration),
}

impl Runner for PrepareRunner {
    fn start(&mut self) -> color_eyre::Result<()> {
        let info = self.info.clone();
        let csv_path = self.csv_path.clone();
        let fuse = self.fuse.clone();
        let handle = thread::spawn(move || run(info, csv_path, fuse));
        self.handle.replace(handle);
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

pub fn new_prepare_runner(csv_path: String, fuse: u8) -> Box<dyn Runner> {
    Box::new(PrepareRunner {
        csv_path,
        fuse,
        info: Arc::new(Mutex::new(PrepareInfo::Nothing)),
        handle: None,
    })
}

fn run(info: Arc<Mutex<PrepareInfo>>, csv_path: String, fuse: u8) {
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

    match fuse {
        8 => {
            let filter = xorf::BinaryFuse8::try_from(&filter_data).unwrap();
            drop(filter_data);
            let mut writer = BufWriter::new(File::create("./data/xorfilter8").unwrap());
            bincode::encode_into_std_write(filter, &mut writer, bincode::config::standard())
                .unwrap();
        }
        16 => {
            let filter = xorf::BinaryFuse16::try_from(&filter_data).unwrap();
            drop(filter_data);
            let mut writer = BufWriter::new(File::create("./data/xorfilter16").unwrap());
            bincode::encode_into_std_write(filter, &mut writer, bincode::config::standard())
                .unwrap();
        }
        32 => {
            let filter = xorf::BinaryFuse32::try_from(&filter_data).unwrap();
            drop(filter_data);
            let mut writer = BufWriter::new(File::create("./data/xorfilter32").unwrap());
            bincode::encode_into_std_write(filter, &mut writer, bincode::config::standard())
                .unwrap();
        }
        _ => unreachable!(),
    }

    *info.lock().unwrap() = PrepareInfo::Finished(iters, start.elapsed());
}
