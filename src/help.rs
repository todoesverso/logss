use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::terminal::Frame;
use tui::text::{Span, Spans};
use tui::widgets::Clear;
use tui::widgets::{Block, Borders, Paragraph};

pub fn render_help<B: Backend>(frame: &mut Frame<'_, B>) {
    let size = frame.size();
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
            "p       - toggles scrolling",
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
    let block = Block::default().title("Help").borders(Borders::ALL);
    let paragraph = Paragraph::new(help_text.clone()).block(block);
    let area = centered_rect(35, 26, size);

    frame.render_widget(Clear, area); // this clears out the background
    frame.render_widget(paragraph, area);
    //(area, paragraph)
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
