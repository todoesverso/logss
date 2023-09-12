use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use ratatui::backend::Backend;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::terminal::Frame;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use regex::Regex;
use slug;

use crate::app::AppResult;
use crate::cb::CircularBuffer;
use crate::states::ContainerState;

#[derive(Debug)]
pub struct Container<'a> {
    /// matching text
    text: String,
    pub re: Regex,
    /// circular buffer with matching lines
    pub cb: CircularBuffer<Line<'a>>,
    pub id: u8,
    pub state: ContainerState,
    pub file: Option<File>,
    count: u64,
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
            text: text.clone(),
            re: Regex::new(&text).unwrap(),
            cb: CircularBuffer::new(buffersize),
            id: 0,
            state: ContainerState::default(),
            file: None,
            count: 0,
        }
    }

    pub fn set_output_path(&mut self, output_path: PathBuf) -> AppResult<()> {
        let file_name = format!(
            "{}/{}.txt",
            output_path.to_string_lossy(),
            slug::slugify(self.text.clone())
        );
        let file_path = Path::new(&file_name);
        self.file = Some(
            OpenOptions::new()
                .append(true)
                .create(true)
                .open(file_path)?,
        );

        Ok(())
    }

    fn process_line(&self, line: &str) -> Option<Line<'a>> {
        // TODO: maybe add smart time coloration?
        if let Some(mat) = self.re.find(line) {
            let start = mat.start();
            let end = mat.end();

            return Some(Line::from(vec![
                Span::from(line[0..start].to_string()),
                Span::styled(
                    line[start..end].to_string(),
                    Style::default().fg(self.state.color),
                ),
                Span::from(line[end..].to_string()),
            ]));
        }
        None
    }

    pub fn push(&mut self, element: Line<'a>) {
        self.count += 1;
        let _ = &self.cb.push(element);
    }

    pub fn proc_and_push_line(&mut self, line: &str) -> Option<Line<'a>> {
        let rt_lines = self.process_line(line);
        let ret = rt_lines.clone();
        if let Some(rt_lines_in) = rt_lines {
            let _ = &self.push(rt_lines_in);
        }
        if let Some(file) = &mut self.file {
            file.write_all(line.as_bytes())
                .expect("Failed to write file");
            file.flush().expect("Failed to flush");
        }
        ret
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
        if self.state.hide {
            return;
        }
        let title = format!("({}) - '{}' [{}]", self.id, self.text, self.count);
        let mut paragraph = Paragraph::new(self.cb.ordered_clone().buffer.clone())
            .block(create_block(&title, self.state.color, self.state.paused))
            .style(self.state.style)
            .scroll((self.state.scroll, 0));
        if self.state.wrap {
            paragraph = paragraph.wrap(Wrap { trim: false });
        }

        frame.render_widget(paragraph, area);
    }

    pub fn reset(&mut self) {
        self.cb.reset();
    }
}

fn create_block(title: &str, color: Color, paused: bool) -> Block {
    let modifier = if paused {
        Modifier::BOLD | Modifier::SLOW_BLINK
    } else {
        Modifier::BOLD
    };
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
    fn test_set_output_path() {
        let _ = std::fs::remove_dir_all("test-sarasa");
        let mut container = Container::new("key".to_string(), 2);
        let path = std::path::PathBuf::from("test-sarasa");
        let mut dir = std::fs::DirBuilder::new();
        dir.recursive(true).create("test-sarasa").unwrap();
        assert!(container.set_output_path(path).is_ok());
        let _ = std::fs::remove_dir_all("test-sarasa");
    }

    #[test]
    fn process_line() {
        let container = Container::new("stringtomatch".to_string(), 2);
        let span = container.process_line("this line should not be proc");
        assert_eq!(span, None);
        let span = container.process_line("stringtomatch this line should be proc");
        let expected_span = Some(Line::from(vec![
            Span::from("".to_string()),
            Span::styled("stringtomatch".to_string(), Style::default().fg(Color::Red)),
            Span::from(" this line should be proc".to_string()),
        ]));
        assert_eq!(span, expected_span);
    }
}
