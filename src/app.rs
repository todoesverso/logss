use clap::Parser;
use std::collections::HashMap;
use std::error;
use std::sync::mpsc::TryRecvError;
use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::Color;
use tui::terminal::Frame;
use tui::text::Spans;

use crate::args::Args;
use crate::container::{Container, CONTAINER_BUFFER};
use crate::help::render_help;
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
    pub containers: HashMap<String, Container<'a>>,
    pub show: Views,
    pub prev_show: Views,
    pub wrap: bool,
    pub pause: bool,
    pub help: bool,
    pub zoom_id: Option<u8>,
    pub up: u16,
    pub up_hot: u16,
    pub down: u16,
    pub down_hot: u16,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Views {
    RawBuffer,
    Containers,
    Zoom,
    Remove,
}

impl<'a> Default for App<'a> {
    fn default() -> Self {
        Self {
            running: true,
            wrap: false,
            pause: false,
            help: false,
            stdin: StdinHandler::new(),
            args: Args::parse(),
            raw_buffer: Container::new("*".to_string(), CONTAINER_BUFFER),
            containers: HashMap::new(),
            show: Views::Containers,
            prev_show: Views::Containers,
            zoom_id: None,
            up: 0,
            up_hot: 0,
            down: 0,
            down_hot: 0,
        }
    }
}

impl<'a> App<'a> {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        let mut ret = Self::default();
        let colors = [
            Color::Red,
            Color::LightRed,
            Color::Blue,
            Color::LightBlue,
            Color::Cyan,
            Color::LightCyan,
            Color::Green,
            Color::LightGreen,
            Color::Yellow,
            Color::LightYellow,
            Color::Magenta,
            Color::LightMagenta,
            Color::Gray,
            Color::DarkGray,
        ];

        let mut i = 0;
        for (id, c) in (0_u8..).zip(ret.args.containers.iter()) {
            if i > ret.args.containers.len() {
                i = 0;
            }

            let mut con = Container::new(c.clone(), CONTAINER_BUFFER);
            con.match_color = colors[i];
            con.id = id;
            ret.containers.insert(c.to_string(), con);
            i += 1;
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
                if !self.pause {
                    self.raw_buffer.cb.push(Spans::from(line.clone()));
                }
                let keys = self.containers.keys().cloned().collect::<Vec<String>>();
                for key in keys {
                    if line.contains(&key.to_string()) && !self.pause {
                        self.containers
                            .get_mut(&key)
                            .unwrap()
                            .proc_and_push_line(line.clone());
                    }
                }
            }
            Err(TryRecvError::Disconnected) => {
                self.running = false;
            }
            _ => {}
        }
    }

    fn get_layout_blocks(&self, size: Rect) -> Vec<Rect> {
        let mut constr = vec![];
        let containers_count = if !self.containers.is_empty() {
            self.containers.len()
        } else {
            1
        };
        let mut per = 100 / containers_count;
        // TODO: fix this, it depends on the size, so use it
        if containers_count % 2 != 0 {
            per -= 1
        };
        for _ in 0..containers_count {
            constr.push(Constraint::Percentage(per as u16));
        }
        let ret = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constr.as_ref())
            .split(size);

        ret
    }

    fn render_containers<B: Backend>(&mut self, frame: &mut Frame<'_, B>) {
        let blocks = self.get_layout_blocks(frame.size());
        // TODO: Review this logic
        for (i, (key, container)) in self.containers.iter().enumerate() {
            let title = format!("({}) - {}", container.id, key);
            container.render(frame, blocks[i], &title, self.pause);
        }
    }

    fn update_containers<B: Backend>(&mut self, frame: &mut Frame<'_, B>) {
        let blocks = self.get_layout_blocks(frame.size());
        let mut area;
        for (i, (_, container)) in self.containers.iter_mut().enumerate() {
            container.wrapui = self.wrap;
            if self.show == Views::Zoom {
                area = frame.size().height;
            } else {
                area = blocks[i].height;
            }
            container.update_scroll(area as usize, &mut self.up, &mut self.down);
        }
        let container = &mut self.raw_buffer;

        container.update_scroll(frame.size().height as usize, &mut self.up, &mut self.down);
    }

    fn render_raw<B: Backend>(&mut self, frame: &mut Frame<'_, B>) {
        let container = &self.raw_buffer;
        container.render(frame, frame.size(), "*", self.pause);
    }

    fn render_id<B: Backend>(&mut self, frame: &mut Frame<'_, B>, id: u8) {
        for (key, container) in self.containers.iter() {
            if container.id == id {
                let title = format!("({id}) - {key}");
                container.render(frame, frame.size(), &title, self.pause);
            }
        }
    }

    fn remove_id(&mut self, id: u8) {
        if let Some(key) = self.containers.iter().find(|c| c.1.id == id).map(|c| c.0) {
            let key = key.clone();
            self.containers.remove(&key);
        }
    }

    fn render_help<B: Backend>(&self, frame: &mut Frame<'_, B>) {
        if self.help {
            render_help(frame);
        }
    }
    /// Renders the user interface widgets.
    pub fn render<B: Backend>(&mut self, frame: &mut Frame<'_, B>) {
        self.update_containers(frame);
        match self.show {
            Views::Containers => {
                self.render_containers(frame);
            }
            Views::RawBuffer => {
                self.render_raw(frame);
            }
            Views::Zoom => {
                if let Some(id) = self.zoom_id {
                    self.render_id(frame, id);
                }
            }
            Views::Remove => {
                if let Some(id) = self.zoom_id {
                    self.remove_id(id);
                    self.show = Views::Containers;
                    self.zoom_id = None;
                }
            }
        }
        // Popups need to go at the bottom
        self.render_help(frame);
    }
}
