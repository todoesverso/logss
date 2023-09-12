use crate::app::{App, AppResult};
use crate::event::EventHandler;
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::Backend;
use ratatui::Terminal;
use std::io;

/// Representation of a terminal user interface.
///
/// It is responsible for setting up the terminal,
/// initializing the interface and handling the draw events.
#[derive(Debug)]
pub struct Tui<B: Backend> {
    /// Interface to the Terminal.
    terminal: Terminal<B>,
    /// Terminal event handler.
    pub events: EventHandler,
}

impl<B: Backend> Tui<B> {
    /// Constructs a new instance of [`Tui`].
    pub fn new(terminal: Terminal<B>, events: EventHandler) -> Self {
        Self { terminal, events }
    }

    /// Initializes the terminal interface.
    ///
    /// It enables the raw mode and sets terminal properties.
    pub fn init(&mut self) -> AppResult<()> {
        terminal::enable_raw_mode()?;
        crossterm::execute!(io::stderr(), EnterAlternateScreen)?;
        self.terminal.hide_cursor()?;
        self.terminal.clear()?;
        self.events.init();
        Ok(())
    }

    /// [`Draw`] the terminal interface by [`rendering`] the widgets.
    ///
    /// [`Draw`]: ratatui::Terminal::draw
    /// [`rendering`]: crate::app::App::render
    pub fn draw(&mut self, app: &mut App) -> AppResult<()> {
        self.terminal.draw(|frame| app.render(frame))?;
        Ok(())
    }

    /// Exits the terminal interface.
    ///
    /// It disables the raw mode and reverts back the terminal properties.
    pub fn exit(&mut self) -> AppResult<()> {
        terminal::disable_raw_mode()?;
        crossterm::execute!(io::stderr(), LeaveAlternateScreen)?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::args::parse_args;
    use ratatui::{
        backend::TestBackend,
        buffer::Buffer,
        style::{Color, Modifier, Style},
        Terminal,
    };

    #[test]
    fn new() {
        let backend = TestBackend::new(13, 10);
        let terminal = Terminal::new(backend).unwrap();
        let events = EventHandler::new(1);
        let mut app = App::new(Some(parse_args()));
        app.raw_buffer.state.color = Color::White;

        let mut tui = Tui::new(terminal, events);
        //tui.init().unwrap(); // This fails in github tests
        tui.draw(&mut app).unwrap();
        let mut expected = Buffer::with_lines(vec![
            "┌(0) - '.*' ┐",
            "│           │",
            "│           │",
            "│           │",
            "│           │",
            "│           │",
            "│           │",
            "│           │",
            "│           │",
            "└───────────┘",
        ]);
        let bolds = 1..=11;
        for x in 0..=12 {
            for y in 0..=9 {
                if bolds.contains(&x) && y == 0 {
                    expected
                        .get_mut(x, y)
                        .set_style(Style::default().add_modifier(Modifier::BOLD));
                }
                expected.get_mut(x, y).set_fg(Color::White);
                expected.get_mut(x, y).set_bg(Color::Black);
            }
        }

        tui.terminal.backend().assert_buffer(&expected);
        tui.exit().unwrap();
    }
}
