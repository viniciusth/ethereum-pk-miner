use ratatui::Frame;

pub mod prepare;

pub trait Runner {
    fn start(&self) -> color_eyre::Result<()>;

    fn draw(&self, frame: &mut Frame) -> color_eyre::Result<()>;
}

