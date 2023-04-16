use crossterm::event::KeyCode;
use ratatui::backend::Backend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::terminal::Frame;
use ratatui::text::Spans;
use std::error;
use std::sync::mpsc::TryRecvError;

use crate::args::{parse_args, Args};
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
    pub containers: Vec<Container<'a>>,
    pub state: AppState,
    pub input: Input,
    stdin: StdinHandler,
    pub raw_buffer: Container<'a>,
    args: Args,
}

impl<'a> Default for App<'a> {
    fn default() -> Self {
        Self {
            stdin: StdinHandler::new(),
            args: parse_args(),
            input: Input::default(),
            raw_buffer: Container::new(".*".to_string(), CONTAINER_BUFFER),
            containers: Vec::new(),
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
            if ret.args.vertical.is_some() {
                ret.state.direction = Direction::Horizontal;
            }
        }

        // Let 0 for raw_buffer
        for (id, c) in (1_u8..).zip(ret.args.containers.iter()) {
            let mut con = Container::new(c.clone(), CONTAINER_BUFFER);
            con.state.color = CONTAINER_COLORS[(id - 1) as usize];
            con.id = id;
            ret.containers.push(con);
        }
        if ret.containers.is_empty() {
            ret.state.show = Views::RawBuffer;
        }
        ret
    }

    pub fn init(&mut self) -> AppResult<()> {
        self.state.running = true;
        self.stdin.init(self.args.command.clone())?;
        Ok(())
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
                self.state.show = Views::Containers;
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
        self.add_container(&self.input.inner_clone());
        self.input.reset();
    }

    pub fn add_container(&mut self, text: &str) {
        let first_free_id = self.get_free_ids();
        let mut con = Container::new(text.to_string(), CONTAINER_BUFFER);
        if let Some(inner_id) = first_free_id.first() {
            con.state.color = CONTAINER_COLORS[(inner_id - 1) as usize];
            con.id = *inner_id;
            self.containers.push(con);
        }
    }

    pub fn zoom_into(&mut self, id: u8) {
        if !self.containers.iter().map(|c| c.id).any(|x| x == id) {
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
        if !self.containers.iter().map(|c| c.id).any(|x| x == id) {
            return;
        }
        self.state.show = Views::Remove;
        self.state.zoom_id = Some(id);
    }

    pub fn hide_view(&mut self, id: u8) {
        if !self.containers.iter().map(|c| c.id).any(|x| x == id) {
            return;
        }
        for container in self.containers.iter_mut() {
            if container.id == id {
                container.state.hide = !container.state.hide;
            }
        }
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
                for c in self.containers.iter_mut() {
                    if c.re.is_match(&line) && !self.state.paused {
                        c.proc_and_push_line(line.clone());
                    }
                }
            }
            Err(TryRecvError::Disconnected) => {
                self.stop();
            }
            Err(TryRecvError::Empty) if self.args.exit.unwrap_or_default() => {
                self.stop();
            }
            _ => {}
        }
    }

    fn get_free_ids(&self) -> Vec<u8> {
        let used_ids: Vec<u8> = self.containers.iter().map(|c| c.id).collect();
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
        let show_cont = self.containers.iter().filter(|c| !c.state.hide).count();
        for _ in 0..show_cont {
            constr.push(Constraint::Ratio(1, show_cont as u32));
        }
        let ret = Layout::default()
            .direction(self.state.direction.clone())
            .constraints(constr.as_ref())
            .split(size);

        ret.to_vec()
    }

    fn render_containers<B: Backend>(&mut self, frame: &mut Frame<'_, B>) {
        let blocks = self.get_layout_blocks(frame.size());

        for (i, container) in self.containers.iter().filter(|c| !c.state.hide).enumerate() {
            container.render(frame, blocks[i]);
        }
    }

    fn update_containers(&mut self, frame_rect: Rect) {
        let blocks = self.get_layout_blocks(frame_rect);
        let mut area;

        // General containers
        for (i, container) in self
            .containers
            .iter_mut()
            .filter(|c| !c.state.hide)
            .enumerate()
        {
            container.state.wrap = self.state.wrap;
            if self.state.show == Views::Zoom {
                area = frame_rect.height;
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
            frame_rect.height as usize,
            &mut self.state.scroll_up,
            &mut self.state.scroll_down,
        );
    }

    fn render_raw<B: Backend>(&mut self, frame: &mut Frame<'_, B>) {
        let container = &self.raw_buffer;
        container.render(frame, frame.size());
    }

    fn render_id<B: Backend>(&mut self, frame: &mut Frame<'_, B>, id: u8) {
        for container in self.containers.iter() {
            if container.id == id {
                container.render(frame, frame.size());
            }
        }
    }

    fn remove_id(&mut self, id: u8) {
        if let Some(index) = self.containers.iter().position(|c| c.id == id) {
            self.containers.swap_remove(index);
        }
        self.containers.sort_by_key(|container| container.id);
    }

    fn render_help<B: Backend>(&self, frame: &mut Frame<'_, B>) {
        if self.state.help {
            render_help(frame);
        }
    }

    fn render_input<B: Backend>(&self, frame: &mut Frame<'_, B>) {
        if self.state.show_input {
            self.input.render(frame);
        }
    }

    /// Renders the user interface widgets.
    pub fn render<B: Backend>(&mut self, frame: &mut Frame<'_, B>) {
        self.update_containers(frame.size());
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
                        self.render_raw(frame);
                    } else {
                        self.state.show = Views::Containers;
                        self.state.zoom_id = None;
                        self.render_containers(frame);
                    }
                }
            }
        }
        // Popups need to go at the bottom
        self.render_help(frame);
        self.render_input(frame);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{
        backend::TestBackend, buffer::Buffer, style::Color, style::Modifier, style::Style, Terminal,
    };

    #[test]
    fn test_new() {
        let mut app = App::new(None);

        // Running
        assert_eq!(app.is_running(), false);
        app.init().unwrap();
        assert_eq!(app.is_running(), true);
        app.stop();
        assert_eq!(app.is_running(), false);

        // Direction
        assert_eq!(app.state.direction, Direction::Vertical);
        app.flip_direction();
        assert_eq!(app.state.direction, Direction::Horizontal);

        // Containers
        assert_eq!(app.containers.len(), 0);
        app.add_container("text");
        assert_eq!(app.containers.len(), 1);
        app.add_container("text2");
        assert_eq!(app.containers.len(), 2);

        let mut args = parse_args();
        args.containers = vec!["a".to_string(), "b".to_string()];
        let app = App::new(Some(args));
        assert_eq!(app.containers.len(), 2);
    }

    #[test]
    fn input() {
        // New all clean
        let mut app = App::new(None);
        assert_eq!(app.containers.len(), 0);
        assert_eq!(app.input.input, "".to_string());
        // Add a char
        app.update_input(KeyCode::Char('a'));
        assert_eq!(app.input.input, "a".to_string());
        // Remove the char
        app.update_input(KeyCode::Backspace);
        assert_eq!(app.input.input, "".to_string());
        // Re add the char
        app.update_input(KeyCode::Char('a'));
        assert_eq!(app.containers.len(), 0);
        app.update_input(KeyCode::Enter);
        // Enter the input
        assert_eq!(app.show_input(), false);
        assert_eq!(app.input.input, "".to_string());
        assert_eq!(app.containers.len(), 1);
    }

    #[test]
    fn zoom_into() {
        let mut app = App::new(None);
        assert_eq!(app.containers.len(), 0);
        app.add_container("text");
        app.add_container("text2");
        app.add_container("text3");
        assert_eq!(app.containers.len(), 3);
        assert_eq!(app.state.show, Views::RawBuffer);
        assert_eq!(app.state.zoom_id, None);

        // Zoom in
        app.zoom_into(1);
        assert_eq!(app.state.show, Views::Zoom);
        assert_eq!(app.state.zoom_id, Some(1));

        // Zoom out
        app.zoom_into(1);
        assert_eq!(app.state.zoom_id, None);
        assert_eq!(app.state.show, Views::Containers);
    }

    #[test]
    fn get_stdin() {
        let mut app = App::new(None);
        app.add_container("a");
        let c = app.containers.get(0).unwrap();
        assert_eq!(c.cb.is_empty(), true);
        assert_eq!(app.raw_buffer.cb.is_empty(), true);
        assert_eq!(app.raw_buffer.cb.len(), 0);
        app.init().unwrap();
        app.stdin.sender.send("abc".to_string()).unwrap();
        app.tick();

        app.stdin.sender.send("def".to_string()).unwrap();
        app.tick();

        let c = app.containers.get(0).unwrap();
        assert_eq!(c.cb.is_empty(), false);
        assert_eq!(c.cb.len(), 1);
        assert_eq!(app.raw_buffer.cb.is_empty(), false);
        assert_eq!(app.raw_buffer.cb.len(), 2);
    }

    #[test]
    fn get_layout_blocks() {
        let mut app = App::new(None);
        app.add_container("a");
        let rect = Rect::new(0, 0, 10, 10);
        let lb = app.get_layout_blocks(rect);
        assert_eq!(lb, vec![rect]);
        app.add_container("b");
        let lb = app.get_layout_blocks(rect);
        let expected_blocks = vec![Rect::new(0, 0, 10, 5), Rect::new(0, 5, 10, 5)];
        assert_eq!(lb, expected_blocks);
    }

    #[test]
    fn render_containers() {
        let mut app = App::new(None);
        app.add_container("a");
        app.add_container("b");
        for c in app.containers.iter_mut() {
            c.state.color = Color::White;
        }
        let backend = TestBackend::new(14, 14);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                app.render_containers(f);
            })
            .unwrap();
        let mut expected = Buffer::with_lines(vec![
            "┌(1) - a─────┐",
            "│            │",
            "│            │",
            "│            │",
            "│            │",
            "│            │",
            "└────────────┘",
            "┌(2) - b─────┐",
            "│            │",
            "│            │",
            "│            │",
            "│            │",
            "│            │",
            "└────────────┘",
        ]);
        let bolds = [1, 2, 3, 4, 5, 6, 7];
        for x in 0..=13 {
            for y in 0..=13 {
                if bolds.contains(&x) && (y == 0 || y == 7) {
                    expected
                        .get_mut(x, y)
                        .set_style(Style::default().add_modifier(Modifier::BOLD));
                }
                expected.get_mut(x, y).set_fg(Color::White);
                expected.get_mut(x, y).set_bg(Color::Black);
            }
        }
        terminal.backend().assert_buffer(&expected);
    }

    #[test]
    fn render_id() {
        let mut app = App::new(None);
        app.add_container("a");
        app.add_container("b");
        for c in app.containers.iter_mut() {
            c.state.color = Color::White;
        }
        app.zoom_into(1);
        let backend = TestBackend::new(14, 14);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                app.render(f);
            })
            .unwrap();
        let mut expected = Buffer::with_lines(vec![
            "┌(1) - a─────┐",
            "│            │",
            "│            │",
            "│            │",
            "│            │",
            "│            │",
            "│            │",
            "│            │",
            "│            │",
            "│            │",
            "│            │",
            "│            │",
            "│            │",
            "└────────────┘",
        ]);
        let bolds = [1, 2, 3, 4, 5, 6, 7];
        for x in 0..=13 {
            for y in 0..=13 {
                if bolds.contains(&x) && (y == 0) {
                    expected
                        .get_mut(x, y)
                        .set_style(Style::default().add_modifier(Modifier::BOLD));
                }
                expected.get_mut(x, y).set_fg(Color::White);
                expected.get_mut(x, y).set_bg(Color::Black);
            }
        }
        terminal.backend().assert_buffer(&expected);
        app.zoom_into(1);
        app.zoom_into(2);
        terminal
            .draw(|f| {
                app.render(f);
            })
            .unwrap();
        let mut expected = Buffer::with_lines(vec![
            "┌(2) - b─────┐",
            "│            │",
            "│            │",
            "│            │",
            "│            │",
            "│            │",
            "│            │",
            "│            │",
            "│            │",
            "│            │",
            "│            │",
            "│            │",
            "│            │",
            "└────────────┘",
        ]);
        let bolds = [1, 2, 3, 4, 5, 6, 7];
        for x in 0..=13 {
            for y in 0..=13 {
                if bolds.contains(&x) && (y == 0) {
                    expected
                        .get_mut(x, y)
                        .set_style(Style::default().add_modifier(Modifier::BOLD));
                }
                expected.get_mut(x, y).set_fg(Color::White);
                expected.get_mut(x, y).set_bg(Color::Black);
            }
        }
        terminal.backend().assert_buffer(&expected);
    }

    #[test]
    fn remove_view() {
        let mut app = App::new(None);
        app.add_container("a");
        app.add_container("b");
        for c in app.containers.iter_mut() {
            c.state.color = Color::White;
        }
        app.flip_raw_view();
        app.remove_view(1);
        let backend = TestBackend::new(14, 14);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                app.render(f);
            })
            .unwrap();
        let mut expected = Buffer::with_lines(vec![
            "┌(2) - b─────┐",
            "│            │",
            "│            │",
            "│            │",
            "│            │",
            "│            │",
            "│            │",
            "│            │",
            "│            │",
            "│            │",
            "│            │",
            "│            │",
            "│            │",
            "└────────────┘",
        ]);
        let bolds = [1, 2, 3, 4, 5, 6, 7];
        for x in 0..=13 {
            for y in 0..=13 {
                if bolds.contains(&x) && (y == 0) {
                    expected
                        .get_mut(x, y)
                        .set_style(Style::default().add_modifier(Modifier::BOLD));
                }
                expected.get_mut(x, y).set_fg(Color::White);
                expected.get_mut(x, y).set_bg(Color::Black);
            }
        }
        terminal.backend().assert_buffer(&expected);
    }

    #[test]
    fn hide_view() {
        let mut app = App::new(None);
        app.add_container("a");
        app.add_container("b");
        for c in app.containers.iter_mut() {
            c.state.color = Color::White;
        }
        app.flip_raw_view();
        app.hide_view(1);
        let backend = TestBackend::new(14, 14);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                app.render(f);
            })
            .unwrap();
        let mut expected = Buffer::with_lines(vec![
            "┌(2) - b─────┐",
            "│            │",
            "│            │",
            "│            │",
            "│            │",
            "│            │",
            "│            │",
            "│            │",
            "│            │",
            "│            │",
            "│            │",
            "│            │",
            "│            │",
            "└────────────┘",
        ]);
        let bolds = [1, 2, 3, 4, 5, 6, 7];
        for x in 0..=13 {
            for y in 0..=13 {
                if bolds.contains(&x) && (y == 0) {
                    expected
                        .get_mut(x, y)
                        .set_style(Style::default().add_modifier(Modifier::BOLD));
                }
                expected.get_mut(x, y).set_fg(Color::White);
                expected.get_mut(x, y).set_bg(Color::Black);
            }
        }
        terminal.backend().assert_buffer(&expected);

        app.hide_view(1);
        terminal
            .draw(|f| {
                app.render(f);
            })
            .unwrap();
        let mut expected = Buffer::with_lines(vec![
            "┌(1) - a─────┐",
            "│            │",
            "│            │",
            "│            │",
            "│            │",
            "│            │",
            "└────────────┘",
            "┌(2) - b─────┐",
            "│            │",
            "│            │",
            "│            │",
            "│            │",
            "│            │",
            "└────────────┘",
        ]);
        let bolds = [1, 2, 3, 4, 5, 6, 7];
        for x in 0..=13 {
            for y in 0..=13 {
                if bolds.contains(&x) && (y == 0 || y == 7) {
                    expected
                        .get_mut(x, y)
                        .set_style(Style::default().add_modifier(Modifier::BOLD));
                }
                expected.get_mut(x, y).set_fg(Color::White);
                expected.get_mut(x, y).set_bg(Color::Black);
            }
        }
        terminal.backend().assert_buffer(&expected);
    }

    #[test]
    fn update_containers() {
        let mut app = App::new(None);
        let rect = Rect::new(0, 0, 10, 10);
        app.add_container("a");
        app.add_container("b");

        for c in app.containers.iter() {
            assert_eq!(c.state.paused, false);
            assert_eq!(c.state.wrap, false);
            assert_eq!(c.state.scroll, 0);
        }

        app.init().unwrap();
        for _ in 0..=128 {
            app.stdin.sender.send("abc".to_string()).unwrap();
            app.tick();
        }

        // Change the app state
        app.state.paused = true;
        app.state.wrap = true;
        app.state.scroll_up = 0;
        app.state.scroll_down = 10;
        app.update_containers(rect);

        for c in app.containers.iter() {
            assert_eq!(c.state.paused, app.state.paused);
            assert_eq!(c.state.wrap, app.state.wrap);
            assert_eq!(c.state.scroll, 49);
        }
        app.state.scroll_up = 5;
        app.update_containers(rect);

        for c in app.containers.iter() {
            assert_eq!(c.state.scroll, 54);
        }
    }
}
