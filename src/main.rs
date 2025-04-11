use std::time::Duration;

use bip39miner::runner::{miner::new_miner_runner, prepare::new_prepare_runner, Runner};
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

        /// Whether to compute the fuse or not, must be a value of 8, 16, 32.
        #[arg(
            short,
            long,
            default_value_t = 0,
        )]
        fuse: u8,

        #[arg(short, long)]
        db: bool,
    },

    Miner {
    }
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
        CliCommands::Prepare { csv_path, fuse, db } => {
            if ![0, 8, 16, 32].contains(&fuse) {
                return Err(clap::Error::new(clap::error::ErrorKind::InvalidValue).into());
            }
            new_prepare_runner(csv_path, fuse, db)
        }
        CliCommands::Miner {  } => {
            new_miner_runner()
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
