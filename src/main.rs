use std::time::Duration;

use eth_pk_miner::runner::{Runner, miner::new_miner_runner, prepare::new_prepare_runner};
use clap::{Parser, Subcommand};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::DefaultTerminal;

const RAW_DATA_PATH_FROM_ROOT: &str = "./data/accounts.csv";
const EXIT_KEY: KeyEvent = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    cmd: CliCommands,
}

#[derive(Subcommand, Debug)]
enum CliCommands {
    Prepare {
        /// Solution file to expand
        #[arg(short, long, default_value = RAW_DATA_PATH_FROM_ROOT)]
        csv_path: String,

        /// Which binary fuse to use, must be a value of 8, 16, 32.
        #[arg(short, long, default_value_t = 16)]
        fuse: u8,

        /// Where to save the fuse, defaults to `./data/xorfilter{fuse}`
        #[arg(long, default_value = "")]
        fuse_path: String,
    },

    Miner {
        /// How many worker threads should be spawned, if empty will use the num_cpus crate.
        #[arg(short, long, default_value_t = 0)]
        threads: u8,

        /// Which binary fuse to use, must be a value of 8, 16, 32.
        #[arg(short, long, default_value_t = 16)]
        fuse: u8,

        /// Where the fuse is saved, if empty will read `./data/xorfilter{fuse}`
        #[arg(long, default_value = "")]
        fuse_path: String,
    },
}

fn main() {
    let cli = Cli::parse();
    color_eyre::install().expect("color_eyre works");
    let terminal = ratatui::init();
    let result = run(terminal, cli);
    ratatui::restore();
    result.expect("Terminal loop didn't break");
}

fn run(mut terminal: DefaultTerminal, cli: Cli) -> color_eyre::Result<()> {
    let mut runner: Box<dyn Runner> = match cli.cmd {
        CliCommands::Prepare {
            csv_path,
            fuse,
            mut fuse_path,
        } => {
            if ![8, 16, 32].contains(&fuse) {
                return Err(clap::Error::new(clap::error::ErrorKind::InvalidValue).into());
            }

            if fuse_path.is_empty() {
                fuse_path = format!("./data/xorfilter{fuse}");
            }

            new_prepare_runner(csv_path, fuse, fuse_path)
        }
        CliCommands::Miner {
            threads,
            fuse,
            mut fuse_path,
        } => {
            if ![8, 16, 32].contains(&fuse) {
                return Err(clap::Error::new(clap::error::ErrorKind::InvalidValue).into());
            }

            if fuse_path.is_empty() {
                fuse_path = format!("./data/xorfilter{fuse}");
            }

            new_miner_runner(threads, fuse, fuse_path)
        }
    };

    runner.start()?;
    loop {
        terminal.draw(|f| {
            runner.draw(f).expect("Runner shouldnt fail draw");
        })?;

        let has_event = event::poll(Duration::from_millis(100))?;

        if has_event && event::read()? == Event::Key(EXIT_KEY) {
            break Ok(());
        }
    }
}
