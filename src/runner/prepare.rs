use std::{
    fs::File,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use csv::Reader;
use ratatui::{
    style::{Style, Stylize}, text::Text, widgets::{Block, Gauge, Paragraph, Widget}
};

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
                Paragraph::new("Setting up...").block(Block::bordered().title("Progress")).render(area, buffer);
            }
            PrepareInfo::Reading(read, total, instant) =>
                Gauge::default()
                    .block(Block::bordered().title(format!(
                        "Progress {read}/{total} ({}s)",
                        instant.elapsed().as_secs()
                    )))
                    .gauge_style(Style::new().white().on_black().italic())
                    .percent((read as f64 / total as f64 * 100.0).round() as u16).render(area, buffer),
            PrepareInfo::Finished(total, duration) => {
                let lines = Text::from_iter([
                    format!("Time taken: {}s", duration.as_secs()),
                    format!("Bytes read: {total}"),
                ]);
                Paragraph::new(lines).block(Block::bordered().title("Finished!")).render(area, buffer);
            },
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

    while let Some(Ok(c)) = reader.next() {
        c.len();
        if iters == 1000 {
            *info.lock().unwrap() =
                PrepareInfo::Reading(reader.reader().position().byte(), file_size, start);
            iters = 0;
        }
        iters += 1;
    }

    *info.lock().unwrap() = PrepareInfo::Finished(file_size, start.elapsed());
}
