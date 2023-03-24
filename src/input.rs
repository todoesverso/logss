use ratatui::backend::Backend;
use ratatui::style::Style;
use ratatui::terminal::Frame;
use ratatui::text::{Span, Spans};
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

    pub fn render<B: Backend>(&self, frame: &mut Frame<'_, B>) {
        let pos = (40, 8);
        let area = centered_rect(pos.0, pos.1, frame.size());
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

    pub fn push(&mut self, ch: char) {
        self.input.push(ch);
    }
    pub fn pop(&mut self) {
        self.input.pop();
    }
    pub fn inner_clone(&self) -> String {
        self.input.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, buffer::Buffer, Terminal};

    #[test]
    fn simple_full_test() {
        let mut input = Input::new();
        assert_eq!(input.input, String::new());

        input.push('a');
        assert_eq!(input.input, "a");
        input.push('b');
        assert_eq!(input.input, "ab");
        input.pop();
        assert_eq!(input.input, "a");
        input.reset();
        assert_eq!(input.input, String::new());
    }

    #[test]
    fn test_render_input() {
        let mut input = Input::new();
        input.push('a');
        input.push('b');
        let backend = TestBackend::new(20, 38);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| input.render(f)).unwrap();
        let expected = Buffer::with_lines(vec![
            "                    ",
            "                    ",
            "                    ",
            "                    ",
            "                    ",
            "                    ",
            "                    ",
            "                    ",
            "                    ",
            "                    ",
            "                    ",
            "                    ",
            "                    ",
            "                    ",
            "                    ",
            "                    ",
            "                    ",
            "      ┌Input─┐      ",
            "      │ab    │      ",
            "      └──────┘      ",
            "                    ",
            "                    ",
            "                    ",
            "                    ",
            "                    ",
            "                    ",
            "                    ",
            "                    ",
            "                    ",
            "                    ",
            "                    ",
            "                    ",
            "                    ",
            "                    ",
            "                    ",
            "                    ",
            "                    ",
            "                    ",
        ]);
        terminal.backend().assert_buffer(&expected);
    }
}
