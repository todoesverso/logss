use clap::Parser;
use std::collections::HashMap;
use std::error;
use std::sync::mpsc::TryRecvError;
use tui::backend::Backend;
use tui::layout::{Constraint, Layout, Rect};
use tui::terminal::Frame;
use tui::text::Spans;

use crate::args::Args;
use crate::container::{Container, CONTAINERS_MAX, CONTAINER_BUFFER, CONTAINER_COLORS};
use crate::help::render_help;
use crate::input::Input;
use crate::states::{AppState, Views};
use crate::tstdin::StdinHandler;

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

/// Application.
///
/// This is the main application.
#[derive(Debug)]
pub struct App<'a> {
    /// Is the application running?
    pub running: bool,
    pub containers: HashMap<String, Container<'a>>,
    pub state: AppState,
    pub input: Input,
    stdin: StdinHandler,
    raw_buffer: Container<'a>,
    args: Args,
}

impl<'a> Default for App<'a> {
    fn default() -> Self {
        Self {
            running: true,
            stdin: StdinHandler::new(),
            args: Args::parse(),
            input: Input::default(),
            raw_buffer: Container::new("*".to_string(), CONTAINER_BUFFER),
            containers: HashMap::new(),
            state: AppState::default(),
        }
    }
}

impl<'a> App<'a> {
    /// Constructs a new instance of [`App`].
    pub fn new(args: Option<Args>) -> Self {
        let mut ret = Self::default();
        if let Some(args_inner) = args {
            ret.args = args_inner;
        }

        // Let 0 for raw_buffer
        for (id, c) in (1_u8..).zip(ret.args.containers.iter()) {
            let mut con = Container::new(c.clone(), CONTAINER_BUFFER);
            con.state.color = CONTAINER_COLORS[(id - 1) as usize];
            con.id = id;
            ret.containers.insert(c.to_string(), con);
        }
        if ret.containers.is_empty() {
            ret.state.show = Views::RawBuffer;
        }
        ret
    }

    fn get_free_ids(&self) -> Vec<u8> {
        let used_ids: Vec<u8> = self.containers.iter().map(|c| c.1.id).collect();
        let mut free_ids: Vec<u8> = Vec::new();

        for id in 1_u8..CONTAINERS_MAX {
            if !used_ids.contains(&id) {
                free_ids.push(id);
            }
        }

        free_ids
    }

    pub fn add_input_as_container(&mut self) {
        self.add_container(&self.input.input.clone());
        self.input.reset();
    }
    pub fn add_container(&mut self, text: &str) {
        let first_free_id = self.get_free_ids();
        let mut con = Container::new(text.to_string(), CONTAINER_BUFFER);
        if let Some(inner_id) = first_free_id.first() {
            con.state.color = CONTAINER_COLORS[(inner_id - 1) as usize];
            con.id = *inner_id;
            self.containers.insert(text.to_string(), con);
        }
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&mut self) {
        self.get_stdin();
    }

    pub fn get_stdin(&mut self) {
        match self.stdin.try_recv() {
            Ok(line) => {
                // save all lines to a raw buffer
                if !self.state.paused {
                    self.raw_buffer.cb.push(Spans::from(line.clone()));
                }
                let keys = self.containers.keys().cloned().collect::<Vec<String>>();
                for key in keys {
                    if line.contains(&key.to_string()) && !self.state.paused {
                        self.containers
                            .get_mut(&key)
                            .unwrap()
                            .proc_and_push_line(line.clone());
                    }
                }
            }
            Err(TryRecvError::Disconnected) => {
                self.state.running = false;
            }
            _ => {}
        }
    }

    fn get_layout_blocks(&self, size: Rect) -> Vec<Rect> {
        let mut constr = vec![];
        for _ in 0..self.containers.len() {
            constr.push(Constraint::Ratio(1, self.containers.len() as u32));
        }
        let ret = Layout::default()
            .direction(self.state.direction.clone())
            .constraints(constr.as_ref())
            .split(size);

        ret
    }

    fn render_containers<B: Backend>(&mut self, frame: &mut Frame<'_, B>) {
        let blocks = self.get_layout_blocks(frame.size());
        // TODO: Review this logic
        for (i, (_, container)) in self.containers.iter().enumerate() {
            container.render(frame, blocks[i]);
        }
    }

    fn update_containers<B: Backend>(&mut self, frame: &mut Frame<'_, B>) {
        let blocks = self.get_layout_blocks(frame.size());
        let mut area;
        // General containers
        for (i, (_, container)) in self.containers.iter_mut().enumerate() {
            container.state.wrap = self.state.wrap;
            if self.state.show == Views::Zoom {
                area = frame.size().height;
            } else {
                area = blocks[i].height;
            }
            container.state.paused = self.state.paused;
            container.state.wrap = self.state.wrap;
            container.update_scroll(
                area as usize,
                &mut self.state.scroll_up,
                &mut self.state.scroll_down,
            );
        }
        // Raw buffer
        let mut container = &mut self.raw_buffer;
        container.state.paused = self.state.paused;

        container.update_scroll(
            frame.size().height as usize,
            &mut self.state.scroll_up,
            &mut self.state.scroll_down,
        );
    }

    fn render_raw<B: Backend>(&mut self, frame: &mut Frame<'_, B>) {
        let container = &self.raw_buffer;
        container.render(frame, frame.size());
    }

    fn render_id<B: Backend>(&mut self, frame: &mut Frame<'_, B>, id: u8) {
        for (_, container) in self.containers.iter() {
            if container.id == id {
                container.render(frame, frame.size());
            }
        }
    }

    fn remove_id(&mut self, id: u8) {
        if let Some(key) = self.containers.iter().find(|c| c.1.id == id).map(|c| c.0) {
            self.containers.remove(&key.clone());
        }
    }

    fn render_help<B: Backend>(&self, frame: &mut Frame<'_, B>) {
        if self.state.help {
            render_help(frame);
        }
    }
    fn render_input<B: Backend>(&self, frame: &mut Frame<'_, B>) {
        if self.state.show_input {
            self.input.render(frame, frame.size());
        }
    }

    /// Renders the user interface widgets.
    pub fn render<B: Backend>(&mut self, frame: &mut Frame<'_, B>) {
        self.update_containers(frame);
        match self.state.show {
            Views::Containers => {
                self.render_containers(frame);
            }
            Views::RawBuffer => {
                self.render_raw(frame);
            }
            Views::Zoom => {
                if let Some(id) = self.state.zoom_id {
                    self.render_id(frame, id);
                }
            }
            Views::Remove => {
                if let Some(id) = self.state.zoom_id {
                    self.remove_id(id);
                    if self.containers.is_empty() {
                        self.state.show = Views::RawBuffer;
                    } else {
                        self.state.show = Views::Containers;
                        self.state.zoom_id = None;
                    }
                }
            }
        }
        // Popups need to go at the bottom
        self.render_help(frame);
        self.render_input(frame);
    }
}
