use ratatui::Frame;

pub mod prepare;
pub mod miner;

pub trait Runner {
    fn start(&mut self) -> color_eyre::Result<()>;

    fn draw(&self, frame: &mut Frame) -> color_eyre::Result<()>;
}

