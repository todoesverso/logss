use tui::backend::Backend;
use tui::layout::Rect;
use tui::style::Style;
use tui::terminal::Frame;
use tui::text::{Span, Spans};
use unicode_width::UnicodeWidthStr;

use crate::popup::{centered_rect, render_popup};

#[derive(Debug, Default)]
pub struct Input {
    /// Current value of the input box
    pub input: String,
}

impl Input {
    pub fn new() -> Self {
        Input::default()
    }

    pub fn render<B: Backend>(&self, frame: &mut Frame<'_, B>, area: Rect) {
        let pos = (40, 8);
        let area = centered_rect(pos.0, pos.1, area);
        let text = vec![Spans::from(Span::styled(
            self.input.clone(),
            Style::default(),
        ))];

        frame.set_cursor(area.x + self.input.width() as u16 + 1, area.y + 1);
        render_popup(frame, "Input", text, (pos.0, pos.1));
    }

    pub fn reset(&mut self) {
        self.input = String::new();
    }
}
