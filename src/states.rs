use tui::layout::Direction;
use tui::style::{Color, Style};

#[derive(Debug, Eq, PartialEq)]
pub enum Views {
    RawBuffer,
    Containers,
    Zoom,
    Remove,
}

#[derive(Debug, Eq, PartialEq)]
pub struct AppState {
    pub running: bool,
    pub paused: bool,
    pub show: Views,
    pub wrap: bool,
    pub help: bool,
    pub show_input: bool,
    pub zoom_id: Option<u8>,
    pub scroll_up: u16,
    pub scroll_down: u16,
    pub direction: Direction,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            running: false,
            paused: false,
            wrap: false,
            show: Views::Containers,
            direction: Direction::Vertical,
            help: false,
            show_input: false,
            zoom_id: None,
            scroll_up: 0,
            scroll_down: 0,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ContainerState {
    pub paused: bool,
    pub wrap: bool,
    pub scroll: u16,
    pub color: Color,
    pub style: Style,
}

impl Default for ContainerState {
    fn default() -> Self {
        Self {
            paused: false,
            wrap: false,
            scroll: 0,
            color: Color::Red,
            style: Style::default().fg(Color::White).bg(Color::Black),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_container_state() {
        let cs = ContainerState::default();
        assert_eq!(cs.paused, false);
        assert_eq!(cs.wrap, false);
        assert_eq!(cs.scroll, 0);
        assert_eq!(cs.color, Color::Red);
        assert_eq!(cs.style, Style::default().fg(Color::White).bg(Color::Black));
    }

    #[test]
    fn test_app_state() {
        let appstate = AppState::default();
        assert_eq!(appstate.wrap, false);
        assert_eq!(appstate.paused, false);
        assert_eq!(appstate.running, false);
        assert_eq!(appstate.show, Views::Containers);
        assert_eq!(appstate.direction, Direction::Vertical);
        assert_eq!(appstate.help, false);
        assert_eq!(appstate.show_input, false);
        assert_eq!(appstate.zoom_id, None);
        assert_eq!(appstate.scroll_up, 0);
        assert_eq!(appstate.scroll_down, 0);
    }
}
