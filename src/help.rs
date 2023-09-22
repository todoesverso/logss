use crate::popup::render_popup;
use ratatui::backend::Backend;
use ratatui::style::{Color, Style};
use ratatui::terminal::Frame;
use ratatui::text::{Line, Span};

pub fn render_help<B: Backend>(frame: &mut Frame<'_, B>) {
    let help_text = vec![
        Line::from(Span::styled(
            "h       - toggles help popup",
            Style::default().bg(Color::Blue),
        )),
        Line::from(Span::styled(
            "b       - toggles BarChart popup",
            Style::default().bg(Color::Blue),
        )),
        Line::from(Span::styled(
            "w       - toggles text wrapping",
            Style::default().bg(Color::Blue),
        )),
        Line::from(Span::styled(
            "i|/     - input new container (Enter/Esc)",
            Style::default().bg(Color::Blue),
        )),
        Line::from(Span::styled(
            "p|Space - toggles scrolling",
            Style::default().bg(Color::Blue),
        )),
        Line::from(Span::styled(
            "v       - toggles vertical",
            Style::default().bg(Color::Blue),
        )),
        Line::from(Span::styled(
            "*       - toggles between containers and raw input",
            Style::default().bg(Color::Blue),
        )),
        Line::from(Span::styled(
            "s       - toggles between containers and single input",
            Style::default().bg(Color::Blue),
        )),
        Line::from(Span::styled(
            "1-9     - toggles zoom to specific container",
            Style::default().bg(Color::Blue),
        )),
        Line::from(Span::styled(
            "Alt+1-9 - removes specific container",
            Style::default().bg(Color::Blue),
        )),
        Line::from(Span::styled(
            "F1/9    - toggles hide/show for container",
            Style::default().bg(Color::Blue),
        )),
        Line::from(Span::styled(
            "Up/Down - Scrolls lines",
            Style::default().bg(Color::Blue),
        )),
        Line::from(Span::styled(
            "c       - continues autoscroll",
            Style::default().bg(Color::Blue),
        )),
        Line::from(Span::styled(
            "Esc     - exits the program",
            Style::default().bg(Color::Red),
        )),
    ];
    render_popup(frame, "Help", &help_text, (50, 50));
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    #[test]
    fn test_render_help() {
        let backend = TestBackend::new(40, 40);
        let mut terminal = Terminal::new(backend).unwrap();
        let res = terminal
            .draw(|f| {
                render_help(f);
            })
            .is_ok();
        assert!(res);
    }
}
