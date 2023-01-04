use clap::Parser;
use std::collections::HashMap;
use std::error;
use std::sync::mpsc::TryRecvError;
use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::terminal::Frame;
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

use crate::args::Args;
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
    args: Args,
    raw_buffer: Container<'a>,
    contains: HashMap<String, Container<'a>>,
    pub show: Views,
    pub prev_show: Views,
    pub wrap: bool,
    pub help: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Views {
    RawBuffer,
    Containers,
}

// TODO: rename all this contains and move it to its own mod
#[derive(Debug)]
struct Container<'a> {
    /// matching text
    text: String,
    /// circular buffer with matching lines
    cb: CircularBuffer<Spans<'a>>,
    /// scroll for the paragraph
    scroll: u16,
}

impl<'a> Container<'a> {
    pub fn new(text: String, buffersize: usize) -> Self {
        Self {
            text,
            cb: CircularBuffer::new(buffersize),
            scroll: 0,
        }
    }

    fn process_line(&self, line: String) -> Spans<'a> {
        // TODO: maybe add smart time coloration?
        // TODO: accept color
        let contains_index = line.find(&self.text).unwrap();

        Spans(vec![
            Span::from(line[0..contains_index].to_string()),
            Span::styled(self.text.clone(), Style::default().fg(Color::Red)),
            Span::from(line[contains_index + self.text.len()..].to_string()),
        ])
    }

    fn push(&mut self, element: Spans<'a>) {
        let _ = &self.cb.push(element);
    }

    fn proc_and_push_line(&mut self, line: String) {
        // TODO: select colors randomly
        let sp = self.process_line(line);
        let _ = &self.push(sp);
    }

    fn update_scroll(&mut self, size: usize) {
        let bufflen = self.cb.len();

        // TODO: Review this logic
        if bufflen < size {
        } else {
            self.scroll = (bufflen - size) as u16;
        }
    }
}

impl<'a> Default for App<'a> {
    fn default() -> Self {
        Self {
            running: true,
            wrap: false,
            help: false,
            stdin: StdinHandler::new(),
            args: Args::parse(),
            raw_buffer: Container::new("*".to_string(), 128),
            contains: HashMap::new(),
            show: Views::Containers,
            prev_show: Views::Containers,
        }
    }
}

impl<'a> App<'a> {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        let mut ret = Self::default();

        for c in &ret.args.contains {
            // TODO: this should be a constant
            let con = Container::new(c.clone(), 128);
            ret.contains.insert(c.to_string(), con);
        }
        ret
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&mut self) {
        self.get_stdin();
    }

    pub fn get_stdin(&mut self) {
        match self.stdin.try_recv() {
            Ok(line) => {
                // save all lines to a raw buffer
                self.raw_buffer.cb.push(Spans::from(line.clone()));
                let keys = self.contains.keys().cloned().collect::<Vec<String>>();
                for key in keys {
                    if line.contains(&key.to_string()) {
                        self.contains
                            .get_mut(&key)
                            .unwrap()
                            .proc_and_push_line(line.clone());
                    }
                }
            }
            Err(TryRecvError::Disconnected) => {
                todo!();
            }
            _ => {}
        }
    }

    fn get_layout_blocks(&self, size: Rect) -> Vec<Rect> {
        let mut constr = vec![];
        let contains_count = if !self.contains.is_empty() {
            self.contains.len()
        } else {
            1
        };
        let mut per = 100 / contains_count;
        // TODO: fix this, it depends on the size, so use it
        if contains_count % 2 != 0 {
            per -= 1
        };
        for _ in 0..contains_count {
            constr.push(Constraint::Percentage(per as u16));
        }
        let ret = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constr.as_ref())
            .split(size);

        ret
    }

    fn render_contains<B: Backend>(&mut self, frame: &mut Frame<'_, B>) {
        let blocks = self.get_layout_blocks(frame.size());
        // TODO: Review this logic
        let keys = self.contains.keys().cloned().collect::<Vec<String>>();
        for (i, key) in keys.into_iter().enumerate() {
            //for (i, key) in self.contains.keys().cloned().enumerate() {
            let container = self.contains.get_mut(&key).unwrap();
            let cb = container.cb.ordered_clone();
            container.update_scroll(blocks[i].height as usize);
            let mut paragraph = Paragraph::new(cb.buffer.clone())
                .block(create_block(&key))
                .style(Style::default().fg(Color::White).bg(Color::Black))
                .scroll((container.scroll, 0));
            if self.wrap {
                paragraph = paragraph.wrap(Wrap { trim: false });
            }
            frame.render_widget(paragraph, blocks[i]);
        }
    }

    // TODO: avoid code duplication
    fn render_raw<B: Backend>(&self, frame: &mut Frame<'_, B>) {
        let container = &self.raw_buffer;
        let cb = container.cb.ordered_clone();
        let mut paragraph = Paragraph::new(cb.buffer.clone())
            .block(create_block("*"))
            .style(Style::default().fg(Color::White).bg(Color::Black));
        if self.wrap {
            paragraph = paragraph.wrap(Wrap { trim: false });
        }

        frame.render_widget(paragraph, frame.size());
    }

    fn render_help<B: Backend>(&self, frame: &mut Frame<'_, B>) {
        let size = frame.size();
        if self.help {
            let help_text = vec![
                Spans::from(Span::styled(
                    "h   - toggles help popup",
                    Style::default().bg(Color::Blue),
                )),
                Spans::from(Span::styled(
                    "w   - toggles text wrapping",
                    Style::default().bg(Color::Blue),
                )),
                Spans::from(Span::styled(
                    "*   - toggles between containers and raw input",
                    Style::default().bg(Color::Blue),
                )),
                Spans::from(Span::styled(
                    "Esc - exits the program",
                    Style::default().bg(Color::Red),
                )),
            ];
            let block = Block::default().title("Help").borders(Borders::ALL);
            let paragraph = Paragraph::new(help_text.clone()).block(block);
            let area = centered_rect(60, 20, size);
            frame.render_widget(Clear, area); //this clears out the background
            frame.render_widget(paragraph, area);
        }
    }
    /// Renders the user interface widgets.
    pub fn render<B: Backend>(&mut self, frame: &mut Frame<'_, B>) {
        match self.show {
            Views::Containers => {
                self.render_contains(frame);
            }
            Views::RawBuffer => {
                self.render_raw(frame);
            }
        }
        // Popups need to go at the bottom
        self.render_help(frame);
    }
}

fn create_block(title: &str) -> Block {
    Block::default().borders(Borders::ALL).title(Span::styled(
        title,
        Style::default().add_modifier(Modifier::BOLD),
    ))
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
