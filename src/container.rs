use tui::backend::Backend;
use tui::layout::Rect;
use tui::style::{Color, Modifier, Style};
use tui::terminal::Frame;
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::cb::CircularBuffer;

#[derive(Debug)]
pub struct Container<'a> {
    /// matching text
    text: String,
    /// circular buffer with matching lines
    pub cb: CircularBuffer<Spans<'a>>,
    /// scroll for the paragraph
    pub scroll: u16,
    pub id: u8,
    /// style for the paragraph
    styleui: Style,
    pub wrapui: bool,
    pub match_color: Color,
}

pub const CONTAINER_BUFFER: usize = 64;

impl<'a> Container<'a> {
    pub fn new(text: String, buffersize: usize) -> Self {
        Self {
            text,
            cb: CircularBuffer::new(buffersize),
            scroll: 0,
            styleui: Style::default().fg(Color::White).bg(Color::Black),
            wrapui: false,
            match_color: Color::Red,
            id: 0,
        }
    }

    fn process_line(&self, line: String) -> Spans<'a> {
        // TODO: maybe add smart time coloration?
        let contains_index = line.find(&self.text).unwrap();

        Spans(vec![
            Span::from(line[0..contains_index].to_string()),
            Span::styled(self.text.clone(), Style::default().fg(self.match_color)),
            Span::from(line[contains_index + self.text.len()..].to_string()),
        ])
    }

    pub fn push(&mut self, element: Spans<'a>) {
        let _ = &self.cb.push(element);
    }

    pub fn proc_and_push_line(&mut self, line: String) {
        let sp = self.process_line(line);
        let _ = &self.push(sp);
    }

    pub fn update_scroll(&mut self, size: usize, up: &mut u16, down: &mut u16) {
        // TODO: rewrite this mess
        let bufflen = self.cb.len();

        if bufflen < size {
        } else {
            self.scroll = (bufflen - size) as u16;
            if self.scroll - *down > 0 {
                self.scroll -= *down;
            } else if *down > 1 {
                *down -= 1;
            }

            if self.scroll + *up <= (bufflen - size) as u16 {
                self.scroll += *up;
            } else if *up > 1 {
                *up -= 1;
            }
        }
    }

    pub fn render<B: Backend>(
        &self,
        frame: &mut Frame<'_, B>,
        area: Rect,
        title: &'a str,
        paused: bool,
    ) {
        let mut paragraph = Paragraph::new(self.cb.ordered_clone().buffer.clone())
            .block(create_block(title, self.match_color, paused))
            .style(self.styleui)
            .scroll((self.scroll, 0));
        if self.wrapui {
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
