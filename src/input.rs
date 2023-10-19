use ratatui::{
    backend::Backend,
    style::Style,
    terminal::Frame,
    text::{Line, Span},
};
use regex::Regex;
use unicode_width::UnicodeWidthStr;

use crate::popup::{centered_rect, render_popup};

#[derive(Debug, Default)]
pub struct Input {
    /// Current value of the input box
    pub input: String,
}

impl Input {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn render<B: Backend>(&self, frame: &mut Frame<'_, B>) {
        let pos = (40, 8);
        let area = centered_rect(pos.0, pos.1, frame.size());
        let text = vec![Line::from(Span::styled(
            self.input.clone(),
            Style::default(),
        ))];

        let title = if self.is_valid() {
            "Input"
        } else {
            "Input (non valid regexp)"
        };
        frame.set_cursor(area.x + self.input.width() as u16 + 1, area.y + 1);
        render_popup(frame, title, &text, (pos.0, pos.1));
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

    pub fn is_valid(&self) -> bool {
        Regex::new(&self.input).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use ratatui::{backend::TestBackend, buffer::Buffer, prelude::Color, Terminal};

    use super::*;

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
        let backend = TestBackend::new(20, 37);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| input.render(f)).unwrap();
        let mut expected = Buffer::with_lines(vec![
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
        ]);

        for x in 6..=13 {
            for y in 17..=19 {
                expected.get_mut(x, y).set_fg(Color::White);
                expected.get_mut(x, y).set_bg(Color::Black);
            }
        }

        terminal.backend().assert_buffer(&expected);
    }

    #[test]
    fn test_render_non_valid_input() {
        let mut input = Input::new();
        input.push('[');
        let backend = TestBackend::new(65, 37);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| input.render(f)).unwrap();
        let mut expected = Buffer::with_lines(vec![
            "                                                                 ",
            "                                                                 ",
            "                                                                 ",
            "                                                                 ",
            "                                                                 ",
            "                                                                 ",
            "                                                                 ",
            "                                                                 ",
            "                                                                 ",
            "                                                                 ",
            "                                                                 ",
            "                                                                 ",
            "                                                                 ",
            "                                                                 ",
            "                                                                 ",
            "                                                                 ",
            "                                                                 ",
            "                    ┌Input (non valid regexp)┐                   ",
            "                    │[                       │                   ",
            "                    └────────────────────────┘                   ",
            "                                                                 ",
            "                                                                 ",
            "                                                                 ",
            "                                                                 ",
            "                                                                 ",
            "                                                                 ",
            "                                                                 ",
            "                                                                 ",
            "                                                                 ",
            "                                                                 ",
            "                                                                 ",
            "                                                                 ",
            "                                                                 ",
            "                                                                 ",
            "                                                                 ",
            "                                                                 ",
            "                                                                 ",
        ]);
        for x in 20..=45 {
            for y in 17..=19 {
                expected.get_mut(x, y).set_fg(Color::White);
                expected.get_mut(x, y).set_bg(Color::Black);
            }
        }

        terminal.backend().assert_buffer(&expected);
    }
}
