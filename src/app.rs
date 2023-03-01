use clap::Parser;
use crossterm::event::KeyCode;
use std::collections::HashMap;
use std::error;
use std::sync::mpsc::TryRecvError;
use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout, Rect};
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

    pub fn init(&mut self) {
        self.state.running = true;
        self.stdin.init();
    }

    pub fn is_running(&self) -> bool {
        self.state.running
    }

    pub fn show_input(&self) -> bool {
        self.state.show_input
    }

    pub fn hide_show_input(&mut self) {
        self.state.show_input = false;
    }

    pub fn stop(&mut self) {
        self.state.running = false;
    }

    pub fn pause(&mut self) {
        self.state.paused = true;
    }

    pub fn unpause(&mut self) {
        self.state.paused = false;
    }

    pub fn flip_pause(&mut self) {
        self.state.paused = !self.state.paused;
    }

    pub fn flip_wrap(&mut self) {
        self.state.wrap = !self.state.wrap;
    }

    pub fn flip_help(&mut self) {
        self.state.help = !self.state.help;
    }

    pub fn flip_show_input(&mut self) {
        self.state.show_input = !self.state.show_input;
    }

    pub fn scroll_up(&mut self) {
        self.state.scroll_up += 1;
    }

    pub fn scroll_down(&mut self) {
        self.state.scroll_down += 1;
    }

    pub fn reset_scroll_up(&mut self) {
        self.state.scroll_up = 0;
    }

    pub fn reset_scroll_down(&mut self) {
        self.state.scroll_down = 0;
    }

    pub fn set_direction(&mut self, direction: Direction) {
        self.state.direction = direction;
    }

    pub fn flip_direction(&mut self) {
        if self.state.direction == Direction::Vertical {
            self.state.direction = Direction::Horizontal;
        } else {
            self.state.direction = Direction::Vertical;
        }
    }

    pub fn update_input(&mut self, key_code: KeyCode) {
        match key_code {
            KeyCode::Enter => {
                self.add_input_as_container();
                self.hide_show_input();
            }
            KeyCode::Char(c) => {
                self.input.push(c);
            }
            KeyCode::Backspace => {
                self.input.pop();
            }
            KeyCode::Esc => {
                self.hide_show_input();
            }
            _ => {}
        }
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

    pub fn zoom_into(&mut self, id: u8) {
        if !self.containers.values().map(|c| c.id).any(|x| x == id) {
            return;
        }

        if self.state.show == Views::Zoom {
            self.state.show = Views::Containers;
            self.state.zoom_id = None;
        } else {
            self.state.show = Views::Zoom;
            self.state.zoom_id = Some(id);
        }
    }

    pub fn remove_view(&mut self, id: u8) {
        if !self.containers.values().map(|c| c.id).any(|x| x == id) {
            return;
        }
        self.state.show = Views::Remove;
        self.state.zoom_id = Some(id);
    }

    pub fn flip_raw_view(&mut self) {
        if !self.containers.is_empty() {
            if self.state.show == Views::RawBuffer {
                self.state.show = Views::Containers;
            } else {
                self.state.show = Views::RawBuffer;
            }
        }
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&mut self) {
        self.get_stdin();
    }

    fn get_stdin(&mut self) {
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
                self.stop();
            }
            _ => {}
        }
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
