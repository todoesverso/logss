use std::{error, sync::mpsc::TryRecvError};

use crossterm::event::KeyCode;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    terminal::Frame,
    text::Line,
};

use crate::{
    args::{parse_args, Args},
    bars::render_bar_chart,
    container::{Container, CONTAINERS_MAX, CONTAINER_BUFFER, CONTAINER_COLORS},
    help::render_help,
    input::Input,
    states::{AppState, ScrollDirection, Views},
    tstdin::StdinHandler,
};
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
    pub single_buffer: Container<'a>,
    args: Args,
}

impl<'a> Default for App<'a> {
    fn default() -> Self {
        Self {
            stdin: StdinHandler::new(),
            args: parse_args(),
            input: Input::default(),
            raw_buffer: Container::new(".*".to_string(), CONTAINER_BUFFER),
            single_buffer: Container::new("single".to_string(), CONTAINER_BUFFER),
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
            if ret.args.single.is_some() {
                ret.state.show = Views::SingleBuffer;
            }
        }

        // Let 0 for raw_buffer
        for (id, c) in (1_u8..).zip(ret.args.containers.iter()) {
            let mut con = Container::new(c.clone(), CONTAINER_BUFFER);
            if let Some(output_path) = ret.args.output.clone() {
                con.set_output_path(output_path).ok();
            }
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

    pub const fn is_running(&self) -> bool {
        self.state.running
    }

    pub const fn show_input(&self) -> bool {
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

    pub fn flip_barchart(&mut self) {
        self.state.barchart = !self.state.barchart;
    }

    pub fn flip_show_input(&mut self) {
        self.state.show_input = !self.state.show_input;
    }

    pub fn scroll_up(&mut self) {
        self.pause();
        self.state.scroll_direction = ScrollDirection::UP;
    }

    pub fn scroll_down(&mut self) {
        self.pause();
        self.state.scroll_direction = ScrollDirection::DOWN;
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
                if self.add_input_as_container() {
                    self.hide_show_input();
                    self.state.show = Views::Containers;
                }
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

    pub fn add_input_as_container(&mut self) -> bool {
        let is_valid = self.input.is_valid();
        if is_valid {
            self.add_container(&self.input.inner_clone());
            self.input.reset();
        }
        is_valid
    }

    pub fn add_container(&mut self, text: &str) {
        let first_free_id = self.get_free_ids();
        let mut con = Container::new(text.to_string(), CONTAINER_BUFFER);
        if let Some(output_path) = self.args.output.clone() {
            con.set_output_path(output_path).ok();
        }
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

    pub fn flip_single_view(&mut self) {
        if !self.containers.is_empty() {
            if self.state.show == Views::SingleBuffer {
                self.state.show = Views::Containers;
            } else {
                self.state.show = Views::SingleBuffer;
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
                    self.raw_buffer.cb.push(Line::from(line.clone()));
                    for c in self.containers.iter_mut() {
                        if c.re.is_match(&line) {
                            let ret = c.proc_and_push_line(&line);
                            if let Some(l) = ret {
                                self.single_buffer.cb.push(l.to_owned());
                            }
                        }
                    }
                }
            }
            Err(TryRecvError::Disconnected) => self.stop(),
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
        let mut constr: Vec<Constraint> = vec![];
        let show_cont = self.containers.iter().filter(|c| !c.state.hide).count();
        for _ in 0..show_cont {
            constr.push(Constraint::Ratio(1, show_cont as u32));
        }
        let ret = Layout::default()
            .direction(self.state.direction)
            .constraints(constr)
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
        match self.state.show {
            Views::RawBuffer => {
                // Raw buffer
                let container = &mut self.raw_buffer;
                container.state.paused = self.state.paused;
                container.state.wrap = self.state.wrap;
                container.update_scroll(frame_rect.height as usize, &self.state.scroll_direction);
            }
            Views::SingleBuffer => {
                // Single buffer
                let container = &mut self.single_buffer;
                container.state.paused = self.state.paused;
                container.state.wrap = self.state.wrap;
                container.update_scroll(frame_rect.height as usize, &self.state.scroll_direction);
            }
            _ => (),
        }
        let blocks = self.get_layout_blocks(frame_rect);
        let mut area;

        // General containers
        for (i, container) in self
            .containers
            .iter_mut()
            .filter(|c| !c.state.hide)
            .enumerate()
        {
            if self.state.show == Views::Zoom {
                area = frame_rect.height;
            } else {
                area = blocks[i].height;
            }
            container.state.paused = self.state.paused;
            container.state.wrap = self.state.wrap;
            container.update_scroll(area as usize, &self.state.scroll_direction);
        }

        // Reset scroll direction so that scroll is done on each key press
        self.state.scroll_direction = ScrollDirection::NONE;
    }

    fn render_raw<B: Backend>(&mut self, frame: &mut Frame<'_, B>) {
        let container = &self.raw_buffer;
        container.render(frame, frame.size());
    }

    fn render_single<B: Backend>(&mut self, frame: &mut Frame<'_, B>) {
        let container = &self.single_buffer;
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
            self.containers[index].reset();
            self.containers.swap_remove(index);
        }
        self.containers.sort_by_key(|container| container.id);
    }

    fn render_help<B: Backend>(&self, frame: &mut Frame<'_, B>) {
        if self.state.help {
            render_help(frame);
        }
    }

    fn render_bar_chart<B: Backend>(&self, frame: &mut Frame<'_, B>) {
        if self.state.barchart {
            render_bar_chart(frame, self);
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
            Views::Containers => self.render_containers(frame),
            Views::RawBuffer => self.render_raw(frame),
            Views::SingleBuffer => self.render_single(frame),
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
        self.render_bar_chart(frame);
        self.render_input(frame);
    }
}

#[cfg(test)]
mod tests {
    use ratatui::{
        backend::TestBackend,
        buffer::Buffer,
        style::{Color, Modifier, Style},
        Terminal,
    };

    use super::*;

    #[test]
    fn test_new() {
        let mut app = App::new(None);

        // Running
        assert!(!app.is_running());
        app.init().unwrap();
        assert!(app.is_running());
        app.stop();
        assert!(!app.is_running());

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
        assert!(!app.show_input());
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
        assert!(c.cb.is_empty());
        assert!(app.raw_buffer.cb.is_empty());
        assert_eq!(app.raw_buffer.cb.len(), 0);
        app.init().unwrap();
        app.stdin.sender.send("abc".to_string()).unwrap();
        app.tick();

        app.stdin.sender.send("def".to_string()).unwrap();
        app.tick();

        let c = app.containers.get(0).unwrap();
        assert!(!c.cb.is_empty());
        assert_eq!(c.cb.len(), 1);
        assert!(!app.raw_buffer.cb.is_empty());
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
        let backend = TestBackend::new(16, 14);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                app.render_containers(f);
            })
            .unwrap();
        let mut expected = Buffer::with_lines(vec![
            "┌[1]'a' (0)────┐",
            "│              │",
            "│              │",
            "│              │",
            "│              │",
            "│              │",
            "└──────────────┘",
            "┌[2]'b' (0)────┐",
            "│              │",
            "│              │",
            "│              │",
            "│              │",
            "│              │",
            "└──────────────┘",
        ]);
        let bolds = 1..=10;
        for x in 0..=15 {
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
        let backend = TestBackend::new(16, 14);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                app.render(f);
            })
            .unwrap();
        let mut expected = Buffer::with_lines(vec![
            "┌[1]'a' (0)────┐",
            "│              │",
            "│              │",
            "│              │",
            "│              │",
            "│              │",
            "│              │",
            "│              │",
            "│              │",
            "│              │",
            "│              │",
            "│              │",
            "│              │",
            "└──────────────┘",
        ]);
        let bolds = 1..=10;
        for x in 0..=15 {
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
            "┌[2]'b' (0)────┐",
            "│              │",
            "│              │",
            "│              │",
            "│              │",
            "│              │",
            "│              │",
            "│              │",
            "│              │",
            "│              │",
            "│              │",
            "│              │",
            "│              │",
            "└──────────────┘",
        ]);
        let bolds = 1..=10;
        for x in 0..=15 {
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
    fn render_single_view() {
        let mut app = App::new(None);
        app.add_container("a");
        for c in app.containers.iter_mut() {
            c.state.color = Color::White;
        }
        app.state.show = Views::SingleBuffer;
        let backend = TestBackend::new(17, 14);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                app.render(f);
            })
            .unwrap();
        let mut expected = Buffer::with_lines(vec![
            "┌[0]'single' (0)┐",
            "│               │",
            "│               │",
            "│               │",
            "│               │",
            "│               │",
            "│               │",
            "│               │",
            "│               │",
            "│               │",
            "│               │",
            "│               │",
            "│               │",
            "└───────────────┘",
        ]);

        let bolds = 1..=15;
        for x in 0..=16 {
            for y in 0..=13 {
                if bolds.contains(&x) && (y == 0) {
                    let st = Style::default().add_modifier(Modifier::BOLD);
                    expected.get_mut(x, y).set_style(st);
                    expected.get_mut(x, y).set_fg(Color::Red);
                } else {
                    expected.get_mut(x, y).set_fg(Color::White);
                }
                expected.get_mut(x, y).set_bg(Color::Black);
            }
        }
        dbg!(&expected);
        terminal.backend().assert_buffer(&expected);
    }

    #[test]
    fn flip_single_view() {
        let mut app = App::new(None);
        app.add_container("a");
        assert_eq!(app.state.show, Views::RawBuffer);
        app.flip_single_view();
        assert_eq!(app.state.show, Views::SingleBuffer);
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
        let backend = TestBackend::new(16, 14);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                app.render(f);
            })
            .unwrap();
        let mut expected = Buffer::with_lines(vec![
            "┌[2]'b' (0)────┐",
            "│              │",
            "│              │",
            "│              │",
            "│              │",
            "│              │",
            "│              │",
            "│              │",
            "│              │",
            "│              │",
            "│              │",
            "│              │",
            "│              │",
            "└──────────────┘",
        ]);
        let bolds = 1..=10;
        for x in 0..=15 {
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
        let backend = TestBackend::new(16, 14);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                app.render(f);
            })
            .unwrap();
        let mut expected = Buffer::with_lines(vec![
            "┌[2]'b' (0)────┐",
            "│              │",
            "│              │",
            "│              │",
            "│              │",
            "│              │",
            "│              │",
            "│              │",
            "│              │",
            "│              │",
            "│              │",
            "│              │",
            "│              │",
            "└──────────────┘",
        ]);
        let bolds = 1..=10;
        for x in 0..=15 {
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
            "┌[1]'a' (0)────┐",
            "│              │",
            "│              │",
            "│              │",
            "│              │",
            "│              │",
            "└──────────────┘",
            "┌[2]'b' (0)────┐",
            "│              │",
            "│              │",
            "│              │",
            "│              │",
            "│              │",
            "└──────────────┘",
        ]);
        let bolds = 1..=10;
        for x in 0..=15 {
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
            assert!(!c.state.paused);
            assert!(!c.state.wrap);
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
        app.update_containers(rect);

        for c in app.containers.iter() {
            assert_eq!(c.state.paused, app.state.paused);
            assert_eq!(c.state.wrap, app.state.wrap);
        }
        app.update_containers(rect);
    }
}
