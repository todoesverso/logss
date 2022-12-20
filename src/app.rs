use std::error;
use tui::backend::Backend;
use tui::layout::Alignment;
use tui::style::{Color, Style};
use tui::terminal::Frame;
use tui::text::Spans;
use tui::widgets::{Block, Borders, Paragraph};

use crate::cb::CircularBuffer;
use crate::tstdin::StdinHandler;

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

/// Application.
#[derive(Debug)]
pub struct App<'a> {
    /// Is the application running?
    pub running: bool,
    stdin: StdinHandler,
    raw_buffer: CircularBuffer<Spans<'a>>,
}

impl<'a> Default for App<'a> {
    fn default() -> Self {
        Self {
            running: true,
            stdin: StdinHandler::new(),
            raw_buffer: CircularBuffer::new(128),
        }
    }
}

impl<'a> App<'a> {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&mut self) {
        self.get_stdin();
    }

    pub fn get_stdin(&mut self) {
        match self.stdin.try_recv() {
            Ok(line) => {
                self.raw_buffer.push(Spans::from(line));
            }
            Err(_) => {
                ();
            }
        }
    }

    /// Renders the user interface widgets.
    pub fn render<B: Backend>(&mut self, frame: &mut Frame<'_, B>) {
        frame.render_widget(
            Paragraph::new(self.raw_buffer.buffer.clone())
                .block(Block::default().borders(Borders::ALL))
                .style(Style::default().fg(Color::White).bg(Color::Black)),
            frame.size(),
        )
    }
}
