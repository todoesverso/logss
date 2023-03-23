use ratatui::backend::Backend;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::terminal::Frame;
use ratatui::text::{Span, Spans};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::cb::CircularBuffer;
use crate::states::ContainerState;

#[derive(Debug)]
pub struct Container<'a> {
    /// matching text
    text: String,
    /// circular buffer with matching lines
    pub cb: CircularBuffer<Spans<'a>>,
    pub id: u8,
    pub state: ContainerState,
}

pub const CONTAINER_BUFFER: usize = 64;
pub const CONTAINERS_MAX: u8 = 10;
pub const CONTAINER_COLORS: [ratatui::style::Color; 10] = [
    Color::Red,
    Color::Blue,
    Color::Cyan,
    Color::Green,
    Color::Yellow,
    Color::LightYellow,
    Color::Magenta,
    Color::LightMagenta,
    Color::Gray,
    Color::DarkGray,
];

impl<'a> Container<'a> {
    pub fn new(text: String, buffersize: usize) -> Self {
        Self {
            text,
            cb: CircularBuffer::new(buffersize),
            id: 0,
            state: ContainerState::default(),
        }
    }

    fn process_line(&self, line: String) -> Option<Spans<'a>> {
        // TODO: maybe add smart time coloration?
        let contains_index = line.find(&self.text);

        contains_index.map(|index| {
            Spans(vec![
                Span::from(line[0..index].to_string()),
                Span::styled(self.text.clone(), Style::default().fg(self.state.color)),
                Span::from(line[index + self.text.len()..].to_string()),
            ])
        })
    }

    pub fn push(&mut self, element: Spans<'a>) {
        let _ = &self.cb.push(element);
    }

    pub fn proc_and_push_line(&mut self, line: String) {
        let sp = self.process_line(line);
        if let Some(spans) = sp {
            let _ = &self.push(spans);
        }
    }

    pub fn update_scroll(&mut self, size: usize, up: &mut u16, down: &mut u16) {
        // TODO: rewrite this mess
        let bufflen = self.cb.len();

        if bufflen < size {
        } else {
            self.state.scroll = (bufflen - size) as u16;
            if self.state.scroll - *down > 0 {
                self.state.scroll -= *down;
            } else if *down > 1 {
                *down -= 1;
            }

            if self.state.scroll + *up <= (bufflen - size) as u16 {
                self.state.scroll += *up;
            } else if *up > 1 {
                *up -= 1;
            }
        }
    }

    pub fn render<B: Backend>(&self, frame: &mut Frame<'_, B>, area: Rect) {
        let title = format!("({}) - {}", self.id, self.text);
        let mut paragraph = Paragraph::new(self.cb.ordered_clone().buffer.clone())
            .block(create_block(&title, self.state.color, self.state.paused))
            .style(self.state.style)
            .scroll((self.state.scroll, 0));
        if self.state.wrap {
            paragraph = paragraph.wrap(Wrap { trim: false });
        }

        frame.render_widget(paragraph, area);
    }
}

fn create_block(title: &str, color: Color, paused: bool) -> Block {
    let mut modifier = Modifier::BOLD;
    if paused {
        modifier = Modifier::BOLD | Modifier::SLOW_BLINK;
    }
    Block::default().borders(Borders::ALL).title(Span::styled(
        title,
        Style::default().add_modifier(modifier).fg(color),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_block() {
        let block = create_block("sarasa", Color::Red, false);
        let expected = Block::default().borders(Borders::ALL).title(Span::styled(
            "sarasa",
            Style::default().add_modifier(Modifier::BOLD).fg(Color::Red),
        ));
        assert_eq!(block, expected);

        let block = create_block("coso", Color::Blue, true);
        let expected = Block::default().borders(Borders::ALL).title(Span::styled(
            "coso",
            Style::default()
                .add_modifier(Modifier::BOLD | Modifier::SLOW_BLINK)
                .fg(Color::Blue),
        ));
        assert_eq!(block, expected);
    }

    #[test]
    fn test_container_new() {
        let container = Container::new("key".to_string(), 2);
        assert_eq!(container.id, 0);
        assert_eq!(container.text, "key");
        assert_eq!(container.cb.len(), 0);
        assert_eq!(container.cb.capacity(), 2);
        assert_eq!(container.state, ContainerState::default());
    }

    #[test]
    fn process_line() {
        let container = Container::new("stringtomatch".to_string(), 2);
        let span = container.process_line("this line should not be proc".to_string());
        assert_eq!(span, None);
        let span = container.process_line("stringtomatch this line should be proc".to_string());
        let expected_span = Some(Spans(vec![
            Span::from("".to_string()),
            Span::styled("stringtomatch".to_string(), Style::default().fg(Color::Red)),
            Span::from(" this line should be proc".to_string()),
        ]));
        assert_eq!(span, expected_span);
    }
}
