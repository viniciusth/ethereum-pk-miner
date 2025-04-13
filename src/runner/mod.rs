use ratatui::Frame;

pub mod miner;
pub mod prepare;

pub trait Runner {
    fn start(&mut self) -> color_eyre::Result<()>;

    fn draw(&self, frame: &mut Frame) -> color_eyre::Result<()>;
}
