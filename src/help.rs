use crate::popup::render_popup;
use tui::backend::Backend;
use tui::style::{Color, Style};
use tui::terminal::Frame;
use tui::text::{Span, Spans};

pub fn render_help<B: Backend>(frame: &mut Frame<'_, B>) {
    let help_text = vec![
        Spans::from(Span::styled(
            "h       - toggles help popup",
            Style::default().bg(Color::Blue),
        )),
        Spans::from(Span::styled(
            "w       - toggles text wrapping",
            Style::default().bg(Color::Blue),
        )),
        Spans::from(Span::styled(
            "i       - input new container (Enter/Esc)",
            Style::default().bg(Color::Blue),
        )),
        Spans::from(Span::styled(
            "p       - toggles scrolling",
            Style::default().bg(Color::Blue),
        )),
        Spans::from(Span::styled(
            "v       - toggles vertical",
            Style::default().bg(Color::Blue),
        )),
        Spans::from(Span::styled(
            "*       - toggles between containers and raw input",
            Style::default().bg(Color::Blue),
        )),
        Spans::from(Span::styled(
            "0-9     - toggles zoom to specific container",
            Style::default().bg(Color::Blue),
        )),
        Spans::from(Span::styled(
            "Alt+0-9 - removes specific container",
            Style::default().bg(Color::Blue),
        )),
        Spans::from(Span::styled(
            "Up/Down - Scrolls lines",
            Style::default().bg(Color::Blue),
        )),
        Spans::from(Span::styled(
            "c       - continues autoscroll",
            Style::default().bg(Color::Blue),
        )),
        Spans::from(Span::styled(
            "Esc     - exits the program",
            Style::default().bg(Color::Red),
        )),
    ];
    render_popup(frame, "Help", help_text, (50, 50));
}
